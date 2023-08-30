use crate::client::EthClientAdapter;
use crate::prelude::ToRpcParams;
use crate::subscription::SubscriptionStream;
use crate::{error::Error, params::RpcParams};
use async_trait::async_trait;
use dashmap::DashMap;
use ethers::providers::{JsonRpcClient, PubsubClient};
use ethers::types::U256;
use jsonrpsee::core::client::BatchResponse;
use jsonrpsee::core::params::BatchRequestBuilder;
use jsonrpsee::core::{
    client::{ClientT, Subscription, SubscriptionClientT, SubscriptionKind},
    error::Error as JsonRpseeError,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

const ETHEREUM_SUBSCRIBE_METHOD: &str = "eth_subscribe";
const ETHEREUM_UNSUBSCRIBE_METHOD: &str = "eth_unsubscribe";

#[derive(Debug)]
pub(crate) enum SubscriptionState {
    Pending(Subscription<serde_json::Value>),
    Subscribed(Arc<AtomicBool>),
    Unsubscribed,
}

impl SubscriptionState {
    fn subscribe(&mut self, id: U256) -> Option<SubscriptionStream> {
        let old_state = std::mem::replace(self, SubscriptionState::Unsubscribed);
        match old_state {
            SubscriptionState::Pending(stream) => {
                let unsubscribe = Arc::new(AtomicBool::new(false));
                *self = SubscriptionState::Subscribed(unsubscribe.clone());
                Some(SubscriptionStream::new(id, stream, unsubscribe))
            }
            SubscriptionState::Subscribed(unsubscribe) => {
                *self = SubscriptionState::Subscribed(unsubscribe);
                None
            }
            SubscriptionState::Unsubscribed => None,
        }
    }

    async fn unsubscribe(&mut self) -> Result<(), JsonRpseeError> {
        let old_state = std::mem::replace(self, SubscriptionState::Unsubscribed);
        match old_state {
            SubscriptionState::Pending(stream) => stream.unsubscribe().await,
            SubscriptionState::Subscribed(unsubscribe) => {
                unsubscribe.store(true, Ordering::SeqCst);
                Ok(())
            }
            SubscriptionState::Unsubscribed => Ok(()),
        }
    }
}

/// Client adapter that supports subscriptions
pub struct EthPubsubAdapter<C> {
    pub(crate) adapter: EthClientAdapter<C>,
    pub(crate) eth_subscriptions: Arc<DashMap<U256, SubscriptionState>>,
}

impl<C> Debug for EthPubsubAdapter<C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PubsubAdapter")
            .field("adapter", &self.adapter)
            .field("subscriptions", &self.eth_subscriptions.len())
            .finish()
    }
}

impl<C> Clone for EthPubsubAdapter<C>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        Self {
            adapter: self.adapter.clone(),
            eth_subscriptions: self.eth_subscriptions.clone(),
        }
    }
}

impl<C> Deref for EthPubsubAdapter<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.adapter.deref()
    }
}

impl<C> DerefMut for EthPubsubAdapter<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.adapter.deref_mut()
    }
}

impl<C> EthPubsubAdapter<C>
where
    C: SubscriptionClientT + Debug + Send + Sync,
{
    pub fn new(client: C) -> Self {
        Self {
            adapter: EthClientAdapter::new(client),
            eth_subscriptions: Arc::new(DashMap::new()),
        }
    }

    pub async fn eth_subscribe<R, P>(&self, params: P) -> Result<R, Error>
    where
        R: DeserializeOwned + Send,
        P: ToRpcParams + Send,
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

        let _ = self
            .eth_subscriptions
            .insert(subscription_id, SubscriptionState::Pending(stream));
        Ok(result)
    }

    pub async fn eth_unsubscribe<R>(&self, params: RpcParams) -> Result<R, Error>
    where
        R: DeserializeOwned + Send,
    {
        let subscription_id = params
            .deserialize_as::<U256>()
            .map_err(Error::from)
            .map_err(|_| JsonRpseeError::InvalidSubscriptionId)?;

        let Some(mut state) = self.eth_subscriptions.get_mut(&subscription_id) else {
            return Err(Error::JsonRpsee {
                original: JsonRpseeError::InvalidSubscriptionId,
                message: None,
            });
        };
        state.unsubscribe().await?;

        serde_json::from_value::<R>(serde_json::value::Value::Bool(true)).map_err(Error::from)
    }
}

#[async_trait]
impl<C> JsonRpcClient for EthPubsubAdapter<C>
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
            ETHEREUM_UNSUBSCRIBE_METHOD => self.eth_unsubscribe(params).await,
            _ => ClientT::request(self, method, params)
                .await
                .map_err(Error::from),
        }
    }
}

impl<C> PubsubClient for EthPubsubAdapter<C>
where
    C: SubscriptionClientT + Debug + Send + Sync,
{
    type NotificationStream = SubscriptionStream;

    /// Add a subscription to this transport
    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Error> {
        let id = id.into();
        let Some(mut state) = self.eth_subscriptions.get_mut(&id) else {
            return Err(Error::JsonRpsee {
                original: JsonRpseeError::InvalidSubscriptionId,
                message: None,
            });
        };

        state.subscribe(id).ok_or_else(|| Error::JsonRpsee {
            original: JsonRpseeError::InvalidSubscriptionId,
            message: None,
        })
    }

    /// Remove a subscription from this transport
    fn unsubscribe<T: Into<U256>>(&self, _id: T) -> Result<(), Self::Error> {
        self.eth_subscriptions
            .remove(&_id.into())
            .map(|_| ())
            .ok_or_else(|| Error::JsonRpsee {
                original: JsonRpseeError::InvalidSubscriptionId,
                message: None,
            })
    }
}

#[async_trait]
impl<C> ClientT for EthPubsubAdapter<C>
where
    C: SubscriptionClientT + Debug + Send + Sync,
{
    async fn notification<Params>(&self, method: &str, params: Params) -> Result<(), JsonRpseeError>
    where
        Params: ToRpcParams + Send,
    {
        ClientT::notification(self, method, params).await
    }

    async fn request<R, Params>(&self, method: &str, params: Params) -> Result<R, JsonRpseeError>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        ClientT::request(self, method, params).await
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, JsonRpseeError>
    where
        R: DeserializeOwned + Debug + 'a,
    {
        ClientT::batch_request(self, batch).await
    }
}

#[async_trait]
impl<C> SubscriptionClientT for EthPubsubAdapter<C>
where
    C: SubscriptionClientT + Debug + Send + Sync,
{
    async fn subscribe<'a, Notif, Params>(
        &self,
        subscribe_method: &'a str,
        params: Params,
        unsubscribe_method: &'a str,
    ) -> Result<Subscription<Notif>, JsonRpseeError>
    where
        Params: ToRpcParams + Send,
        Notif: DeserializeOwned,
    {
        SubscriptionClientT::subscribe(self, subscribe_method, params, unsubscribe_method).await
    }

    async fn subscribe_to_method<'a, Notif>(
        &self,
        method: &'a str,
    ) -> Result<Subscription<Notif>, JsonRpseeError>
    where
        Notif: DeserializeOwned,
    {
        SubscriptionClientT::subscribe_to_method(self, method).await
    }
}
