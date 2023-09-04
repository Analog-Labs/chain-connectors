#![cfg(feature = "ws")]
mod config;
mod error;
mod jsonrpsee_client;
mod reconnect;
mod reconnect_impl;
mod retry_strategy;
mod tungstenite_jsonrpsee;

use crate::ws::reconnect::{AutoReconnectClient, Reconnect};
use crate::ws::reconnect_impl::{Config as ReconnectConfig, DefaultStrategy};
use crate::ws::retry_strategy::RetryStrategy;
pub use config::{RpcClientConfig, WsTransportClient};
use futures_util::{future::BoxFuture, FutureExt};
use jsonrpsee::core::Error as JsonRpseeError;
use jsonrpsee::{
    client_transport::ws::WsTransportClientBuilder,
    core::client::{Client, ClientBuilder},
};
use std::time::Duration;
use tide::http::url::Url;
pub use tungstenite_jsonrpsee::{TungsteniteClient, WsError};

pub type DefaultClient = AutoReconnectClient<DefaultStrategy<DefaultReconnectConfig>>;

async fn connect_client(url: Url, config: RpcClientConfig) -> Result<Client, JsonRpseeError> {
    let builder = ClientBuilder::from(&config);
    let client = match config.client {
        WsTransportClient::Auto => {
            log::info!("Connecting using Socketto...");
            match build_socketto_client(builder, url.clone(), &config).await {
                Ok(client) => client,
                Err(error) => {
                    log::warn!("Socketto failed: {}", error);
                    log::trace!("Retrying to connect using Tungstenite.");
                    build_tungstenite_client(builder, url, &config).await?
                }
            }
        }
        WsTransportClient::Socketto => {
            let client = build_socketto_client(builder, url.clone(), &config).await?;
            log::info!("Connected to {} using Socketto", url);
            client
        }
        WsTransportClient::Tungstenite => {
            let client = build_tungstenite_client(builder, url.clone(), &config).await?;
            log::info!("Connected to {} using Tungstenite", url);
            client
        }
    };
    Ok(client)
}

#[derive(Debug)]
pub struct DefaultReconnectConfig {
    /// Url to connect to.
    url: Url,

    /// RPC Client configuration.
    config: RpcClientConfig,
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

pub async fn default_client(
    url: &str,
    config: Option<RpcClientConfig>,
) -> Result<DefaultClient, JsonRpseeError> {
    let config = config.unwrap_or_default();
    let url = url
        .parse::<Url>()
        .map_err(|e| JsonRpseeError::Transport(anyhow::Error::from(e)))?;
    let reconnect_config = DefaultReconnectConfig { url, config };

    DefaultStrategy::connect(reconnect_config)
        .await
        .map(|strategy| strategy.into_client())
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
        .map_err(|error| JsonRpseeError::Transport(anyhow::Error::from(error)))?;
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
        .map_err(|error| JsonRpseeError::Transport(anyhow::Error::from(error)))?;
    let (sender, receiver) = client.split();
    let client = builder.build_with_tokio(sender, receiver);
    Ok(client)
}
