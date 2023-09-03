#![allow(dead_code)]
use super::{error::CloneableError, reconnect::Reconnect};
use futures_timer::Delay;
use futures_util::future::{Either, Select};
use futures_util::{future::Shared, FutureExt};
use jsonrpsee::core::{client::SubscriptionClientT, error::Error};
use pin_project::pin_project;
use std::time::Duration;
use std::{
    fmt::{Debug, Formatter},
    future::Future,
    ops::Deref,
    pin::Pin,
    sync::Arc,
    sync::RwLock,
    task::{Context, Poll},
};

/// The default reconnect strategy.
///
/// This strategy will reconnect the client using the following algorithm:
/// - When the client returns a RestartNeeded error, the strategy will try to reconnect
/// - Thread-safety: one single reconnect attempt is allowed at the same time
/// - After reconnecting, this strategy will retry to process the request
/// - While reconnecting, this strategy will hold all the requests until the reconnect finishes
/// - When the reconnect fails, all pending requests fails with the same error message
/// - When the reconnect succeed, all pending requests are processed
///
/// # TODO:
/// - add a timeout for the reconnect
/// - set a max number of reconnect attempts, shutdown when the max number is reached
/// - automatically restore the subscriptions after reconnecting
pub struct DefaultStrategy<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    inner: Arc<SharedState<C, Fut, F>>,
}

impl<C, Fut, F> DefaultStrategy<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    pub async fn connect(builder: F) -> Result<Self, Error> {
        let client = Arc::new((builder.clone())().await?);
        Ok(Self {
            inner: Arc::new(SharedState {
                builder,
                max_pending_delay: Duration::from_secs(10), // TODO: make this configurable
                reconnect_delay: Duration::from_secs(5),    // TODO: make this configurable
                connection_status: RwLock::new(ConnectionStatus::Ready(client)),
            }),
        })
    }

    pub fn state(&self) -> Arc<SharedState<C, Fut, F>> {
        self.inner.clone()
    }

    /// Creates a future that is immediately ready if the client is idle. or pending if reconnecting.
    pub fn acquire_client(&self) -> ReadyOrWaitFuture<C, Fut, F> {
        let guard = match self.inner.connection_status.read() {
            Ok(guard) => guard,
            Err(error) => {
                return ReadyOrWaitFuture::ready(Err(Error::Custom(format!(
                    "Fatal error, client lock was poisoned: {error}"
                ))));
            }
        };

        match guard.deref().clone() {
            ConnectionStatus::Ready(client) => ReadyOrWaitFuture::ready(Ok(client)),
            ConnectionStatus::Reconnecting(future) => {
                ReadyOrWaitFuture::<C, Fut, F>::wait(self.inner.max_pending_delay, future)
            }
        }
    }

    /// Creates a future that reconnects the client, the reconnect only works when the provided
    /// client_id is greater than the current client id, this is a mechanism for avoid racing conditions.
    pub fn reconnect_or_wait(&self) -> ReadyOrWaitFuture<C, Fut, F> {
        // Make sure only one thread is handling the reconnect
        let mut guard = self
            .inner
            .connection_status
            .write()
            .expect("not poisoned; qed");

        // If the client is already reconnecting, reuse the same future
        if let ConnectionStatus::Reconnecting(future) = guard.deref() {
            return ReadyOrWaitFuture::wait(self.inner.max_pending_delay, future.clone());
        };

        // Creates a new reconnect attempt
        let reconnect_future = ReconnectFuture::new(self.inner.clone()).shared();
        *guard = ConnectionStatus::Reconnecting(reconnect_future.clone());

        ReadyOrWaitFuture::wait(
            Duration::from_secs(60), // TODO: Reconnect in another task
            reconnect_future,
        )
    }
}

impl<C, Fut, F> Reconnect<C> for DefaultStrategy<C, Fut, F>
where
    C: SubscriptionClientT + Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    type ClientRef = Arc<C>;
    type ReadyFuture<'a> = ReadyOrWaitFuture<C, Fut, F> where Self: 'a;
    type RestartNeededFuture<'a> = ReadyOrWaitFuture<C, Fut, F> where Self: 'a;
    type ReconnectFuture<'a> = ReadyOrWaitFuture<C, Fut, F> where Self: 'a;

    fn ready(&self) -> Self::ReadyFuture<'_> {
        self.acquire_client()
    }

    fn restart_needed(&self, _client: Self::ClientRef) -> Self::RestartNeededFuture<'_> {
        self.reconnect_or_wait()
    }

    fn reconnect(&self) -> Self::ReconnectFuture<'_> {
        self.reconnect_or_wait()
    }
}

// impl_client_trait!(ClientState<C> where C: ClientT + 'static + Send + Sync);
// impl_subscription_trait!(ClientState<C> where C: SubscriptionClientT + 'static + Send + Sync);

/// The connection status of the client.
pub enum ConnectionStatus<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Clone + Send + Sync + 'static,
{
    /// The client is idle and ready to receive requests.
    Ready(Arc<C>),

    /// The client is reconnecting.
    /// This stores a shared future which will resolves when the reconnect completes.
    Reconnecting(Shared<ReconnectFuture<C, Fut, F>>),
}

impl<C, Fut, F> Clone for ConnectionStatus<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Ready(client) => Self::Ready(client.clone()),
            Self::Reconnecting(future) => Self::Reconnecting(Shared::clone(future)),
        }
    }
}

/// Stores the connection status and builder.
/// This state is shared between all the clients.
pub struct SharedState<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Clone + Send + Sync + 'static,
{
    builder: F,
    /// Maximum amount of time the request will wait for the reconnect before failure.
    max_pending_delay: Duration,
    /// Amount of seconds to wait between reconnect attempts.
    reconnect_delay: Duration,
    connection_status: RwLock<ConnectionStatus<C, Fut, F>>,
}

