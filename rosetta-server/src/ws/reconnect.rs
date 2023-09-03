use super::jsonrpsee_client::Params as RpcParams;
use async_trait::async_trait;
use jsonrpsee::core::client::BatchResponse;
use jsonrpsee::core::params::BatchRequestBuilder;
use jsonrpsee::core::{
    client::{ClientT, Subscription, SubscriptionClientT},
    traits::ToRpcParams,
    Error,
};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Reconnect trait.
/// This trait exposes callbacks which are called when the server returns a RestartNeeded error.
pub trait Reconnect<C>: 'static + Send + Sync
where
    C: ClientT + Send + Sync,
{
    type ClientRef: AsRef<C> + Send + Sync;
    type Error: Into<Error> + Send + Sync;

    type ReadyFuture<'a>: Future<Output = Result<Self::ClientRef, Self::Error>> + 'a + Send
    where
        Self: 'a;

    type RestartNeededFuture<'a>: Future<Output = Result<Self::ClientRef, Self::Error>> + 'a + Send
    where
        Self: 'a;

    type ReconnectFuture<'a>: Future<Output = Result<Self::ClientRef, Self::Error>> + 'a + Send
    where
        Self: 'a;

    /// Return a reference to the client.
    /// This method is called before every request.
    /// Here is the right place to block the requests until the client reconnects.
    fn ready(&self) -> Self::ReadyFuture<'_>;

    /// Callback called when the client returns a RestartNeeded error.
    /// # Params
    /// - `client` - The client which returned the RestartNeeded error.
    fn restart_needed(&self, client: Self::ClientRef) -> Self::RestartNeededFuture<'_>;

    /// Force reconnect and return a new client.
    fn reconnect(&self) -> Self::ReconnectFuture<'_>;
}

pub struct AutoReconnectClient<C, T> {
    _marker: PhantomData<C>,
    client: T,
}

impl<C, T> Deref for AutoReconnectClient<C, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<C, T> DerefMut for AutoReconnectClient<C, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

#[async_trait]
impl<C, T> ClientT for AutoReconnectClient<C, T>
where
    C: ClientT + Send + Sync,
    T: Reconnect<C>,
{
    async fn notification<Params>(&self, method: &str, params: Params) -> Result<(), Error>
    where
        Params: ToRpcParams + Send,
    {
        let client = Reconnect::ready(&self.client).await.map_err(Into::into)?;
        let params = RpcParams::new(params)?;
        match ClientT::notification(client.as_ref(), method, params.clone()).await {
            Ok(r) => Ok(r),
            Err(Error::RestartNeeded(_)) => {
                let client = Reconnect::restart_needed(&self.client, client)
                    .await
                    .map_err(Into::into)?;
                ClientT::notification(client.as_ref(), method, params).await
            }
            Err(error) => Err(error),
        }
    }

    async fn request<R, Params>(&self, method: &str, params: Params) -> Result<R, Error>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        let client = Reconnect::ready(&self.client).await.map_err(Into::into)?;
        let params = RpcParams::new(params)?;
        let error = match ClientT::request::<R, _>(client.as_ref(), method, params.clone()).await {
            Ok(r) => return Ok(r),
            Err(error) => error,
        };

        match error {
            Error::RestartNeeded(_) => {
                let client = Reconnect::restart_needed(&self.client, client)
                    .await
                    .map_err(Into::into)?;
                ClientT::request::<R, _>(client.as_ref(), method, params).await
            }
            error => Err(error),
        }
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, Error>
    where
        R: DeserializeOwned + Debug + 'a,
    {
        let client = Reconnect::ready(&self.client).await.map_err(Into::into)?;
        let error = match ClientT::batch_request(client.as_ref(), batch.clone()).await {
            Ok(r) => return Ok(r),
            Err(error) => error,
        };

        match error {
            Error::RestartNeeded(_) => {
                let client = Reconnect::restart_needed(&self.client, client)
                    .await
                    .map_err(Into::into)?;
                ClientT::batch_request(client.as_ref(), batch).await
            }
            error => Err(error),
        }
    }
}

#[async_trait]
impl<C, T> SubscriptionClientT for AutoReconnectClient<C, T>
where
    C: SubscriptionClientT + Send + Sync,
    T: Reconnect<C>,
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
        let client = Reconnect::ready(&self.client).await.map_err(Into::into)?;
        let params = RpcParams::new(params)?;
        let error = match SubscriptionClientT::subscribe::<Notif, _>(
            client.as_ref(),
            subscribe_method,
            params.clone(),
            unsubscribe_method,
        )
        .await
        {
            Ok(subscription) => return Ok(subscription),
            Err(error) => error,
        };

        match error {
            Error::RestartNeeded(_) => {
                let client = Reconnect::restart_needed(&self.client, client)
                    .await
                    .map_err(Into::into)?;
                SubscriptionClientT::subscribe::<Notif, _>(
                    client.as_ref(),
                    subscribe_method,
                    params,
                    unsubscribe_method,
                )
                .await
            }
            error => Err(error),
        }
    }

    async fn subscribe_to_method<'a, Notif>(
        &self,
        method: &'a str,
    ) -> Result<Subscription<Notif>, Error>
    where
        Notif: DeserializeOwned,
    {
        let client = Reconnect::ready(&self.client).await.map_err(Into::into)?;
        let error = match SubscriptionClientT::subscribe_to_method(client.as_ref(), method).await {
            Ok(subscription) => return Ok(subscription),
            Err(error) => error,
        };

        match error {
            Error::RestartNeeded(_) => {
                let client = Reconnect::restart_needed(&self.client, client)
                    .await
                    .map_err(Into::into)?;
                SubscriptionClientT::subscribe_to_method(client.as_ref(), method).await
            }
            error => Err(error),
        }
    }
}
