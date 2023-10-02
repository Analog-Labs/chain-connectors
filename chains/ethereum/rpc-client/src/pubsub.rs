use crate::client::EthClientAdapter;
use crate::prelude::ToRpcParams;
use crate::subscription::EthSubscription;
use crate::{
    error::EthError,
    extension::{impl_client_trait, impl_subscription_trait},
    params::EthRpcParams,
};
use async_trait::async_trait;
use dashmap::DashMap;
use ethers::providers::{JsonRpcClient, PubsubClient};
use ethers::types::U256;
use jsonrpsee::{
    core::{
        client::{ClientT, Subscription, SubscriptionClientT, SubscriptionKind},
        error::Error as JsonRpseeError,
    },
    types::SubscriptionId,
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

/// Adapter for [`jsonrpsee::core::client::SubscriptionClientT`] to [`ethers::providers::PubsubClient`].
pub struct EthPubsubAdapter<C> {
    pub(crate) adapter: EthClientAdapter<C>,
    pub(crate) eth_subscriptions: Arc<DashMap<U256, SubscriptionState>>,
}

impl<C> AsRef<C> for EthPubsubAdapter<C> {
    fn as_ref(&self) -> &C {
        &self.adapter.client
    }
}

impl_client_trait!(EthPubsubAdapter<C> where C: SubscriptionClientT + Debug + Send + Sync);
impl_subscription_trait!(EthPubsubAdapter<C> where C: SubscriptionClientT + Debug + Send + Sync);

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

    pub async fn eth_subscribe<R, P>(&self, params: P) -> Result<R, EthError>
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
        .map_err(EthError::from)?;

        // The ethereum subscription id must be an U256
        let maybe_id = match stream.kind() {
            SubscriptionKind::Subscription(id) => {
                tracing::trace!("subscription_id: {id:?}");
                let maybe_subscription_id = serde_json::to_value(id)
                    .ok()
                    .and_then(|value| serde_json::from_value::<U256>(value).ok());

                let id = if let Some(id) = maybe_subscription_id {
                    id
                } else {
                    // id is not a valid U256, convert str to bytes
                    match id {
                        SubscriptionId::Num(id) => U256::from(*id),
                        SubscriptionId::Str(id) => {
                            let str_bytes = id.as_bytes();
                            let mut bytes = [0u8; 32];
                            let size = usize::min(str_bytes.len(), bytes.len());
                            bytes[0..size].copy_from_slice(str_bytes);
                            U256::from_big_endian(bytes.as_slice())
                        }
                    }
                };
                Some(id)
            }
            _ => None,
        }
        .and_then(|subscription_id| {
            // For ethereum subscriptions, R is always U256.
            let result =
                serde_json::from_value::<R>(serde_json::to_value(subscription_id).ok()?).ok()?;
            Some((subscription_id, result))
        });

        // Unsubscribe in case of error
        let Some((subscription_id, result)) = maybe_id else {
            stream.unsubscribe().await?;
            return Err(EthError::JsonRpsee {
                original: JsonRpseeError::InvalidSubscriptionId,
                message: None,
            });
        };

        let _ = self
            .eth_subscriptions
            .insert(subscription_id, SubscriptionState::Pending(stream));
        Ok(result)
    }

    pub async fn eth_unsubscribe<R>(&self, params: EthRpcParams) -> Result<R, EthError>
    where
        R: DeserializeOwned + Send,
    {
        let subscription_id = params
            .deserialize_as::<U256>()
            .map_err(EthError::from)
            .map_err(|_| JsonRpseeError::InvalidSubscriptionId)?;

        let Some(mut state) = self.eth_subscriptions.get_mut(&subscription_id) else {
            return Err(EthError::JsonRpsee {
                original: JsonRpseeError::InvalidSubscriptionId,
                message: None,
            });
        };
        state.unsubscribe().await?;

        // For unsubscribe, R is always boolean
        serde_json::from_value::<R>(serde_json::value::Value::Bool(true)).map_err(EthError::from)
    }
}

#[async_trait]
impl<C> JsonRpcClient for EthPubsubAdapter<C>
where
    C: SubscriptionClientT + Debug + Send + Sync,
{
    type Error = EthError;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        let params = EthRpcParams::from_serializable(&params)?;
        match method {
            ETHEREUM_SUBSCRIBE_METHOD => self.eth_subscribe(params).await,
            ETHEREUM_UNSUBSCRIBE_METHOD => self.eth_unsubscribe(params).await,
            _ => ClientT::request(&self.adapter, method, params)
                .await
                .map_err(EthError::from),
        }
    }
}

impl<C> PubsubClient for EthPubsubAdapter<C>
where
    C: SubscriptionClientT + Debug + Send + Sync,
{
    type NotificationStream = EthSubscription;

    /// Add a subscription to this transport
    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, EthError> {
        let id = id.into();
        let Some(mut state) = self.eth_subscriptions.get_mut(&id) else {
            return Err(EthError::JsonRpsee {
                original: JsonRpseeError::InvalidSubscriptionId,
                message: None,
            });
        };

        state.subscribe(id).ok_or_else(|| EthError::JsonRpsee {
            original: JsonRpseeError::InvalidSubscriptionId,
            message: None,
        })
    }

    /// Remove a subscription from this transport
    fn unsubscribe<T: Into<U256>>(&self, _id: T) -> Result<(), Self::Error> {
        self.eth_subscriptions
            .remove(&_id.into())
            .map(|_| ())
            .ok_or_else(|| EthError::JsonRpsee {
                original: JsonRpseeError::InvalidSubscriptionId,
                message: None,
            })
    }
}

#[derive(Debug)]
pub(crate) enum SubscriptionState {
    Pending(Subscription<serde_json::Value>),
    Subscribed(Arc<AtomicBool>),
    Unsubscribed,
}

impl SubscriptionState {
    fn subscribe(&mut self, id: U256) -> Option<EthSubscription> {
        let old_state = std::mem::replace(self, SubscriptionState::Unsubscribed);
        match old_state {
            SubscriptionState::Pending(stream) => {
                let unsubscribe = Arc::new(AtomicBool::new(false));
                *self = SubscriptionState::Subscribed(unsubscribe.clone());
                Some(EthSubscription::new(id, stream, unsubscribe))
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
