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
use std::ops::{Deref, DerefMut};

/// Reconnect trait.
/// This trait exposes callbacks which are called when the server returns a RestartNeeded error.
pub trait Reconnect: 'static + Sized + Send + Sync {
    type Client: SubscriptionClientT + 'static + Send + Sync;
    type ClientRef: AsRef<Self::Client> + Send + Sync;

    type ReadyFuture<'a>: Future<Output = Result<Self::ClientRef, Error>> + 'a + Send + Unpin
    where
        Self: 'a;

    type RestartNeededFuture<'a>: Future<Output = Result<Self::ClientRef, Error>>
        + 'a
        + Send
        + Unpin
    where
        Self: 'a;

    type ReconnectFuture<'a>: Future<Output = Result<Self::ClientRef, Error>> + 'a + Send + Unpin
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

    /// Return a reference to the client.
    fn into_client(self) -> AutoReconnectClient<Self> {
        AutoReconnectClient { client: self }
    }
}

pub struct AutoReconnectClient<T> {
    client: T,
}

impl<T> Deref for AutoReconnectClient<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<T> DerefMut for AutoReconnectClient<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

#[async_trait]
impl<T> ClientT for AutoReconnectClient<T>
where
    T: Reconnect,
    T::Client: ClientT,
{
    async fn notification<Params>(&self, method: &str, params: Params) -> Result<(), Error>
    where
        Params: ToRpcParams + Send,
    {
        let client = Reconnect::ready(&self.client).await?;
        let params = RpcParams::new(params)?;
        match ClientT::notification(client.as_ref(), method, params.clone()).await {
            Ok(r) => Ok(r),
            Err(Error::RestartNeeded(_)) => {
                let client = Reconnect::restart_needed(&self.client, client).await?;
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
        let client = Reconnect::ready(&self.client).await?;
        let params = RpcParams::new(params)?;
        let error = match ClientT::request::<R, _>(client.as_ref(), method, params.clone()).await {
            Ok(r) => return Ok(r),
            Err(error) => error,
        };

        match error {
            Error::RestartNeeded(_) => {
                let client = Reconnect::restart_needed(&self.client, client).await?;
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
        let client = Reconnect::ready(&self.client).await?;
        let error = match ClientT::batch_request(client.as_ref(), batch.clone()).await {
            Ok(r) => return Ok(r),
            Err(error) => error,
        };

        match error {
            Error::RestartNeeded(_) => {
                let client = Reconnect::restart_needed(&self.client, client).await?;
                ClientT::batch_request(client.as_ref(), batch).await
            }
            error => Err(error),
        }
    }
}

#[async_trait]
impl<T> SubscriptionClientT for AutoReconnectClient<T>
where
    T: Reconnect,
    T::Client: SubscriptionClientT,
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
        let client = Reconnect::ready(&self.client).await?;
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
                let client = Reconnect::restart_needed(&self.client, client).await?;
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
        let client = Reconnect::ready(&self.client).await?;
        let error = match SubscriptionClientT::subscribe_to_method(client.as_ref(), method).await {
            Ok(subscription) => return Ok(subscription),
            Err(error) => error,
        };

        match error {
            Error::RestartNeeded(_) => {
                let client = Reconnect::restart_needed(&self.client, client).await?;
                SubscriptionClientT::subscribe_to_method(client.as_ref(), method).await
            }
            error => Err(error),
        }
    }
}
