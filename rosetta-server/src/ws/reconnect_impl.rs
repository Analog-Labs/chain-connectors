#![allow(dead_code)]
use super::{error::CloneableError, extension::Extended, reconnect::Reconnect};
use arc_swap::ArcSwap;
use futures_util::{
    future::{BoxFuture, Shared},
    FutureExt,
};
use jsonrpsee::core::{client::SubscriptionClientT, error::Error};
use pin_project::pin_project;
use std::{
    convert::AsRef,
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
pub struct DefaultStrategy<C, F> {
    inner: Arc<SharedState<C, F>>,
}

impl<C, Fut, F> DefaultStrategy<C, F>
where
    C: SubscriptionClientT + Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    pub async fn connect(builder: F) -> Result<Self, Error> {
        let client = Arc::new((builder.clone())().await?);
        Ok(Self {
            inner: Arc::new(SharedState {
                builder,
                connection_status: RwLock::new(ConnectionStatus::Idle(0)),
                client: ArcSwap::from(Arc::new(Extended::new(ClientState { id: 0, client }))),
            }),
        })
    }

    pub fn state(&self) -> Arc<SharedState<C, F>> {
        self.inner.clone()
    }

    /// Creates a future that is immediately ready if the client is idle. or pending if reconnecting.
    pub fn acquire_client(&self) -> ClientReadyFuture<C, ReconnectAttempt<C>, F> {
        // Check if the client is reconnecting
        let guard = match self.inner.connection_status.read() {
            Ok(guard) => guard,
            Err(error) => {
                return ClientReadyFuture::from(Error::Custom(format!(
                    "Fatal error, client lock was poisoned: {error}"
                )))
            }
        };

        let connection_status = guard.deref().clone();

        // If the client is reconnecting, wait for the reconnect to finish
        if let ConnectionStatus::Reconnecting(future) = connection_status {
            return ClientReadyFuture::<C, ReconnectAttempt<C>, F>::reconnecting(
                future.client_id,
                self.inner.clone(),
                future,
            );
        }
        drop(guard);

        let result = self.inner.client.load().clone();
        ClientReadyFuture::ready(Ok(result))
    }

    /// Creates a future that reconnects the client, the reconnect only works when the provided
    /// client_id is greater than the current client id, this is a mechanism for avoid racing conditions.
    pub fn reconnect_with_client_id(&self, client_id: u32) -> ReconnectAttempt<C> {
        let state = self.state();

        // Make sure only one thread is handling the reconnect
        let mut guard = match state.connection_status.write() {
            Ok(guard) => guard,
            Err(error) => {
                return ReconnectAttempt::failure(Error::Custom(format!(
                    "Fatal error, client lock was poisoned: {error}"
                )))
            }
        };

        // If the client is already reconnecting, reuse the same future
        let connection_status = guard.deref().clone();
        let actual_client_id = match connection_status {
            ConnectionStatus::Reconnecting(future) => return future,
            ConnectionStatus::Idle(attempt) => attempt,
        };

        // If the provided client_id is less than the actual client id, simply return the current client
        if client_id <= actual_client_id {
            let client = state.client.load().clone();
            return ReconnectAttempt::<C>::new(
                actual_client_id,
                futures_util::future::ready(Ok(client)).boxed(),
            );
        }

        // Creates a new reconnect attempt
        let reconnect_future = {
            let future = (self.inner.builder.clone())();
            ReconnectAttempt::new(
                client_id,
                async move {
                    let state = Arc::new(Extended::new(ClientState {
                        id: client_id,
                        client: Arc::new(future.await?),
                    }));
                    Ok(state)
                }
                .boxed(),
            )
        };

        // Store the reconnect attempt
        *guard = ConnectionStatus::Reconnecting(reconnect_future.clone());

        reconnect_future
    }
}

pub type Client<C> = Extended<C, ClientState<C>>;
pub type ClientRef<C> = Arc<Client<C>>;
pub type ClientId = u32;

impl<C, Fut, F> Reconnect<Client<C>> for DefaultStrategy<C, F>
where
    C: SubscriptionClientT + Send + Sync + 'static,
    Fut: Future<Output = Result<C, Error>> + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send + Sync + Clone + 'static,
{
    type ClientRef = ClientRef<C>;
    type ReadyFuture<'a> = ClientReadyFuture<C, ReconnectAttempt<C>, F> where Self: 'a;
    type RestartNeededFuture<'a> = ReconnectAttempt<C> where Self: 'a;
    type ReconnectFuture<'a> = ReconnectAttempt<C> where Self: 'a;

    fn ready(&self) -> Self::ReadyFuture<'_> {
        self.acquire_client()
    }

    fn restart_needed(&self, client: Self::ClientRef) -> Self::RestartNeededFuture<'_> {
        let client_id = client.state().id;
        self.reconnect_with_client_id(client_id + 1)
    }

    fn reconnect(&self) -> Self::ReconnectFuture<'_> {
        let client_id = self.state().client.load().state().id;
        self.reconnect_with_client_id(client_id + 1)
    }
}

/// Stores the client id and the client. Needed to avoid racing conditions.
pub struct ClientState<C> {
    id: ClientId,
    client: Arc<C>,
}

