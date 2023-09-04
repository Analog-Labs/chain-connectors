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

pub trait Config: 'static + Sized + Send + Sync {
    type Client: SubscriptionClientT + Send + Sync + 'static;

    type ConnectFuture: Future<Output = Result<Self::Client, Error>> + 'static + Unpin + Send;

    fn connect(&self) -> Self::ConnectFuture;
}

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
pub struct DefaultStrategy<T: Config> {
    inner: Arc<SharedState<T>>,
}

impl<T: Config> DefaultStrategy<T> {
    pub async fn connect(config: T) -> Result<Self, Error> {
        log::info!("Connecting");
        let client = Arc::new(config.connect().await?);
        Ok(Self {
            inner: Arc::new(SharedState {
                config,
                max_pending_delay: Duration::from_secs(15), // TODO: make this configurable
                min_delay_between_reconnect_retries: Duration::from_secs(5), // TODO: make this configurable
                connection_status: RwLock::new(ConnectionStatus::Ready(client)),
            }),
        })
    }

    pub fn state(&self) -> Arc<SharedState<T>> {
        self.inner.clone()
    }

    /// Creates a future that is immediately ready if the client is idle. or pending if reconnecting.
    pub fn acquire_client(&self) -> ReadyOrWaitFuture<T> {
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
                ReadyOrWaitFuture::<T>::wait(self.inner.max_pending_delay, future)
            }
        }
    }

    /// Creates a future that reconnects the client, the reconnect only works when the provided
    /// client_id is greater than the current client id, this is a mechanism for avoid racing conditions.
    pub fn reconnect_or_wait(&self) -> ReadyOrWaitFuture<T> {
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
        // TODO: Reconnect in another task/thread
        let reconnect_future = ReconnectFuture::new(self.inner.clone()).shared();
        *guard = ConnectionStatus::Reconnecting(reconnect_future.clone());

        ReadyOrWaitFuture::wait(self.inner.max_pending_delay, reconnect_future)
    }
}

impl<T: Config> Reconnect for DefaultStrategy<T> {
    type Client = T::Client;
    type ClientRef = Arc<T::Client>;
    type ReadyFuture<'a> = ReadyOrWaitFuture<T> where Self: 'a;
    type RestartNeededFuture<'a> = ReadyOrWaitFuture<T> where Self: 'a;
    type ReconnectFuture<'a> = ReadyOrWaitFuture<T> where Self: 'a;

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

/// The connection status of the client.
pub enum ConnectionStatus<T: Config> {
    /// The client is idle and ready to receive requests.
    Ready(Arc<T::Client>),

    /// The client is reconnecting.
    /// This stores a shared future which will resolves when the reconnect completes.
    Reconnecting(Shared<ReconnectFuture<T>>),
}

impl<T: Config> Clone for ConnectionStatus<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Ready(client) => Self::Ready(client.clone()),
            Self::Reconnecting(future) => Self::Reconnecting(Shared::clone(future)),
        }
    }
}

/// Stores the connection status and config.
/// This state is shared between all the clients.
pub struct SharedState<T: Config> {
    config: T,
    /// Maximum amount of time the request will wait for the reconnect before failure.
    max_pending_delay: Duration,
    /// Minimum delay allowed between reconnect attempts.
    min_delay_between_reconnect_retries: Duration,
    connection_status: RwLock<ConnectionStatus<T>>,
}

impl<T: Config> Debug for SharedState<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedState").finish_non_exhaustive()
    }
}

/// Future that resolves when the client is ready to process requests.
pub enum ReadyOrWaitState<T: Config> {
    Ready(Result<Arc<T::Client>, Error>),
    Waiting(Select<Delay, Shared<ReconnectFuture<T>>>),
}

#[pin_project]
pub struct ReadyOrWaitFuture<T: Config> {
    state: Option<ReadyOrWaitState<T>>,
}

impl<T: Config> ReadyOrWaitFuture<T> {
    pub fn ready(result: Result<Arc<T::Client>, Error>) -> Self {
        Self {
            state: Some(ReadyOrWaitState::Ready(result)),
        }
    }

    pub fn wait(timeout: Duration, future: Shared<ReconnectFuture<T>>) -> Self {
        let future = futures_util::future::select(Delay::new(timeout), future);
        Self {
            state: Some(ReadyOrWaitState::Waiting(future)),
        }
    }
}

