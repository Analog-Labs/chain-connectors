use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use ethers::prelude::*;
use ethers::providers::JsonRpcClient;
use ethers::providers::JsonRpcError;
use futures_timer::Delay;
use jsonrpsee::{
    client_transport::ws::{WsHandshakeError, WsTransportClientBuilder},
    core::{
        client::{Client, ClientBuilder, ClientT},
        error::Error as JsonRpseeError,
        traits::ToRpcParams,
    },
    rpc_params,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::value::RawValue;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use std::time::Duration;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum RpcClientError {
    /// Thrown if the response could not be parsed
    #[error("{original}")]
    JsonRpcError {
        original: JsonRpseeError,
        message: Option<JsonRpcError>,
    },

    /// Failed to parse the data.
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

    /// Error that can happen during the WebSocket handshake.
    #[error("WS Handshake failed: {0}")]
    HandshakeFailed(WsHandshakeError),

    /// The background task has been terminated.
    #[error("The background task been terminated because: {0}; restart required")]
    RestartNeeded(String),

    /// The client is reconnecting
    #[error("The client is restarting the background task")]
    Reconnecting,
}

impl From<JsonRpseeError> for RpcClientError {
    fn from(error: JsonRpseeError) -> Self {
        match error {
            JsonRpseeError::Call(call) => {
                let code = call.code() as i64;
                let data = call
                    .data()
                    .and_then(|raw_value| serde_json::value::to_value(raw_value).ok());
                let message = call.message().to_string();
                Self::JsonRpcError {
                    original: JsonRpseeError::Call(call),
                    message: Some(JsonRpcError {
                        code,
                        message,
                        data,
                    }),
                }
            }
            JsonRpseeError::ParseError(serde_error) => Self::ParseError(serde_error),
            JsonRpseeError::RestartNeeded(reason) => Self::RestartNeeded(reason),
            error => {
                let message = format!("{}", &error);
                Self::JsonRpcError {
                    original: error,
                    message: Some(JsonRpcError {
                        code: 9999,
                        message,
                        data: None,
                    }),
                }
            }
        }
    }
}

impl From<WsHandshakeError> for RpcClientError {
    fn from(error: WsHandshakeError) -> Self {
        Self::HandshakeFailed(error)
    }
}

impl From<RpcClientError> for ProviderError {
    fn from(error: RpcClientError) -> Self {
        match error {
            RpcClientError::ParseError(error) => ProviderError::SerdeJson(error),
            RpcClientError::HandshakeFailed(error) => ProviderError::CustomError(error.to_string()),
            error => ProviderError::JsonRpcClientError(Box::new(error)),
        }
    }
}

impl RpcError for RpcClientError {
    fn as_error_response(&self) -> Option<&JsonRpcError> {
        match self {
            RpcClientError::JsonRpcError { message, .. } => message.as_ref(),
            _ => None,
        }
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        match self {
            Self::ParseError(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Params(Option<Box<RawValue>>);

impl Params {
    pub fn from_serializable<T>(params: &T) -> Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        let params = serde_json::value::to_raw_value(params)?;
        Ok(Self(Some(params)))
    }
}

impl ToRpcParams for Params {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, JsonRpseeError> {
        Ok(self.0)
    }
}

impl Display for Params {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(v) => Display::fmt(v.as_ref(), f),
            None => f.write_str("null"),
        }
    }
}

#[derive(Debug)]
pub struct JsonRpseeClient {
    uri: Url,
    client: ArcSwapOption<Client>,
}

impl JsonRpseeClient {
    pub async fn connect(uri: Url) -> Result<Self, RpcClientError> {
        let (tx, rx) = WsTransportClientBuilder::default()
            .build(uri.clone())
            .await?;
        let client = ClientBuilder::default().build_with_tokio(tx, rx);
        Ok(Self {
            uri,
            client: ArcSwapOption::new(Some(Arc::new(client))),
        })
    }

    // TODO: reconnect in a different thread, otherwise the request will block
    pub async fn reconnect(&self) -> Result<(), RpcClientError> {
        // Check if is connected
        let Some(client) = self.client.load().clone() else {
            return Ok(());
        };

        if client.is_connected() && is_connected(client.as_ref()).await {
            self.client.store(Some(client));
            return Ok(());
        }

        // Guarantee that only one thread is attempting to reconnect
        if self.client.swap(None).is_none() {
            return Ok(());
        };

        // TODO: close the client after a given number of attempts
        let mut attempts = 0;
        loop {
            attempts += 1;

            log::info!("Reconnecting... attempt {attempts}");
            let result = WsTransportClientBuilder::default()
                .build(self.uri.clone())
                .await;

            match result {
                Ok((tx, rx)) => {
                    log::info!("Client was successfully reconnected: {}", self.uri);
                    let client = ClientBuilder::default().build_with_tokio(tx, rx);
                    self.client.store(Some(Arc::new(client)));
                    break;
                }
                Err(e) => {
                    log::error!("Reconnect attempt failed: {}", e);
                }
            };
            Delay::new(Duration::from_secs(5)).await;
        }
        Ok(())
    }
}

// const ETHEREUM_SUBSCRIBE_METHOD: &str = "eth_subscribe";
// const ETHEREUM_UNSUBSCRIBE_METHOD: &str = "eth_unsubscribe";

#[async_trait]
impl JsonRpcClient for JsonRpseeClient {
    type Error = RpcClientError;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        let Some(client) = self.client.load().clone() else {
            log::error!("Requested failed, client is reconnecting...");
            return Err(RpcClientError::Reconnecting);
        };
        let params = Params::from_serializable(&params)?;

        log::info!("{method} {params}");
        let result = client.request::<R, Params>(method, params).await;

        match result {
            Err(JsonRpseeError::RestartNeeded(reason)) => {
                log::error!("Requested failed, WS Connection is close: {reason}");
                self.reconnect().await?;
                Err(RpcClientError::Reconnecting)
            }
            result => result.map_err(RpcClientError::from),
        }
    }
}

// enum SubscriptionStreamState {
//     Pending(Arc<Client>),
//     Requesting {
//         client: Arc<Client>,
//         future: BoxFuture<'static, Result<Subscription<serde_json::Value>, JsonRpseeError>>,
//     },
//     Ready(Subscription<serde_json::Value>),
// }

// impl PubsubClient for JsonRpseeClient {
//     type NotificationStream = <Ws as PubsubClient>::NotificationStream;
//
//     /// Add a subscription to this transport
//     fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, RpcClientError> {
//         let Some(client) = self.client.load().clone() else {
//             log::error!("Requested failed, client is reconnecting...");
//             return Err(RpcClientError::Reconnecting);
//         };
//
//         let client = self.client.load().clone();
//         PubsubClient::subscribe(client.as_ref(), id)
//     }
//
//     /// Remove a subscription from this transport
//     fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error> {
//         if self.is_reconnecting.load(Ordering::Relaxed) {
//             log::error!("unsubscribe {} failed, client is reconnecting", id.into());
//             return Err(WsClientError::UnexpectedClose);
//         }
//
//         let client = self.client.load().clone();
//         PubsubClient::unsubscribe(client.as_ref(), id)
//     }
// }

pub async fn is_connected(provider: &Client) -> bool {
    let result = provider
        .request::<U64, _>("eth_blockNumber", rpc_params![])
        .await;
    !matches!(
        result,
        Err(JsonRpseeError::RestartNeeded(_)
            | JsonRpseeError::AlreadyStopped
            | JsonRpseeError::Transport(_))
    )
}

// Websocket Provider that supports reconnecting
// #[derive(Debug)]
// pub struct ExtendedWs {
//     conn_details: ConnectionDetails,
//     client: ArcSwap<Ws>,
//     is_reconnecting: Arc<AtomicBool>,
// }
//
// impl ExtendedWs {
//     pub async fn connect(conn: impl Into<ConnectionDetails>) -> Result<Self, WsClientError> {
//         let conn_details = conn.into();
//         let client = Ws::connect(conn_details.clone()).await?;
//         Ok(Self {
//             conn_details,
//             client: ArcSwap::new(Arc::from(client)),
//             is_reconnecting: Arc::new(AtomicBool::new(false)),
//         })
//     }
//
//     // TODO: reconnect in a different thread, otherwise the request will block
//     pub async fn reconnect(&self) -> Result<(), WsClientError> {
//         // Guarantee that only one thread is attempting to reconnect
//         if self.is_reconnecting.swap(true, Ordering::SeqCst) {
//             return Err(WsClientError::UnexpectedClose);
//         }
//
//         // Get the current client
//         let client = self.client.load();
//
//         // Check if is already connected
//         if is_connected(&client).await {
//             return Ok(());
//         }
//
//         // TODO: close the client after a given number of attempts
//         let mut attempts = 0;
//         loop {
//             attempts += 1;
//
//             log::info!("Retrying to connect... attempt {attempts}");
//             let client = Ws::connect(self.conn_details.clone()).await;
//
//             match client {
//                 Ok(client) => {
//                     log::info!("Client reconnected successfully: {}", self.conn_details.url);
//                     self.client.store(Arc::from(client));
//                     break;
//                 }
//                 Err(e) => {
//                     log::error!("Failed to reconnect: {:?}", e);
//                 }
//             };
//
//             Delay::new(Duration::from_secs(5)).await;
//         }
//         self.is_reconnecting.store(false, Ordering::SeqCst);
//         Ok(())
//     }
// }
//
// #[async_trait]
// impl JsonRpcClient for ExtendedWs {
//     type Error = WsClientError;
//
//     async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
//     where
//         T: Debug + Serialize + Send + Sync,
//         R: DeserializeOwned + Send,
//     {
//         if self.is_reconnecting.load(Ordering::Relaxed) {
//             log::error!("Cannot process request, client is reconnecting: {method}");
//             return Err(WsClientError::UnexpectedClose);
//         }
//
//         let provider = self.client.load().clone();
//         let result = JsonRpcClient::request(&provider, method, params).await;
//
//         // Attempt to reconnect Connection unexpectedly closed
//         // TODO: execute this in a different thread/task, this will block the request
//         match result {
//             Err(WsClientError::UnexpectedClose) => {
//                 log::error!("Websocket closed unexpectedly, reconnecting...");
//                 self.reconnect().await?;
//                 Err(WsClientError::UnexpectedClose)
//             }
//             Err(err) => {
//                 // Log the error
//                 log::error!("{err:?}");
//                 Err(err)
//             }
//             _ => result,
//         }
//     }
// }
//
// impl PubsubClient for ExtendedWs {
//     type NotificationStream = <Ws as PubsubClient>::NotificationStream;
//
//     /// Add a subscription to this transport
//     fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, WsClientError> {
//         if self.is_reconnecting.load(Ordering::Relaxed) {
//             log::error!("subscription {} failed, client is reconnecting", id.into());
//             return Err(WsClientError::UnexpectedClose);
//         }
//
//         let client = self.client.load().clone();
//         PubsubClient::subscribe(client.as_ref(), id)
//     }
//
//     /// Remove a subscription from this transport
//     fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error> {
//         if self.is_reconnecting.load(Ordering::Relaxed) {
//             log::error!("unsubscribe {} failed, client is reconnecting", id.into());
//             return Err(WsClientError::UnexpectedClose);
//         }
//
//         let client = self.client.load().clone();
//         PubsubClient::unsubscribe(client.as_ref(), id)
//     }
// }
//
// pub async fn is_connected(provider: &Ws) -> bool {
//     let result = JsonRpcClient::request::<_, U64>(provider, "eth_blockNumber", ()).await;
//     !matches!(
//         result,
//         Err(WsClientError::UnexpectedClose) | Err(WsClientError::InternalError(_))
//     )
// }
