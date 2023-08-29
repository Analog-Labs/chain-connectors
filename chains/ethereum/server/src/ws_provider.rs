use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use dashmap::DashMap;
use ethers::prelude::*;
use ethers::providers::JsonRpcClient;
use ethers::providers::JsonRpcError;
use ethers::types::U256;
use futures_timer::Delay;
use futures_util::future::BoxFuture;
use futures_util::{FutureExt, Stream};
use jsonrpsee::core::client::{Subscription, SubscriptionClientT, SubscriptionKind};
use jsonrpsee::{
    client_transport::ws::{WsHandshakeError, WsTransportClientBuilder},
    core::{
        client::{Client, ClientBuilder, ClientT},
        error::Error as JsonRpseeError,
        traits::ToRpcParams,
    },
    rpc_params,
};
use pin_project::pin_project;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::value::RawValue;
use std::fmt::{Debug, Display, Formatter};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
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

const ETHEREUM_SUBSCRIBE_METHOD: &str = "eth_subscribe";
const ETHEREUM_UNSUBSCRIBE_METHOD: &str = "eth_unsubscribe";

// Websocket Client that supports reconnecting
#[derive(Debug)]
pub struct JsonRpseeClient {
    uri: Url,
    client: ArcSwapOption<Client>,
    subscriptions: DashMap<U256, Subscription<serde_json::Value>>,
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
            subscriptions: DashMap::new(),
        })
    }

    // TODO: reconnect in a different thread, otherwise the request will block
    pub async fn reconnect(&self) -> Result<(), RpcClientError> {
        // Check if is connected
        let Ok(client) = self.client() else {
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

    pub fn client(&self) -> Result<Arc<Client>, RpcClientError> {
        let Some(client) = self.client.load().clone() else {
            log::error!("Requested failed, client is reconnecting...");
            return Err(RpcClientError::Reconnecting);
        };
        Ok(client)
    }

    pub async fn request<R>(&self, method: &str, params: Params) -> Result<R, RpcClientError>
    where
        R: DeserializeOwned + Send,
    {
        let client = self.client()?;
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

    pub async fn subscribe<R>(&self, params: Params) -> Result<R, RpcClientError>
    where
        R: DeserializeOwned + Send,
    {
        let client = self.client()?;

        let response = client
            .subscribe::<serde_json::Value, _>(
                ETHEREUM_SUBSCRIBE_METHOD,
                params,
                ETHEREUM_UNSUBSCRIBE_METHOD,
            )
            .await;
        let stream = match response {
            Err(JsonRpseeError::RestartNeeded(reason)) => {
                log::error!("Subscription failed, WS Connection is close: {reason}");
                self.reconnect().await?;
                return Err(RpcClientError::Reconnecting);
            }
            result => result.map_err(RpcClientError::from)?,
        };

        // The ethereum subscription id must be an U256
        let maybe_id = match stream.kind() {
            SubscriptionKind::Subscription(id) => serde_json::to_value(id).ok(),
            _ => None,
        }
        .and_then(|value| {
            let subscription_id = serde_json::from_value::<U256>(value.clone()).ok()?;
            let result = serde_json::from_value::<R>(value).ok()?;
            Some((subscription_id, result))
        });

        // Unsubscribe in case of error
        let Some((subscription_id, result)) = maybe_id else {
            stream.unsubscribe().await?;
            return Err(RpcClientError::JsonRpcError {
                original: JsonRpseeError::InvalidSubscriptionId,
                message: None,
            });
        };

        let _ = self.subscriptions.insert(subscription_id, stream);
        Ok(result)
    }
}

#[async_trait]
impl JsonRpcClient for JsonRpseeClient {
    type Error = RpcClientError;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        let params = Params::from_serializable(&params)?;

        log::info!("{method} {params}");
        match method {
            ETHEREUM_SUBSCRIBE_METHOD => self.subscribe::<R>(params).await,
            _ => self.request::<R>(method, params).await,
        }
    }
}

pub enum SubscriptionStreamState {
    Subscribed(Subscription<serde_json::Value>),
    Unsubscribing(BoxFuture<'static, Result<(), JsonRpseeError>>),
}

#[pin_project]
pub struct SubscriptionStream {
    id: U256,
    failures: u32,
    state: Option<SubscriptionStreamState>,
}

impl SubscriptionStream {
    pub fn new(id: U256, stream: Subscription<serde_json::Value>) -> Self {
        Self {
            id,
            failures: 0,
            state: Some(SubscriptionStreamState::Subscribed(stream)),
        }
    }
}

impl Stream for SubscriptionStream {
    type Item = Box<RawValue>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        loop {
            match this.state.take() {
                Some(SubscriptionStreamState::Subscribed(mut stream)) => {
                    let result = match stream.poll_next_unpin(cx) {
                        Poll::Ready(result) => result,
                        Poll::Pending => {
                            *this.state = Some(SubscriptionStreamState::Subscribed(stream));
                            return Poll::Pending;
                        }
                    };

                    // Stream is close
                    let Some(result) = result else {
                        return Poll::Ready(None);
                    };

                    // Parse the result
                    let result = result.and_then(|value| {
                        serde_json::value::to_raw_value(&value).map_err(JsonRpseeError::ParseError)
                    });

                    match result {
                        Ok(value) => {
                            *this.state = Some(SubscriptionStreamState::Subscribed(stream));
                            return Poll::Ready(Some(value));
                        }
                        Err(error) => {
                            log::error!(
                                "Invalid response from subscription error {}: {:?}",
                                this.failures,
                                error
                            );
                            *this.failures += 1;

                            if *this.failures > 5 {
                                log::error!("Too many errors, unsubscribing...");
                                *this.state = Some(SubscriptionStreamState::Unsubscribing(
                                    stream.unsubscribe().boxed(),
                                ));
                            } else {
                                *this.state = Some(SubscriptionStreamState::Subscribed(stream));
                            }
                            continue;
                        }
                    }
                }
                Some(SubscriptionStreamState::Unsubscribing(mut future)) => {
                    return match future.poll_unpin(cx) {
                        Poll::Ready(Ok(_)) => Poll::Ready(None),
                        Poll::Ready(Err(error)) => {
                            log::error!("Failed to unsubscribe: {:?}", error);
                            Poll::Ready(None)
                        }
                        Poll::Pending => {
                            *this.state = Some(SubscriptionStreamState::Unsubscribing(future));
                            Poll::Pending
                        }
                    };
                }
                None => {
                    log::error!("stream must not be polled after being closed`");
                    return Poll::Ready(None);
                }
            }
        }
    }
}

impl PubsubClient for JsonRpseeClient {
    type NotificationStream = SubscriptionStream;

    /// Add a subscription to this transport
    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, RpcClientError> {
        let Some((id, stream)) = self.subscriptions.remove(&id.into()) else {
            return Err(RpcClientError::JsonRpcError {
                original: JsonRpseeError::InvalidSubscriptionId,
                message: None,
            });
        };

        Ok(SubscriptionStream::new(id, stream))
    }

    /// Remove a subscription from this transport
    fn unsubscribe<T: Into<U256>>(&self, _id: T) -> Result<(), Self::Error> {
        Ok(())
    }
}

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