impl<T: Config> From<Error> for ReadyOrWaitFuture<T> {
    fn from(error: Error) -> Self {
        Self::ready(Err(error))
    }
}

impl<T: Config> Future for ReadyOrWaitFuture<T> {
    type Output = Result<Arc<T::Client>, Error>;

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

/// State-machine that controls the reconnect flow and delay between retries
pub enum ReconnectStateMachine<T: Config> {
    /// Client is reconnecting and waiting for the retry delay
    ///
    /// # Description
    /// Reconnecting while waiting for the retry delay is important to
    /// guarantee consistent retries attempts, example:
    /// 1 - The retry delay is 10 seconds
    /// 2 - The reconnect attempt takes 7 seconds
    /// 3 - The next retry attempt will be in 3 seconds
    ///
    /// # State Transitions
    /// 1 - [`ReconnectStateMachine::Reconnecting`] if the reconnect attempt takes longer than the retry delay
    /// 2 - [`ReconnectStateMachine::Failure`] if reconnect fails before the retry delay, the delay is passed as parameter
    /// 3 - [`ReconnectStateMachine::Success`] if reconnect succeeds
    ReconnectAndWaitDelay(Select<Delay, T::ConnectFuture>),

    /// Retry timeout reached, waiting for reconnecting to complete
    ///
    /// # Description
    /// This state means that the reconnect attempt took longer than the retry delay, example:
    /// 1 - The retry delay is 10 seconds
    /// 2 - The reconnect attempt takes 15 seconds
    /// 3 - The next retry attempt will start immediately after the previous one failed
    ///
    /// # State Transitions
    /// 1 - [`ReconnectStateMachine::Failure`] if reconnect fails, no delay is passed as parameter
    /// 2 - [`ReconnectStateMachine::Success`] if reconnect succeeds
    Reconnecting(T::ConnectFuture),

    /// waiting for the next reconnect attempt
    ///
    /// # Description
    /// Previous reconnect attempt failed, waiting before the next reconnect attempt
    ///
    /// # State Transitions
    /// 1 - [`ReconnectStateMachine::Retry`] After the delay
    Waiting(Delay),

    /// Reconnect attempt failed, logs the error and retry to reconnect after delay
    ///
    /// # State Transitions
    /// 1 - [`ReconnectStateMachine::Retry`] if no delay is provided
    /// 2 - [`ReconnectStateMachine::Waiting`] if a delay is provided
    Failure {
        error: Error,
        maybe_delay: Option<Delay>,
    },

    /// Retrying to connect
    ///
    /// # State Transitions
    /// 1 - [`ReconnectStateMachine::ReconnectAndWaitDelay`] reconnect using the configured retry delay
    Retry,

