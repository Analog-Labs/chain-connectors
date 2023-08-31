use async_trait::async_trait;
use jsonrpsee::core::client::{BatchResponse, Subscription, SubscriptionClientT};
use jsonrpsee::core::params::BatchRequestBuilder;
use jsonrpsee::core::{client::ClientT, traits::ToRpcParams, Error};
use serde::de::DeserializeOwned;
use std::fmt::{Debug, Formatter};

/// Extension helper for `ClientT` and `SubscriptionClientT`
pub struct Extended<C, T> {
    pub(crate) client: C,
    pub(crate) data: T,
}

#[allow(dead_code)]
impl<C, T> Extended<C, T>
where
    C: Send + Sync,
    T: Send + Sync,
{
    pub fn new(client: C, data: T) -> Self {
        Self { client, data }
    }

    pub fn client(&self) -> &C {
        &self.client
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn into_inner(self) -> (C, T) {
        (self.client, self.data)
    }
}

impl<C, T> Clone for Extended<C, T>
where
    C: Clone + Send + Sync,
    T: Clone + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            data: self.data.clone(),
        }
    }
}

impl<C, T> Debug for Extended<C, T>
where
    C: Debug + Send + Sync,
    T: Debug + Send + Sync,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extended")
            .field("client", &self.client)
            .field("data", &self.data)
            .finish()
    }
}

impl<C, T> AsRef<C> for Extended<C, T>
where
    C: Send + Sync,
    T: Send + Sync,
{
    fn as_ref(&self) -> &C {
        &self.client
    }
}

impl<C, T> AsMut<C> for Extended<C, T>
where
    C: Send + Sync,
    T: Send + Sync,
{
    fn as_mut(&mut self) -> &mut C {
        &mut self.client
    }
}

#[async_trait]
impl<C, T> ClientT for Extended<C, T>
where
    C: ClientT + Send + Sync,
    T: Send + Sync,
{
    async fn notification<Params>(&self, method: &str, params: Params) -> Result<(), Error>
    where
        Params: ToRpcParams + Send,
    {
        ClientT::notification::<Params>(&self.client, method, params).await
    }

    async fn request<R, Params>(&self, method: &str, params: Params) -> Result<R, Error>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        ClientT::request::<R, Params>(&self.client, method, params).await
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, Error>
    where
        R: DeserializeOwned + Debug + 'a,
    {
        ClientT::batch_request::<R>(&self.client, batch).await
    }
}

#[async_trait]
impl<C, T> SubscriptionClientT for Extended<C, T>
where
    C: SubscriptionClientT + Send + Sync,
    T: Send + Sync,
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
            &self.client,
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
        SubscriptionClientT::subscribe_to_method(&self.client, method).await
    }
}
