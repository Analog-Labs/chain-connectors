use crate::{error::Error, params::RpcParams};
use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use dashmap::DashMap;
use ethers::prelude::*;
use ethers::providers::JsonRpcClient;
use ethers::types::U256;
use futures_timer::Delay;
use futures_util::future::BoxFuture;
use futures_util::{FutureExt, Stream};
use jsonrpsee::core::client::{Subscription, SubscriptionClientT, SubscriptionKind};
use jsonrpsee::{
    client_transport::ws::WsTransportClientBuilder,
    core::{
        client::{Client, ClientBuilder, ClientT},
        error::Error as JsonRpseeError,
    },
    rpc_params,
};
use pin_project::pin_project;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::value::RawValue;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use url::Url;

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
    pub async fn connect(uri: Url) -> Result<Self, Error> {
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

    pub fn client(&self) -> Result<Arc<Client>, Error> {
        let Some(client) = self.client.load().clone() else {
            log::error!("Requested failed, client is reconnecting...");
            return Err(Error::Reconnecting);
        };
        Ok(client)
    }

    // TODO: reconnect in a different thread, otherwise the request will block
    pub async fn reconnect(&self) -> Result<(), Error> {
        // Check if is connected
        let Ok(client) = self.client() else {
            return Ok(());
        };

        if is_connected(client.as_ref()).await {
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

    pub async fn request<R>(&self, method: &str, params: RpcParams) -> Result<R, Error>
    where
        R: DeserializeOwned + Send,
    {
        let client = self.client()?;
        let result = client.request::<R, RpcParams>(method, params).await;

        match result {
            Err(JsonRpseeError::RestartNeeded(reason)) => {
                log::error!("Requested failed, WS Connection is close: {reason}");
                self.reconnect().await?;
                Err(Error::Reconnecting)
            }
            result => result.map_err(Error::from),
        }
    }

    pub async fn subscribe<R>(&self, params: RpcParams) -> Result<R, Error>
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
                return Err(Error::Reconnecting);
            }
            result => result.map_err(Error::from)?,
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
            return Err(Error::JsonRpsee {
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
    type Error = Error;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        let params = RpcParams::from_serializable(&params)?;

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
    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Error> {
        let Some((id, stream)) = self.subscriptions.remove(&id.into()) else {
            return Err(Error::JsonRpsee {
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

pub async fn is_connected(client: &Client) -> bool {
    if !client.is_connected() {
        return false;
    }

    // Do a simple request to check if the RPC node is functional
    let result = client
        .request::<U64, _>("eth_blockNumber", rpc_params![])
        .await;
    !matches!(
        result,
        Err(JsonRpseeError::RestartNeeded(_)
            | JsonRpseeError::AlreadyStopped
            | JsonRpseeError::Transport(_))
    )
}