    /// The connection was reestablished successfully
    ///
    /// # Description
    /// Update the ConnectionStatus on the [`SharedState`] and return the client
    ///
    /// # State Transitions
    /// This state is final, may return an error if the `connection_status` at [`SharedState`] was poisoned
    Success(T::Client),
}

/// Future that resolves the client reconnects or fail to reconnect.
/// This future can be cloned and polled from multiple threads
#[pin_project]
pub struct ReconnectFuture<T: Config> {
    pub attempt: u32,
    pub state: Arc<SharedState<T>>,
    pub state_machine: Option<ReconnectStateMachine<T>>,
}

impl<T: Config> ReconnectFuture<T> {
    pub fn new(state: Arc<SharedState<T>>) -> Self {
        let delay = Delay::new(state.min_delay_between_reconnect_retries);
        let reconnect = state.config.connect();
        let future = futures_util::future::select(delay, reconnect);
        Self {
            attempt: 1,
            state,
            state_machine: Some(ReconnectStateMachine::ReconnectAndWaitDelay(future)),
        }
    }
}

impl<T: Config> Future for ReconnectFuture<T> {
    type Output = Result<Arc<T::Client>, CloneableError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        loop {
            match this.state_machine.take() {
                // Client is reconnecting and the retry delay is counting
                Some(ReconnectStateMachine::ReconnectAndWaitDelay(mut future)) => {
                    match future.poll_unpin(cx) {
                        // Reconnect attempt timeout, wait for reconnect complete and retry to reconnect immediatly
                        Poll::Ready(Either::Left((_, reconnect_future))) => {
                            *this.state_machine =
                                Some(ReconnectStateMachine::Reconnecting(reconnect_future));
                            continue;
                        }

                        // Reconnect attempt failed before the retry timeout.
                        Poll::Ready(Either::Right((Err(error), delay))) => {
                            *this.state_machine = Some(ReconnectStateMachine::Failure {
                                error,
                                maybe_delay: Some(delay),
                            });
                            continue;
                        }

                        // Reconnect attempt succeeded!
                        Poll::Ready(Either::Right((Ok(client), _))) => {
                            *this.state_machine = Some(ReconnectStateMachine::Success(client));
                            continue;
                        }

                        // Pending
                        Poll::Pending => {
                            *this.state_machine =
                                Some(ReconnectStateMachine::ReconnectAndWaitDelay(future));
                            return Poll::Pending;
                        }
                    }
                }

                // Retry timeout was reached, now just wait for the reconnect to complete
                Some(ReconnectStateMachine::Reconnecting(mut future)) => {
                    match future.poll_unpin(cx) {
                        // Reconnect Succeed!
                        Poll::Ready(Ok(client)) => {
                            *this.state_machine = Some(ReconnectStateMachine::Success(client));
                            continue;
                        }

                        // Reconnect attempt failed, don't need to wait for the next retry
                        Poll::Ready(Err(error)) => {
                            *this.state_machine = Some(ReconnectStateMachine::Failure {
                                error,
                                maybe_delay: None,
                            });
                            continue;
                        }

                        // Pending
                        Poll::Pending => {
                            *this.state_machine = Some(ReconnectStateMachine::Reconnecting(future));
                            break;
                        }
                    }
                }

                // Waiting for next reconnect attempt
                Some(ReconnectStateMachine::Waiting(mut delay)) => match delay.poll_unpin(cx) {
                    // Retry timeout reached, retry to reconnect
                    Poll::Ready(_) => {
                        *this.state_machine = Some(ReconnectStateMachine::Retry);
                        continue;
                    }
                    Poll::Pending => {
                        *this.state_machine = Some(ReconnectStateMachine::Waiting(delay));
                        break;
                    }
                },

                // Reconnect attempt failed, retry to reconnect after delay or immediately
                Some(ReconnectStateMachine::Failure { error, maybe_delay }) => {
                    log::error!(
                        "Reconnect attempt {} failed with error: {:?}",
                        *this.attempt,
                        error
                    );
                    *this.state_machine = match maybe_delay {
                        Some(delay) => {
                            // Wait for delay
                            Some(ReconnectStateMachine::Waiting(delay))
                        }
                        None => {
                            // Retry immediately
                            Some(ReconnectStateMachine::Retry)
                        }
                    };
                    continue;
                }

                // Increment the attempt counter and retry to connect
                Some(ReconnectStateMachine::Retry) => {
                    let reconnect = this.state.config.connect();
                    let delay = Delay::new(this.state.min_delay_between_reconnect_retries);
                    *this.attempt += 1;
                    *this.state_machine = Some(ReconnectStateMachine::ReconnectAndWaitDelay(
                        futures_util::future::select(delay, reconnect),
                    ));
                    continue;
                }

                // Reconnect Succeeded! update the connection status and return the client
                Some(ReconnectStateMachine::Success(client)) => {
                    log::info!("Reconnect attempt {} succeeded!!", *this.attempt);
                    let client = Arc::new(client);

                    // Update connection status
                    let mut guard = match this.state.connection_status.write() {
                        Ok(guard) => guard,
                        Err(error) => {
                            log::error!("FATAL ERROR: client lock was poisoned: {error}");
                            return Poll::Ready(Err(CloneableError::from(Error::Custom(format!(
                                "FATAL ERROR: client lock was poisoned: {error}"
                            )))));
                        }
                    };

                    if let ConnectionStatus::Ready(client) = guard.deref() {
                        log::warn!(
                            "Racing condition detected, two reconnects running at the same time"
                        );
                        return Poll::Ready(Ok(client.clone()));
                    }

                    *guard = ConnectionStatus::Ready(client.clone());
                    return Poll::Ready(Ok(client));
                }
                None => panic!("ReconnectFuture polled after completion"),
            }
        }
        Poll::Pending
    }
}
