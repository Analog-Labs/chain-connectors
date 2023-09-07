#![allow(dead_code)]
use super::{error::CloneableError, reconnect::Reconnect};
use futures_timer::Delay;
use futures_util::{
    future::{Either, Select, Shared},
    FutureExt,
};
use jsonrpsee::core::{client::SubscriptionClientT, error::Error};
use pin_project::pin_project;
use std::{
    fmt::{Debug, Formatter},
    future::Future,
    num::NonZeroU32,
    ops::Deref,
    pin::Pin,
    sync::Arc,
    sync::RwLock,
    task::{Context, Poll},
    time::Duration,
};

pub trait Config: 'static + Sized + Send + Sync + Debug {
    type Client: SubscriptionClientT + Debug + Send + Sync + 'static;

    type ConnectFuture: Future<Output = Result<Self::Client, Error>> + 'static + Unpin + Send;

    /// A retry strategy
    type RetryStrategy: Iterator<Item = Duration> + 'static + Send + Sync;

    /// Maximum time the request will wait for the reconnect before fails.
    fn max_pending_delay(&self) -> Duration;

    /// The strategy that drives multiple attempts at an action via a retry strategy
    /// and a reconnect strategy.
    ///
    /// # Example of Retry Strategies:
    /// - FixedInterval: A retry is performed in fixed intervals.
    /// - Exponential Backoff: The resulting duration is calculated by taking the base to the `n`-th power,
    /// where `n` denotes the number of past attempts.
    fn retry_strategy(&self) -> Self::RetryStrategy;

    /// Try to connect to the client.
    fn connect(&self) -> Self::ConnectFuture;

    /// Checks whether the client is connected or not.
    /// returns None if the client doesn't support this feature.
    fn is_connected(&self, _client: &Self::Client) -> Option<bool> {
        None
    }
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
#[derive(Debug)]
pub struct DefaultStrategy<T: Config> {
    inner: Arc<SharedState<T>>,
}

impl<T: Config> Clone for DefaultStrategy<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Config> DefaultStrategy<T> {
    pub async fn connect(config: T) -> Result<Self, Error> {
        let client = Arc::new(config.connect().await?);
        Ok(Self {
            inner: Arc::new(SharedState {
                config,
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
                    "FATAL ERROR, client lock was poisoned: {error}"
                ))));
            }
        };

        match guard.deref().clone() {
            ConnectionStatus::Ready(client) => ReadyOrWaitFuture::ready(Ok(client)),
            ConnectionStatus::Reconnecting(future) => {
                ReadyOrWaitFuture::<T>::wait(self.inner.config.max_pending_delay(), future)
            }
        }
    }

    /// Creates a future that reconnects the client, the reconnect only works when the provided
    /// client_id is greater than the current client id, this is a mechanism for avoid racing conditions.
    pub fn reconnect_or_wait(&self) -> ReadyOrWaitFuture<T> {
        // Acquire write lock, making sure only one thread is handling the reconnect
        let mut guard = match self.inner.connection_status.write() {
            Ok(guard) => guard,
            Err(error) => {
                return ReadyOrWaitFuture::ready(Err(Error::Custom(format!(
                    "FATAL ERROR, client lock was poisoned: {error}"
                ))));
            }
        };

        // If the client is already reconnecting, reuse the same future
        if let ConnectionStatus::Reconnecting(future) = guard.deref() {
            return ReadyOrWaitFuture::wait(self.inner.config.max_pending_delay(), future.clone());
        };

        // Update the connection status to reconnecting
        // TODO: Reconnect in another task/thread
        let reconnect_future = ReconnectFuture::new(self.inner.clone()).shared();
        *guard = ConnectionStatus::Reconnecting(reconnect_future.clone());

        ReadyOrWaitFuture::wait(self.inner.config.max_pending_delay(), reconnect_future)
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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct SharedState<T: Config> {
    config: T,
    connection_status: RwLock<ConnectionStatus<T>>,
}

/// Future that resolves when the client is connected or
/// when the timeout is reached.
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
                                "Timeout: cannot process request, client reconnecting..."
                                    .to_string(),
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

    /// Waiting for reconnecting to complete, retry immediately if fails
    ///
    /// # Description
    /// This state is reached in two cases:
    /// 1 - The current reconnect attempt took longer than the retry delay
    /// 2 - or the retry_strategy returns None, meaning that there is no delay between retries
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
    /// 1 - if retry_strategy.next() is Some(delay), transition to [`ReconnectStateMachine::ReconnectAndWaitDelay`]
    /// 2 - if retry_strategy.next() is None, transition to [`ReconnectStateMachine::Reconnecting`]
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

impl<T: Config> Debug for ReconnectStateMachine<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReconnectStateMachine::ReconnectAndWaitDelay(_) => {
                f.debug_struct("ReconnectStateMachine::ReconnectAndWaitDelay")
            }
            ReconnectStateMachine::Reconnecting(_) => {
                f.debug_struct("ReconnectStateMachine::Reconnecting")
            }
            ReconnectStateMachine::Waiting(_) => f.debug_struct("ReconnectStateMachine::Waiting"),
            ReconnectStateMachine::Failure { .. } => {
                f.debug_struct("ReconnectStateMachine::Failure")
            }
            ReconnectStateMachine::Retry => f.debug_struct("ReconnectStateMachine::Retry"),
            ReconnectStateMachine::Success(_) => f.debug_struct("ReconnectStateMachine::Success"),
        }
        .finish()
    }
}

/// Future that resolves the client reconnects or fail to reconnect.
/// This future can be cloned and polled from multiple threads
#[derive(Debug)]
#[pin_project]
pub struct ReconnectFuture<T: Config> {
    pub attempt: NonZeroU32,
    pub retry_strategy: T::RetryStrategy,
    pub state: Arc<SharedState<T>>,
    pub state_machine: Option<ReconnectStateMachine<T>>,
}

impl<T: Config> ReconnectFuture<T> {
    pub fn new(state: Arc<SharedState<T>>) -> Self {
        let mut retry_strategy = state.config.retry_strategy();
        let reconnect = state.config.connect();
        let state_machine = match retry_strategy.next() {
            None => ReconnectStateMachine::Reconnecting(reconnect),
            Some(delay) => {
                let delay = Delay::new(delay);
                let future = futures_util::future::select(delay, reconnect);
                ReconnectStateMachine::ReconnectAndWaitDelay(future)
            }
        };

        let attempt = NonZeroU32::new(1).expect("non zero; qed");
        Self {
            attempt,
            retry_strategy,
            state,
            state_machine: Some(state_machine),
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
                    *this.attempt = (*this.attempt).saturating_add(1);
                    let reconnect = this.state.config.connect();
                    let next_state = match this.retry_strategy.next() {
                        None => ReconnectStateMachine::Reconnecting(reconnect),
                        Some(delay) => {
                            let delay = Delay::new(delay);
                            let future = futures_util::future::select(delay, reconnect);
                            ReconnectStateMachine::ReconnectAndWaitDelay(future)
                        }
                    };
                    *this.state_machine = Some(next_state);
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
