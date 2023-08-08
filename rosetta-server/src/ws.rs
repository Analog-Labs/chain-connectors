#![cfg(feature = "ws")]
mod config;
mod jsonrpsee_client;
mod tungstenite_jsonrpsee;

pub use config::{RpcClientConfig, WsTransportClient};
pub use jsonrpsee::client_transport::ws::WsHandshakeError;
use jsonrpsee::core::client::{Client, ClientBuilder};
pub use jsonrpsee_client::RpcClient;
pub use tungstenite_jsonrpsee::{TungsteniteClient, WsError};

pub async fn default_client(
    url: &str,
    config: Option<RpcClientConfig>,
) -> Result<RpcClient, WsHandshakeError> {
    let config = config.unwrap_or_default();
    let rpc_client_builder = ClientBuilder::from(&config);

    let client = match config.client {
        WsTransportClient::Auto => {
            log::info!("Connecting using Socketto...");
            match build_socketto_client(rpc_client_builder, url, &config).await {
                Ok(client) => client,
                Err(error) => {
                    log::warn!("Socketto failed: {}", error);
                    log::trace!("Retrying to connect using Tungstenite.");
                    build_tungstenite_client(rpc_client_builder, url, &config).await?
                }
            }
        }
        WsTransportClient::Socketto => {
            let client = build_socketto_client(rpc_client_builder, url, &config).await?;
            log::info!("Connected to {} using Socketto", url);
            client
        }
        WsTransportClient::Tungstenite => {
            let client = build_tungstenite_client(rpc_client_builder, url, &config).await?;
            log::info!("Connected to {} using Tungstenite", url);
            client
        }
    };
    Ok(RpcClient(client))
}

/// Creates a default jsonrpsee client using socketto.
async fn build_socketto_client(
    builder: ClientBuilder,
    uri: &str,
    config: &RpcClientConfig,
) -> Result<Client, WsHandshakeError> {
    use jsonrpsee::client_transport::ws::WsTransportClientBuilder;
    use tokio_tungstenite::tungstenite::http::uri::{Authority, Uri};

    let mut uri = uri
        .parse::<Uri>()
        .map_err(|e| WsHandshakeError::Url(e.to_string().into()))?;
    let default_port = match uri.scheme().map(|s| s.as_str()) {
        Some("ws" | "http") => Some(80),
        Some("wss" | "https") => Some(443),
        _ => None,
    };

    // Set default port, workaround because jsonrpsee doesn't infer the url port:
    // https://github.com/paritytech/jsonrpsee/blob/v0.19.0/client/transport/src/ws/mod.rs#L509-L511
    if let (None, Some(port), Some(authority)) =
        (uri.port(), default_port, uri.authority().cloned())
    {
        let new_authority =
            Authority::try_from(format!("{}:{}", authority, port)).unwrap_or(authority);
        uri = {
            let mut parts = uri.clone().into_parts();
            parts.authority = Some(new_authority);
            Uri::from_parts(parts).unwrap_or(uri)
        };
    }

    let (sender, receiver) = WsTransportClientBuilder::from(config).build(uri).await?;
    let client = builder.build_with_tokio(sender, receiver);
    Ok(client)
}

/// Creates a default jsonrpsee client using tungstenite.
async fn build_tungstenite_client(
    builder: ClientBuilder,
    url: &str,
    config: &RpcClientConfig,
) -> Result<Client, WsHandshakeError> {
    use tide::http::url::Url;

    let url = url
        .parse::<Url>()
        .map_err(|e| WsHandshakeError::Url(e.to_string().into()))?;
    let client = TungsteniteClient::new(url, config)
        .await
        .map_err(|e| match e {
            WsError::Url(error) => WsHandshakeError::Url(error.to_string().into()),
            WsError::Io(error) => WsHandshakeError::Io(error),
            _ => WsHandshakeError::Url(e.to_string().into()),
        })?;
    let (sender, receiver) = client.split();
    let client = builder.build_with_tokio(sender, receiver);
    Ok(client)
}