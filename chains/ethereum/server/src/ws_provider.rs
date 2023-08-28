use arc_swap::ArcSwap;
use async_trait::async_trait;
use ethers::prelude::*;
use ethers::providers::{ConnectionDetails, JsonRpcClient, PubsubClient, WsClientError};
use futures_timer::Delay;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Websocket Provider that supports reconnecting
#[derive(Debug)]
pub struct ExtendedWs {
    conn_details: ConnectionDetails,
    client: ArcSwap<Ws>,
    is_reconnecting: Arc<AtomicBool>,
}

impl ExtendedWs {
    pub async fn connect(conn: impl Into<ConnectionDetails>) -> Result<Self, WsClientError> {
        let conn_details = conn.into();
        let client = Ws::connect(conn_details.clone()).await?;
        Ok(Self {
            conn_details,
            client: ArcSwap::new(Arc::from(client)),
            is_reconnecting: Arc::new(AtomicBool::new(false)),
        })
    }

    // TODO: reconnect in a different thread, otherwise the request will block
    pub async fn reconnect(&self) -> Result<(), WsClientError> {
        // Guarantee that only one thread is attempting to reconnect
        if self.is_reconnecting.swap(true, Ordering::SeqCst) {
            return Err(WsClientError::UnexpectedClose);
        }

        // Get the current client
        let client = self.client.load();

        // Check if is already connected
        if is_connected(&client).await {
            return Ok(());
        }

        // TODO: close the client after a given number of attempts
        let mut attempts = 0;
        loop {
            attempts += 1;

            log::info!("Retrying to connect... attempt {attempts}");
            let client = Ws::connect(self.conn_details.clone()).await;

            match client {
                Ok(client) => {
                    log::info!("Client reconnected successfully: {}", self.conn_details.url);
                    self.client.store(Arc::from(client));
                    break;
                }
                Err(e) => {
                    log::error!("Failed to reconnect: {:?}", e);
                }
            };

            Delay::new(Duration::from_secs(5)).await;
        }
        self.is_reconnecting.store(false, Ordering::SeqCst);
        Ok(())
    }
}

#[async_trait]
impl JsonRpcClient for ExtendedWs {
    type Error = WsClientError;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        if self.is_reconnecting.load(Ordering::Relaxed) {
            log::error!("Cannot process request, client is reconnecting: {method}");
            return Err(WsClientError::UnexpectedClose);
        }

        let provider = self.client.load().clone();
        let result = JsonRpcClient::request(&provider, method, params).await;

        // Attempt to reconnect Connection unexpectedly closed
        // TODO: execute this in a different thread/task, this will block the request
        match result {
            Err(WsClientError::UnexpectedClose) => {
                log::error!("Websocket closed unexpectedly, reconnecting...");
                self.reconnect().await?;
                Err(WsClientError::UnexpectedClose)
            }
            Err(err) => {
                // Log the error
                log::error!("{err:?}");
                Err(err)
            }
            _ => result,
        }
    }
}

impl PubsubClient for ExtendedWs {
    type NotificationStream = <Ws as PubsubClient>::NotificationStream;

    /// Add a subscription to this transport
    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, WsClientError> {
        if self.is_reconnecting.load(Ordering::Relaxed) {
            log::error!("subscription {} failed, client is reconnecting", id.into());
            return Err(WsClientError::UnexpectedClose);
        }

        let client = self.client.load().clone();
        PubsubClient::subscribe(client.as_ref(), id)
    }

    /// Remove a subscription from this transport
    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error> {
        if self.is_reconnecting.load(Ordering::Relaxed) {
            log::error!("unsubscribe {} failed, client is reconnecting", id.into());
            return Err(WsClientError::UnexpectedClose);
        }

        let client = self.client.load().clone();
        PubsubClient::unsubscribe(client.as_ref(), id)
    }
}

pub async fn is_connected(provider: &Ws) -> bool {
    let result = JsonRpcClient::request::<_, U64>(provider, "eth_blockNumber", ()).await;
    !matches!(
        result,
        Err(WsClientError::UnexpectedClose) | Err(WsClientError::InternalError(_))
    )
}
