use crate::client::ClientAdapter;
use crate::subscription::SubscriptionStream;
use crate::{error::Error, params::RpcParams};
use async_trait::async_trait;
use dashmap::DashMap;
use ethers::providers::{JsonRpcClient, PubsubClient};
use ethers::types::U256;
use jsonrpsee::core::{
    client::{Subscription, SubscriptionClientT, SubscriptionKind},
    error::Error as JsonRpseeError,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug, Formatter};

const ETHEREUM_SUBSCRIBE_METHOD: &str = "eth_subscribe";
const ETHEREUM_UNSUBSCRIBE_METHOD: &str = "eth_unsubscribe";

// Client that supports subscriptions
pub struct PubsubAdapter<C> {
    pub(crate) adapter: ClientAdapter<C>,
    pub(crate) subscriptions: DashMap<U256, Subscription<serde_json::Value>>,
}

impl<C> Debug for PubsubAdapter<C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PubsubAdapter")
            .field("adapter", &self.adapter)
            .field("subscriptions", &self.subscriptions.len())
            .finish()
    }
}

impl<C> PubsubAdapter<C>
where
    C: SubscriptionClientT + Debug + Send + Sync,
{
    pub fn new(client: C) -> Self {
        Self {
            adapter: ClientAdapter::new(client),
            subscriptions: DashMap::new(),
        }
    }

    pub async fn eth_subscribe<R>(&self, params: RpcParams) -> Result<R, Error>
    where
        R: DeserializeOwned + Send,
    {
        let stream = SubscriptionClientT::subscribe::<serde_json::Value, _>(
            self.adapter.as_ref(),
            ETHEREUM_SUBSCRIBE_METHOD,
            params,
            ETHEREUM_UNSUBSCRIBE_METHOD,
        )
        .await
        .map_err(Error::from)?;

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
impl<C> JsonRpcClient for PubsubAdapter<C>
where
    C: SubscriptionClientT + Debug + Send + Sync,
{
    type Error = Error;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        let params = RpcParams::from_serializable(&params)?;
        match method {
            ETHEREUM_SUBSCRIBE_METHOD => self.eth_subscribe(params).await,
            _ => self.adapter.eth_request(method, params).await,
        }
    }
}

impl<C> PubsubClient for PubsubAdapter<C>
where
    C: SubscriptionClientT + Debug + Send + Sync,
{
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

// pub async fn is_connected<C>(client: &C) -> bool
// where
//     C: ClientT,
// {
//     // Do a simple request to check if the RPC node is functional
//     let result = client
//         .request::<U64, _>("eth_blockNumber", rpc_params![])
//         .await;
//     !matches!(
//         result,
//         Err(JsonRpseeError::RestartNeeded(_)
//             | JsonRpseeError::AlreadyStopped
//             | JsonRpseeError::Transport(_))
//     )
// }
