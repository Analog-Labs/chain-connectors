#![allow(dead_code)]
use super::{auto_reconnect::Reconnect, error::CloneableError, extension::Extended};
use arc_swap::ArcSwapOption;
use futures_util::future::BoxFuture;
use futures_util::future::Shared;
use futures_util::FutureExt;
use jsonrpsee::core::{client::SubscriptionClientT, error::Error};
use pin_project::pin_project;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;
use std::task::{Context, Poll};
use std::{future::Future, sync::Arc};

pub type Client<C> = Extended<C, u32>;
pub type ClientRef<C> = Arc<Client<C>>;

pub enum ClientReadyState<C, Fut> {
    Ready(Result<ClientRef<C>, Error>),
    Reconnecting(Fut),
}

#[pin_project]
pub struct ClientReadyFuture<C, Fut> {
    state: Option<ClientReadyState<C, Fut>>,
}

impl<C, Fut> ClientReadyFuture<C, Fut>
where
    Fut: Future<Output = Result<ClientRef<C>, Error>> + Send,
{
    pub fn ready(result: Result<ClientRef<C>, Error>) -> Self {
        Self {
            state: Some(ClientReadyState::Ready(result)),
        }
    }

    pub fn reconnecting(future: Fut) -> Self {
        Self {
            state: Some(ClientReadyState::Reconnecting(future)),
        }
    }
}

impl<Fut, C> From<Error> for ClientReadyFuture<C, Fut>
where
    Fut: Future<Output = Result<ClientRef<C>, Error>> + Send + Unpin,
{
    fn from(error: Error) -> Self {
        Self::ready(Err(error))
    }
}

impl<C, Fut> Future for ClientReadyFuture<C, Fut>
where
    Fut: Future<Output = Result<ClientRef<C>, Error>> + Send + Unpin,
{
    type Output = Result<ClientRef<C>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.state.take() {
            Some(ClientReadyState::Ready(result)) => Poll::Ready(result),
            Some(ClientReadyState::Reconnecting(mut future)) => {
                let result = future.poll_unpin(cx);
                *this.state = Some(ClientReadyState::Reconnecting(future));
                result
            }
            None => panic!("ClientReadyFuture polled after completion"),
        }
    }
}

#[pin_project]
pub struct ReconnectAttempt<C> {
    pub attempt: u32,
    pub future: Shared<BoxFuture<'static, Result<ClientRef<C>, CloneableError>>>,
}

impl<C> Debug for ReconnectAttempt<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReconnectAttempt")
            .field("attempt", &self.attempt)
            .field("future", &self.future)
            .finish()
    }
}

impl<C> Clone for ReconnectAttempt<C> {
    fn clone(&self) -> Self {
        Self {
            attempt: self.attempt,
            future: self.future.clone(),
        }
    }
}

impl<C> ReconnectAttempt<C> {
    pub fn new(
        attempt: u32,
        future: BoxFuture<'static, Result<ClientRef<C>, CloneableError>>,
    ) -> Self {
        Self {
            attempt,
            future: future.shared(),
        }
    }

    pub fn failure(error: Error) -> Self {
        Self {
            attempt: 0,
            future: async move {
                Result::<ClientRef<C>, CloneableError>::Err(CloneableError::from(error))
            }
            .boxed()
            .shared(),
        }
    }
}

impl<C> Future for ReconnectAttempt<C> {
    type Output = Result<ClientRef<C>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        match Future::poll(Pin::new(&mut this.future), cx) {
            Poll::Ready(result) => {
                let result = result.map_err(|error| error.clone().into_inner());
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct DefaultStrategy<C, B> {
    builder: B,
    reconnects_count: AtomicU32,
    is_reconnecting: RwLock<Option<ReconnectAttempt<C>>>,
    client: ArcSwapOption<Client<C>>,
}

impl<C, Fut, B> DefaultStrategy<C, B>
where
    C: SubscriptionClientT + Send + Sync,
    Fut: Future<Output = Result<C, Error>> + Send + Sync + 'static,
    B: FnOnce() -> Fut + Send + Sync + Clone,
{
    pub async fn connect(builder: B) -> Result<Self, Error> {
        let client = (builder.clone())().await?;
        let client = Client::new(client, 0);
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
    pub fn acquire_client(&self) -> ClientReadyFuture<C, ReconnectAttempt<C>> {
        {
            // Check if the client is reconnecting
            let guard = match self.is_reconnecting.read() {
                Ok(guard) => guard,
                Err(error) => {
                    return ClientReadyFuture::from(Error::Custom(format!(
                        "Fatal error, client lock was poisoned: {error}"
                    )))
                }
            };

            // If the client is reconnecting, wait for the reconnect to finish
            if let Some(reconnect) = guard.as_ref() {
                return ClientReadyFuture::reconnecting(reconnect.clone());
            }
        };

        let result = self
            .client
            .load()
            .clone()
            .ok_or_else(|| Error::Custom("Client is reconnecting...".to_string()));
        ClientReadyFuture::ready(result)
    }

    pub fn reconnect_attempt(&self, attempt: u32) -> ReconnectAttempt<C> {
        // Make sure only one thread is handling the reconnect
        let mut guard = match self.is_reconnecting.write() {
            Ok(guard) => guard,
            Err(error) => {
                return ReconnectAttempt::failure(Error::Custom(format!(
                    "Fatal error, client lock was poisoned: {error}"
                )))
            }
        };

        // If the client is already reconnecting, return the current attempt
        if let Some(reconnect_attempt) = guard.as_ref() {
            return reconnect_attempt.clone();
        }

        // Create a new reconnect attempt
        let reconnect_future = {
            let attempt = attempt;
            let future = (self.builder.clone())().map(move |value| {
                value
                    .map(|client| {
                        let client = Client::new(client, attempt);
                        Arc::new(client)
                    })
                    .map_err(CloneableError::from)
            });
            ReconnectAttempt::new(attempt, Box::pin(future))
        };

        // Store the reconnect attempt
        guard.replace(reconnect_future.clone());

        reconnect_future
    }
}

impl<C, Fut, B> Reconnect<Client<C>> for DefaultStrategy<C, B>
where
    C: SubscriptionClientT + Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Send + Sync + 'static,
    B: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    type ClientRef = ClientRef<C>;
    type ReadyFuture<'a> = ClientReadyFuture<C, ReconnectAttempt<C>> where Self: 'a;
    type ReconnectFuture<'a> = ReconnectAttempt<C> where Self: 'a;

    fn client(&self) -> Self::ReadyFuture<'_> {
        self.acquire_client()
    }

    fn restart_needed(&self, client: ClientRef<C>) -> Self::ReconnectFuture<'_> {
        // The current reconnect attempt
        let current_attempt = client.data + 1;

        self.reconnect_attempt(current_attempt)
    }

    fn reconnect(&self) -> Self::ReconnectFuture<'_> {
        // The current reconnect attempt
        let reconnect_attempt = match self.client.load().clone() {
            Some(client) => client.data + 1,
            None => self.reconnects_count.load(Ordering::SeqCst) + 1,
        };

        self.reconnect_attempt(reconnect_attempt)
    }
}
