use super::{auto_reconnect::Reconnect, extension::Extended};
use arc_swap::ArcSwapOption;
use futures_util::future::BoxFuture;
use futures_util::future::Shared;
use futures_util::FutureExt;
use jsonrpsee::core::{client::SubscriptionClientT, error::Error};
use pin_project::pin_project;
use std::pin::Pin;
use std::sync::atomic::AtomicU32;
use std::sync::RwLock;
use std::task::{Context, Poll};
use std::{future::Future, sync::Arc};

pub type ClientWithIdentifier<C> = Extended<C, u32>;

#[pin_project(project = ClientReadyFutureProj)]
pub enum ClientReadyFuture<C> {
    Ready(Result<Arc<C>, Error>),
    Reconnecting(BoxFuture<'static, Result<Arc<C>, Error>>),
}

impl<C> Future for ClientReadyFuture<C> {
    type Output = Result<Arc<C>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this {
            ClientReadyFutureProj::Ready(result) => Poll::Ready(result.clone()),
            ClientReadyFutureProj::Reconnecting(mut reconnect_attempt) => {
                return reconnect_attempt.poll_unpin(cx);
            }
        }
    }
}

#[pin_project]
#[derive(Debug, Clone)]
pub struct ReconnectAttempt<C> {
    pub attempt: u32,
    pub future: Shared<BoxFuture<'static, Result<Option<Arc<ClientWithIdentifier<C>>>, Error>>>,
}

impl<C> ReconnectAttempt<C> {
    pub fn new(
        attempt: u32,
        future: BoxFuture<'static, Result<Arc<ClientWithIdentifier<C>>, Error>>,
    ) -> Self {
        Self {
            attempt,
            future: future.shared(),
        }
    }
}

impl<C, F> Future for ReconnectAttempt<F>
where
    F: Future<Output = Result<C, Error>> + Send + Sync,
{
    type Output = Result<Option<Arc<ClientWithIdentifier<C>>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        Future::poll(Pin::new(&mut this.future), cx)
    }
}

pub struct DefaultStrategy<C, Fut, B> {
    builder: B,
    reconnects_count: AtomicU32,
    is_reconnecting: RwLock<Option<ReconnectAttempt<Fut>>>,
    client: ArcSwapOption<ClientWithIdentifier<C>>,
}

impl<C, Fut, B> DefaultStrategy<C, Fut, B>
where
    C: SubscriptionClientT + Send + Sync,
    Fut: Future<Output = Result<C, Error>> + Send + Sync,
    B: FnOnce() -> Fut + Send + Sync + Clone,
{
    pub async fn connect(builder: B) -> Result<Self, Error> {
        let client = (builder.clone())().await?;
        let client = ClientWithIdentifier::new(client, 0);
        Ok(Self {
            builder,
            reconnects_count: AtomicU32::new(0),
            is_reconnecting: RwLock::new(None),
            client: ArcSwapOption::from(Some(Arc::new(client))),
        })
    }

    /// Check if the client is reconnecting.
    ///
    /// Returns true if the client was reconnecting, false otherwise
    pub fn acquire_client(&self) -> ClientReadyFuture<C> {
        // let is_reconnecting = {
        //     // Check if the client is reconnecting
        //     let guard = self.is_reconnecting.read().map_err(|_| Error::Custom("Fatal error: client lock was poisoned".to_string()))?;
        //     guard.as_ref().map(|reconnect| reconnect.future.clone())
        // };

        let result = self
            .client
            .load()
            .clone()
            .ok_or_else(|| Error::Custom("Client is reconnecting...".to_string()));
        ClientReadyFuture::Ready(result)
    }
}

impl<C, Fut, B> Reconnect<C> for DefaultStrategy<C, Fut, B>
where
    C: SubscriptionClientT + Send + Sync,
    Fut: Future<Output = Result<C, Error>> + Send + Sync,
    B: FnOnce() -> Fut + Send + Sync + Clone,
{
    type ClientRef = Arc<ClientWithIdentifier<C>>;
    type ReadyFuture<'a> = ClientReadyFuture<Self::ClientRef> where Self: 'a;
    type ReconnectFuture<'a> = ReconnectAttempt<Self::ClientRef> where Self: 'a;

    fn client(&self) -> Self::ReadyFuture<'_> {
        self.acquire_client()
    }

    fn reconnect(&self) -> Self::ReconnectFuture<'_> {
        // Load the current reconnect attempt
        let current_attempt = self
            .reconnects_count
            .load(std::sync::atomic::Ordering::SeqCst)
            + 1;

        // Make sure only one thread is handling the reconnect
        let guard = self
            .is_reconnecting
            .write()
            .map_err(|_| Error::Custom("Fatal error: client lock was poisoned".to_string()))?;

        let future = (self.builder.clone())().map(move |value| {
            value.map(|client| {
                let client = ClientWithIdentifier::new(client, current_attempt);
                Some(Arc::new(client))
            })
        });

        ReconnectAttempt::new(current_attempt, Box::pin(future))
    }
}
