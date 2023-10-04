use super::jsonrpsee_client::Params as RpcParams;
use async_trait::async_trait;
use jsonrpsee::core::{
    client::{BatchResponse, ClientT, Subscription, SubscriptionClientT},
    params::BatchRequestBuilder,
    traits::ToRpcParams,
    Error,
};
use serde::de::DeserializeOwned;
use std::{
    fmt::{Debug, Display, Formatter},
    future::Future,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
};

/// Reconnect trait.
/// This trait exposes callbacks which are called when the server returns a [`Error::RestartNeeded`]
/// error.
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

    /// Callback called when the client returns a [`Error::RestartNeeded`] error.
    /// # Params
    /// - `client` - The client which returned the [`Error::RestartNeeded`] error.
    fn restart_needed(&self, client: Self::ClientRef) -> Self::RestartNeededFuture<'_>;

    /// Force reconnect and return a new client.
    fn reconnect(&self) -> Self::ReconnectFuture<'_>;

    /// Return a reference to the client.
    fn into_client(self) -> AutoReconnectClient<Self> {
        AutoReconnectClient::new(self)
    }
}

pub struct AutoReconnectClient<T> {
    client: T,
    reconnect_count: AtomicU32,
    span: tracing::Span,
}

impl<T> AutoReconnectClient<T>
where
    T: Reconnect,
    T::Client: ClientT,
{
    pub fn new(client: T) -> Self {
        Self {
            client,
            reconnect_count: AtomicU32::new(0),
            span: tracing::info_span!("rpc_client", reconnects = 0),
        }
    }

    async fn ready(&self) -> Result<T::ClientRef, Error> {
        Reconnect::ready(&self.client).await.map_err(|error| {
            tracing::error!("rpc client is unavailable: {error:?}");
            error
        })
    }

    async fn restart_needed(
        &self,
        reason: String,
        client: T::ClientRef,
    ) -> Result<T::ClientRef, Error> {
        let reconnect_count = self.reconnect_count.fetch_add(1, Ordering::SeqCst) + 1;
        self.span.record("reconnects", reconnect_count);
        tracing::error!("Reconneting RPC client due error: {reason}");
        Reconnect::restart_needed(&self.client, client).await.map_err(|error| {
            tracing::error!("rpc client is unavailable: {error:?}");
            error
        })
    }
}

impl<T> Clone for AutoReconnectClient<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let reconnects = self.reconnect_count.load(Ordering::SeqCst);
        Self {
            client: self.client.clone(),
            reconnect_count: AtomicU32::new(reconnects),
            span: tracing::info_span!("rpc_client", reconnects = reconnects),
        }
    }
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

impl<T> Debug for AutoReconnectClient<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoReconnectClient")
            .field("client", &self.client)
            .finish_non_exhaustive()
    }
}

impl<T> Display for AutoReconnectClient<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.client, f)
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
        let _enter = self.span.enter();
        let client = self.ready().await?;
        let params = RpcParams::new(params)?;
        match ClientT::notification(client.as_ref(), method, params.clone()).await {
            Ok(r) => Ok(r),
            Err(Error::RestartNeeded(message)) => {
                let client = self.restart_needed(message, client).await?;
                ClientT::notification(client.as_ref(), method, params).await
            },
            Err(error) => {
                tracing::error!("notification '{method}' failed: {error:?}");
                Err(error)
            },
        }
    }

    async fn request<R, Params>(&self, method: &str, params: Params) -> Result<R, Error>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        let _enter = self.span.enter();
        let client = self.ready().await?;
        let params = RpcParams::new(params)?;
        let error = match ClientT::request::<R, _>(client.as_ref(), method, params.clone()).await {
            Ok(r) => return Ok(r),
            Err(error) => error,
        };

        match error {
            Error::RestartNeeded(message) => {
                let client = self.restart_needed(message, client).await?;
                ClientT::request::<R, _>(client.as_ref(), method, params).await
            },
            error => {
                tracing::error!("rpc request '{method}' failed: {error:?}");
                Err(error)
            },
        }
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, Error>
    where
        R: DeserializeOwned + Debug + 'a,
    {
        let _enter = self.span.enter();
        let client = self.ready().await?;
        let error = match ClientT::batch_request(client.as_ref(), batch.clone()).await {
            Ok(r) => return Ok(r),
            Err(error) => error,
        };

        match error {
            Error::RestartNeeded(message) => {
                let client = self.restart_needed(message, client).await?;
                ClientT::batch_request(client.as_ref(), batch).await
            },
            error => {
                tracing::error!("batch request failed: {error:?}");
                Err(error)
            },
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
        let _enter = self.span.enter();
        let client = self.ready().await?;
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
            Error::RestartNeeded(message) => {
                let client = self.restart_needed(message, client).await?;
                SubscriptionClientT::subscribe::<Notif, _>(
                    client.as_ref(),
                    subscribe_method,
                    params,
                    unsubscribe_method,
                )
                .await
            },
            error => {
                tracing::error!("subscription to '{subscribe_method}' failed: {error:?}");
                Err(error)
            },
        }
    }

    async fn subscribe_to_method<'a, Notif>(
        &self,
        method: &'a str,
    ) -> Result<Subscription<Notif>, Error>
    where
        Notif: DeserializeOwned,
    {
        let _enter = self.span.enter();
        let client = self.ready().await?;
        let error = match SubscriptionClientT::subscribe_to_method(client.as_ref(), method).await {
            Ok(subscription) => return Ok(subscription),
            Err(error) => error,
        };

        match error {
            Error::RestartNeeded(message) => {
                let client = self.restart_needed(message, client).await?;
                SubscriptionClientT::subscribe_to_method(client.as_ref(), method).await
            },
            error => {
                tracing::error!("subscription to '{method}' failed: {error:?}");
                Err(error)
            },
        }
    }
}
