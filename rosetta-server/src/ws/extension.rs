use async_trait::async_trait;
use jsonrpsee::core::client::{BatchResponse, Subscription, SubscriptionClientT};
use jsonrpsee::core::params::BatchRequestBuilder;
use jsonrpsee::core::{client::ClientT, traits::ToRpcParams, Error};
use serde::de::DeserializeOwned;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

/// Extension helper for `ClientT` and `SubscriptionClientT`
pub struct Extended<C, T> {
    _marker: PhantomData<C>,
    pub(crate) state: T,
}

#[allow(dead_code)]
impl<C, T> Extended<C, T>
where
    C: Send + Sync,
    T: AsRef<C> + Send + Sync,
{
    pub fn new(state: T) -> Self {
        Self {
            _marker: PhantomData,
            state,
        }
    }

    pub fn client(&self) -> &C {
        self.state.as_ref()
    }

    pub fn state(&self) -> &T {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut T {
        &mut self.state
    }

    pub fn into_inner(self) -> T {
        self.state
    }
}

impl<C, T> Clone for Extended<C, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            _marker: self._marker,
            state: self.state.clone(),
        }
    }
}

impl<C, T> Debug for Extended<C, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extended")
            .field("_marker", &self._marker)
            .field("state", &self.state)
            .finish()
    }
}

impl<C, T> AsRef<C> for Extended<C, T>
where
    T: AsRef<C>,
{
    fn as_ref(&self) -> &C {
        self.state.as_ref()
    }
}

impl<C, T> AsMut<C> for Extended<C, T>
where
    T: AsMut<C>,
{
    fn as_mut(&mut self) -> &mut C {
        self.state.as_mut()
    }
}

#[async_trait]
impl<C, T> ClientT for Extended<C, T>
where
    C: ClientT + Send + Sync,
    T: AsRef<C> + Send + Sync,
{
    async fn notification<Params>(&self, method: &str, params: Params) -> Result<(), Error>
    where
        Params: ToRpcParams + Send,
    {
        ClientT::notification::<Params>(self.client(), method, params).await
    }

    async fn request<R, Params>(&self, method: &str, params: Params) -> Result<R, Error>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        ClientT::request::<R, Params>(self.client(), method, params).await
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, Error>
    where
        R: DeserializeOwned + Debug + 'a,
    {
        ClientT::batch_request::<R>(self.client(), batch).await
    }
}

#[async_trait]
impl<C, T> SubscriptionClientT for Extended<C, T>
where
    C: SubscriptionClientT + Send + Sync,
    T: AsRef<C> + Send + Sync,
{
    async fn subscribe<'a, Notif, Params>(
        &self,
        subscribe_method: &'a str,
        params: Params,
        unsubscribe_method: &'a str,
    ) -> Result<Subscription<Notif>, Error>
    where
        Params: ToRpcParams + Send,
        Notif: DeserializeOwned,
    {
        SubscriptionClientT::subscribe::<Notif, Params>(
            self.client(),
            subscribe_method,
            params,
            unsubscribe_method,
        )
        .await
    }

    async fn subscribe_to_method<'a, Notif>(
        &self,
        method: &'a str,
    ) -> Result<Subscription<Notif>, Error>
    where
        Notif: DeserializeOwned,
    {
        SubscriptionClientT::subscribe_to_method(self.client(), method).await
    }
}
