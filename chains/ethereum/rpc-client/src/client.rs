use crate::{error::Error, params::RpcParams};
use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use dashmap::DashMap;
use ethers::prelude::*;
use ethers::providers::JsonRpcClient;
use ethers::types::U256;
use futures_timer::Delay;
use jsonrpsee::core::client::{Subscription, SubscriptionClientT, SubscriptionKind};
use jsonrpsee::{
    client_transport::ws::WsTransportClientBuilder,
    core::{
        client::{Client as JsonRpseeClient, ClientBuilder, ClientT},
        error::Error as JsonRpseeError,
    },
    rpc_params,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

const ETHEREUM_SUBSCRIBE_METHOD: &str = "eth_subscribe";
const ETHEREUM_UNSUBSCRIBE_METHOD: &str = "eth_unsubscribe";

// Websocket Client that supports reconnecting
#[derive(Debug)]
pub struct EthClient {
    pub(crate) uri: Url,
    pub(crate) client: ArcSwapOption<JsonRpseeClient>,
    pub(crate) subscriptions: DashMap<U256, Subscription<serde_json::Value>>,
}

impl EthClient {
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

    pub fn client(&self) -> Result<Arc<JsonRpseeClient>, Error> {
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
impl JsonRpcClient for EthClient {
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

pub async fn is_connected(client: &JsonRpseeClient) -> bool {
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