impl<C, Fut, F> Debug for SharedState<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedState").finish_non_exhaustive()
    }
}

/// Future that resolves when the client is ready to process requests.
pub enum ReadyOrWaitState<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    Ready(Result<Arc<C>, Error>),
    Waiting(Select<Delay, Shared<ReconnectFuture<C, Fut, F>>>),
}

#[pin_project]
pub struct ReadyOrWaitFuture<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    state: Option<ReadyOrWaitState<C, Fut, F>>,
}

impl<C, Fut, F> ReadyOrWaitFuture<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    pub fn ready(result: Result<Arc<C>, Error>) -> Self {
        Self {
            state: Some(ReadyOrWaitState::Ready(result)),
        }
    }

    pub fn wait(timeout: Duration, future: Shared<ReconnectFuture<C, Fut, F>>) -> Self {
        let future = futures_util::future::select(Delay::new(timeout), future);
        Self {
            state: Some(ReadyOrWaitState::Waiting(future)),
        }
    }
}

impl<C, Fut, F> From<Error> for ReadyOrWaitFuture<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    fn from(error: Error) -> Self {
        Self::ready(Err(error))
    }
}

impl<C, Fut, F> Future for ReadyOrWaitFuture<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Send + Sync + Unpin + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    type Output = Result<Arc<C>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        loop {
            match this.state.take() {
                Some(ReadyOrWaitState::Ready(result)) => return Poll::Ready(result),
                Some(ReadyOrWaitState::Waiting(mut future)) => {
                    match future.poll_unpin(cx) {
                        // The request delay timeout
                        Poll::Ready(Either::Left(_)) => {
                            return Poll::Ready(Err(Error::Custom(
                                "Cannot process request, client reconnecting...".to_string(),
                            )));
                        }
                        // The client was reconnected!
                        Poll::Ready(Either::Right((Ok(client), _))) => {
                            return Poll::Ready(Ok(client));
                        }
                        // Failed to reconnect
                        Poll::Ready(Either::Right((Err(result), _))) => {
                            return Poll::Ready(Err(result.into_inner()));
                        }
                        Poll::Pending => {
                            *this.state = Some(ReadyOrWaitState::Waiting(future));
                            return Poll::Pending;
                        }
                    }
                }
                None => panic!("ClientReadyFuture polled after completion"),
            }
        }
    }
}

/// Future that resolves the client reconnects or fail to reconnect.
/// This future can be cloned and polled from multiple threads
pub enum ReconnectStatus<Fut> {
    /// Client is reconnecting
    Reconnecting(Fut),

    /// Client is waiting for the next reconnect attempt
    Waiting(Delay),
}

/// Future that resolves the client reconnects or fail to reconnect.
/// This future can be cloned and polled from multiple threads
#[pin_project]
pub struct ReconnectFuture<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    pub attempt: u32,
    pub state: Arc<SharedState<C, Fut, F>>,
    pub status: Option<ReconnectStatus<Fut>>,
}

impl<C, Fut, F> ReconnectFuture<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Send + Sync + Unpin + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    pub fn new(state: Arc<SharedState<C, Fut, F>>) -> Self {
        let future = (state.builder.clone())();
        Self {
            attempt: 1,
            state,
            status: Some(ReconnectStatus::Reconnecting(future)),
        }
    }
}

impl<C, Fut, F> Future for ReconnectFuture<C, Fut, F>
where
    C: Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Unpin + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    type Output = Result<Arc<C>, CloneableError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        loop {
            match this.status.take() {
                Some(ReconnectStatus::Reconnecting(mut future)) => match future.poll_unpin(cx) {
                    Poll::Ready(Ok(client)) => {
                        log::info!("Reconnect attempt {} succeeded!!", *this.attempt);
                        let client = Arc::new(client);

                        // Release the pending lock
                        let mut guard = match this.state.connection_status.write() {
                            Ok(guard) => guard,
                            Err(error) => {
                                return Poll::Ready(Err(CloneableError::from(Error::Custom(
                                    format!("Fatal error, client lock was poisoned: {error}"),
                                ))))
                            }
                        };

                        if let ConnectionStatus::Ready(client) = guard.deref() {
                            log::warn!("Racing condition detected, two reconnects running at happening at the same time");
                            return Poll::Ready(Ok(client.clone()));
                        }

                        *guard = ConnectionStatus::Ready(client.clone());
                        return Poll::Ready(Ok(client));
                    }
                    Poll::Ready(Err(error)) => {
                        log::error!(
                            "Reconnect attempt {} failed with error: {:?}",
                            *this.attempt,
                            error
                        );
                        let future = Delay::new(this.state.reconnect_delay);
                        *this.status = Some(ReconnectStatus::Waiting(future));
                        continue;
                    }
                    Poll::Pending => {
                        *this.status = Some(ReconnectStatus::Reconnecting(future));
                        break;
                    }
                },
                Some(ReconnectStatus::Waiting(mut delay)) => match delay.poll_unpin(cx) {
                    Poll::Ready(_) => {
                        let future = (this.state.builder.clone())();
                        *this.status = Some(ReconnectStatus::Reconnecting(future));
                        *this.attempt += 1;
                        continue;
                    }
                    Poll::Pending => {
                        *this.status = Some(ReconnectStatus::Waiting(delay));
                        break;
                    }
                },
                None => panic!("ReconnectFuture polled after completion"),
            }
        }
        Poll::Pending
    }
}