impl<C> Clone for ClientState<C> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            client: self.client.clone(),
        }
    }
}

impl<C> AsRef<C> for ClientState<C> {
    fn as_ref(&self) -> &C {
        &self.client
    }
}

/// The connection status of the client.
pub enum ConnectionStatus<C> {
    /// The client is idle and ready to receive requests.
    Idle(ClientId),

    /// The client is reconnecting.
    /// This stores a shared future which will resolves when the reconnect completes.
    Reconnecting(ReconnectAttempt<C>),
}

impl<C> Clone for ConnectionStatus<C> {
    fn clone(&self) -> Self {
        match self {
            Self::Idle(attempt) => Self::Idle(*attempt),
            Self::Reconnecting(attempt) => Self::Reconnecting(attempt.clone()),
        }
    }
}

/// Stores the connection status and builder.
/// This state is shared between all the clients.
pub struct SharedState<C, F> {
    builder: F,
    connection_status: RwLock<ConnectionStatus<C>>,
    client: ArcSwap<Client<C>>,
}

impl<C, F> Debug for SharedState<C, F>
where
    C: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedState").finish_non_exhaustive()
    }
}

/// Future that resolves when the client is ready to process requests.
pub enum ClientReadyState<C, Fut, F> {
    Ready(Result<ClientRef<C>, Error>),
    Reconnecting {
        client_id: u32,
        shared: Arc<SharedState<C, F>>,
        future: Fut,
    },
}

#[pin_project]
pub struct ClientReadyFuture<C, Fut, F> {
    state: Option<ClientReadyState<C, Fut, F>>,
}

impl<C, Fut, F> ClientReadyFuture<C, Fut, F> {
    pub fn ready(result: Result<ClientRef<C>, Error>) -> Self {
        Self {
            state: Some(ClientReadyState::Ready(result)),
        }
    }

    pub fn reconnecting(attempt: u32, shared: Arc<SharedState<C, F>>, future: Fut) -> Self {
        Self {
            state: Some(ClientReadyState::Reconnecting {
                client_id: attempt,
                shared,
                future,
            }),
        }
    }
}

impl<C, Fut, F> From<Error> for ClientReadyFuture<C, Fut, F> {
    fn from(error: Error) -> Self {
        Self::ready(Err(error))
    }
}

impl<C, Fut, F> Future for ClientReadyFuture<C, Fut, F>
where
    Fut: Future<Output = Result<ClientRef<C>, Error>> + Send + Sync + Unpin + 'static,
    F: Send + Sync + 'static,
    C: Send + Sync + 'static,
{
    type Output = Result<ClientRef<C>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        loop {
            match this.state.take() {
                Some(ClientReadyState::Ready(result)) => return Poll::Ready(result),
                Some(ClientReadyState::Reconnecting {
                    client_id,
                    shared,
                    mut future,
                }) => {
                    match future.poll_unpin(cx) {
                        Poll::Ready(result) => {
                            // Release the pending lock
                            let mut guard = match shared.connection_status.write() {
                                Ok(guard) => guard,
                                Err(error) => {
                                    return Poll::Ready(Err(Error::Custom(format!(
                                        "Fatal error, client lock was poisoned: {error}"
                                    ))))
                                }
                            };

                            let connection_status = guard.deref().clone();

                            // Checks if the client needs to be updated
                            let should_update = match connection_status {
                                ConnectionStatus::Idle(current_client_id) => {
                                    client_id > current_client_id
                                }
                                ConnectionStatus::Reconnecting(future) => {
                                    // Store the new client
                                    if client_id >= future.client_id {
                                        true
                                    } else {
                                        panic!("two reconnects at the same time (this is a bug)");
                                    }
                                }
                            };

                            if should_update {
                                // Update the connection status
                                *guard = ConnectionStatus::Idle(client_id);

                                // Store the new client
                                if let Ok(client) = &result {
                                    shared.client.store(client.clone());
                                }
                            }

                            return Poll::Ready(result);
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
                None => panic!("ClientReadyFuture polled after completion"),
            }
        }
    }
}

/// Future that resolves the client reconnects or fail to reconnect.
/// This future can be cloned and polled from multiple threads
#[pin_project]
pub struct ReconnectAttempt<C> {
    pub client_id: ClientId,
    pub future: Shared<BoxFuture<'static, Result<ClientRef<C>, CloneableError>>>,
}

impl<C> Debug for ReconnectAttempt<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReconnectAttempt")
            .field("attempt", &self.client_id)
            .field("future", &self.future)
            .finish()
    }
}

impl<C> Clone for ReconnectAttempt<C> {
    fn clone(&self) -> Self {
        Self {
            client_id: self.client_id,
            future: self.future.clone(),
        }
    }
}

impl<C> ReconnectAttempt<C> {
    pub fn new(
        client_id: ClientId,
        future: BoxFuture<'static, Result<ClientRef<C>, CloneableError>>,
    ) -> Self {
        Self {
            client_id,
            future: future.shared(),
        }
    }

    pub fn failure(error: Error) -> Self {
        Self {
            client_id: 0,
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
