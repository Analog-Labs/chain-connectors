#![cfg(feature = "ws")]
mod config;
mod error;
mod extension;
mod jsonrpsee_client;
mod reconnect;
mod reconnect_impl;
mod tungstenite_jsonrpsee;

use crate::ws::reconnect::{AutoReconnectClient, Reconnect};
use crate::ws::reconnect_impl::{Config as ReconnectConfig, DefaultStrategy};
pub use config::{RpcClientConfig, WsTransportClient};
use futures_util::{future::BoxFuture, FutureExt};
use jsonrpsee::core::Error as JsonRpseeError;
use jsonrpsee::{
    client_transport::ws::WsTransportClientBuilder,
    core::client::{Client, ClientBuilder},
};
use tide::http::url::Url;
pub use tungstenite_jsonrpsee::{TungsteniteClient, WsError};

async fn build_client(url: Url, config: RpcClientConfig) -> Result<Client, JsonRpseeError> {
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

pub struct ReconnectConfigImpl {
    url: Url,
    config: RpcClientConfig,
}
impl ReconnectConfig for ReconnectConfigImpl {
    type Client = Client;
    type ConnectFuture = BoxFuture<'static, Result<Client, JsonRpseeError>>;

    fn connect(&self) -> Self::ConnectFuture {
        let url = self.url.clone();
        let config = self.config.clone();
        async move { build_client(url, config).await }.boxed()
    }
}

pub async fn default_client(
    url: &str,
    config: Option<RpcClientConfig>,
) -> Result<AutoReconnectClient<DefaultStrategy<ReconnectConfigImpl>>, JsonRpseeError> {
    let config = config.unwrap_or_default();

    let url = url
        .parse::<Url>()
        .map_err(|e| JsonRpseeError::Transport(anyhow::Error::from(e)))?;

    let reconnect_config = ReconnectConfigImpl { url, config };

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
