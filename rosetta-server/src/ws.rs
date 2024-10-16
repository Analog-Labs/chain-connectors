mod config;
mod error;
mod jsonrpsee_client;
mod reconnect;
mod reconnect_impl;
mod retry_strategy;
mod tungstenite_jsonrpsee;

use crate::ws::{
    reconnect::{AutoReconnectClient, Reconnect},
    reconnect_impl::{Config as ReconnectConfig, DefaultStrategy},
    retry_strategy::RetryStrategy,
};
pub use config::{RpcClientConfig, WsTransportClient};
use futures_util::{future::BoxFuture, FutureExt};
use jsonrpsee::{
    client_transport::ws::WsTransportClientBuilder,
    core::{
        client::{Client, ClientBuilder},
        ClientError as JsonRpseeError,
    },
};
use std::time::Duration;
pub use tungstenite_jsonrpsee::{TungsteniteClient, WsError};
use url::Url;

pub type DefaultClient = AutoReconnectClient<DefaultStrategy<DefaultReconnectConfig>>;
pub type HttpClient = jsonrpsee::http_client::HttpClient;

async fn connect_client(url: Url, config: RpcClientConfig) -> Result<Client, JsonRpseeError> {
    let builder = ClientBuilder::from(&config);
    let client = match config.client {
        WsTransportClient::Auto => {
            tracing::info!("Connecting using Socketto...");
            match build_socketto_client(builder, url.clone(), &config).await {
                Ok(client) => client,
                Err(error) => {
                    tracing::warn!("Socketto failed: {}", error);
                    tracing::trace!("Retrying to connect using Tungstenite.");
                    build_tungstenite_client(builder, url, &config).await?
                },
            }
        },
        WsTransportClient::Socketto => {
            let client = build_socketto_client(builder, url.clone(), &config).await?;
            tracing::info!("Connected to {} using Socketto", url);
            client
        },
        WsTransportClient::Tungstenite => {
            let client = build_tungstenite_client(builder, url.clone(), &config).await?;
            tracing::info!("Connected to {} using Tungstenite", url);
            client
        },
    };
    Ok(client)
}

#[derive(Debug)]
pub struct DefaultReconnectConfig {
    /// Url to connect to.
    pub url: Url,

    /// RPC Client configuration.
    pub config: RpcClientConfig,
}

impl ReconnectConfig for DefaultReconnectConfig {
    type Client = Client;
    type ConnectFuture = BoxFuture<'static, Result<Client, JsonRpseeError>>;

    /// Using fixed-interval strategy.
    type RetryStrategy = RetryStrategy;

    fn max_pending_delay(&self) -> Duration {
        self.config.rpc_request_timeout
    }

    fn retry_strategy(&self) -> Self::RetryStrategy {
        RetryStrategy::from(&self.config.retry_strategy)
    }

    fn connect(&self) -> Self::ConnectFuture {
        let url = self.url.clone();
        let config = self.config.clone();
        connect_client(url, config).boxed()
    }

    fn is_connected(&self, client: &Self::Client) -> Option<bool> {
        Some(client.is_connected())
    }
}

/// Creates an Json-RPC client with default settings
///
/// # Errors
///
/// Returns `Err` if it fails to connect to the provided `url`
pub async fn default_client(
    url: &str,
    config: Option<RpcClientConfig>,
) -> Result<DefaultClient, JsonRpseeError> {
    let config = config.unwrap_or_default();
    let url = url.parse::<Url>().map_err(|e| JsonRpseeError::Transport(e.into()))?;
    let reconnect_config = DefaultReconnectConfig { url, config };

    DefaultStrategy::connect(reconnect_config).await.map(Reconnect::into_client)
}

/// Creates an Json-RPC HTTP client with default settings
///
/// # Errors
/// Returns `Err` if the url is not valid
pub fn default_http_client(url: &str) -> Result<HttpClient, JsonRpseeError> {
    let url = url.parse::<Url>().map_err(|e| JsonRpseeError::Transport(e.into()))?;
    let client = jsonrpsee::http_client::HttpClientBuilder::new().build(url)?;
    Ok(client)
}

/// Creates a default jsonrpsee client using socketto.
async fn build_socketto_client(
    builder: ClientBuilder,
    url: Url,
    config: &RpcClientConfig,
) -> Result<Client, JsonRpseeError> {
    let (sender, receiver) = WsTransportClientBuilder::from(config)
        .build(url)
        .await
        .map_err(|error| JsonRpseeError::Transport(error.into()))?;
    let client = builder.build_with_tokio(sender, receiver);
    Ok(client)
}

/// Creates a default jsonrpsee client using tungstenite.
async fn build_tungstenite_client(
    builder: ClientBuilder,
    url: Url,
    config: &RpcClientConfig,
) -> Result<Client, JsonRpseeError> {
    let client = TungsteniteClient::new(url, config)
        .await
        .map_err(|error| JsonRpseeError::Transport(error.into()))?;
    let (sender, receiver) = client.split();
    let client = builder.build_with_tokio(sender, receiver);
    Ok(client)
}
