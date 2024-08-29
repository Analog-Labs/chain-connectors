#![allow(clippy::option_if_let_else)]
use super::FutureFactory;
use futures_timer::Delay;
use futures_util::{future::BoxFuture, FutureExt, Stream, StreamExt};
use jsonrpsee_core::client::{Error as RpcError, Subscription};
use serde::de::DeserializeOwned;
use std::{
    mem,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

/// Manages the subscription's state
enum State<T, F>
where
    F: FutureFactory<Output = Result<Subscription<T>, RpcError>>,
{
    /// Idle
    Idle(F),
    /// Currently subscribing
    Subscribing(BoxFuture<'static, (F, F::Output)>),
    /// Subscription is active.
    Subscribed { subscriber: F, subscription: Subscription<T> },
    /// Previous subscribe attempt failed, retry after delay.
    ResubscribeAfterDelay { subscriber: F, delay: Delay },
    /// Unsubscribing from stream.
    Unsubscribing { subscriber: F, fut: BoxFuture<'static, Result<(), RpcError>> },
    /// Unsubscribed.
    Unsubscribed { subscriber: F, result: Option<RpcError> },
    /// Subscription is poisoned
    Poisoned,
}

/// A stream which auto resubscribe when closed
#[pin_project::pin_project]
pub struct AutoSubscribe<T, F>
where
    F: FutureFactory<Output = Result<Subscription<T>, RpcError>>,
{
    /// Subscription state
    state: State<T, F>,
    /// Count of consecutive errors.
    pub consecutive_subscription_errors: u32,
    /// Total number of successful subscriptions.
    pub total_subscriptions: u32,
    /// Min interval between subscription attemps
    pub retry_interval: Duration,
    /// The timestamp of the last successful subscription.
    pub last_subscription_timestamp: Option<Instant>,
    pub unsubscribe: bool,
}

impl<T, F> AutoSubscribe<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: FutureFactory<Output = Result<Subscription<T>, RpcError>>,
{
    pub const fn new(retry_interval: Duration, subscriber: F) -> Self {
        Self {
            state: State::Idle(subscriber),
            consecutive_subscription_errors: 0,
            total_subscriptions: 0,
            retry_interval,
            last_subscription_timestamp: None,
            unsubscribe: false,
        }
    }

    #[must_use]
    pub const fn is_initializing(&self) -> bool {
        matches!(self.state, State::Idle(_) | State::Subscribing(_)) &&
            self.total_subscriptions == 0
    }

    #[must_use]
    pub const fn is_subscribed(&self) -> bool {
        matches!(self.state, State::Subscribed { .. })
    }

    #[must_use]
    pub const fn terminated(&self) -> bool {
        matches!(self.state, State::Poisoned | State::Unsubscribed { .. })
    }

    /// Unsubscribe and consume the subscription.
    ///
    /// # Errors
    /// Return an error if the unsubscribe fails.
    pub fn unsubscribe(&mut self) {
        self.unsubscribe = true;
    }

    /// Consume the subscription and return the inner subscriber.
    pub fn into_subscriber(self) -> Option<F> {
        match self.state {
            State::Idle(subscriber) |
            State::Subscribed { subscriber, .. } |
            State::ResubscribeAfterDelay { subscriber, .. } |
            State::Unsubscribing { subscriber, .. } |
            State::Unsubscribed { subscriber, .. } => Some(subscriber),
            State::Subscribing(fut) => fut.now_or_never().map(|(subscriber, _)| subscriber),
            State::Poisoned => None,
        }
    }
}

impl<T, F> Stream for AutoSubscribe<T, F>
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: FutureFactory<Output = Result<Subscription<T>, RpcError>>,
{
    type Item = Result<T, RpcError>;

    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        loop {
            match mem::replace(this.state, State::Poisoned) {
                State::Idle(mut subscriber) => {
                    // Check if it was requested to unsubscribe.
                    if *this.unsubscribe {
                        *this.state = State::Unsubscribed { subscriber, result: None };
                        continue;
                    }

                    // Subscribe
                    let fut = async move {
                        let result = subscriber.new_future().await;
                        (subscriber, result)
                    }
                    .boxed();
                    *this.state = State::Subscribing(fut);
                    continue;
                },

                /////////////////////
                // Subscribing ... //
                /////////////////////
                State::Subscribing(mut fut) => match fut.poll_unpin(cx) {
                    // Subscription succeeded
                    Poll::Ready((subscriber, Ok(subscription))) => {
                        let attempts = *this.consecutive_subscription_errors;
                        if let Some(timestamp) = this.last_subscription_timestamp.take() {
                            let elapsed = timestamp.elapsed();
                            tracing::info!(
                                "succesfully resubscribed after {elapsed:?}, attemps: {attempts}",
                                elapsed = elapsed
                            );
                        } else if attempts > 0 {
                            tracing::info!("succesfully subscribed after {attempts} attempt(s)");
                        }
                        // Reset error counter and update last subscription timestamp.
                        *this.total_subscriptions += 1;
                        *this.consecutive_subscription_errors = 0;
                        *this.last_subscription_timestamp = Some(Instant::now());
                        *this.state = State::Subscribed { subscriber, subscription };
                    },

                    // Subscription failed
                    Poll::Ready((subscriber, Err(err))) => {
                        // Check if it was requested to unsubscribe.
                        if *this.unsubscribe {
                            *this.state = State::Unsubscribed { subscriber, result: None };
                            continue;
                        }
                        if matches!(err, RpcError::HttpNotImplemented) {
                            // Http doesn't support subscriptions, return error and close the stream
                            *this.state = State::Unsubscribed { subscriber, result: None };
                            return Poll::Ready(Some(Err(RpcError::HttpNotImplemented)));
                        }
                        // increment error counter and retry after delay.
                        let attempts = *this.consecutive_subscription_errors + 1;
                        *this.consecutive_subscription_errors = attempts;

                        // Schedule next subscription attempt.
                        *this.state = State::ResubscribeAfterDelay {
                            subscriber,
                            delay: Delay::new(*this.retry_interval),
                        };
                    },

                    // Subscription is pending
                    Poll::Pending => {
                        *this.state = State::Subscribing(fut);
                        return Poll::Pending;
                    },
                },

                ////////////////////////////
                // Subscription is active //
                ////////////////////////////
                State::Subscribed { subscriber, mut subscription } => {
                    // Check if it was requested to unsubscribe.
                    if *this.unsubscribe {
                        *this.state = State::Unsubscribing {
                            subscriber,
                            fut: subscription.unsubscribe().boxed(),
                        };
                        continue;
                    }
                    match subscription.poll_next_unpin(cx) {
                        // Got a new item
                        Poll::Ready(Some(Ok(item))) => {
                            *this.state = State::Subscribed { subscriber, subscription };
                            return Poll::Ready(Some(Ok(item)));
                        },

                        // Got an error
                        Poll::Ready(Some(Err(err))) => {
                            *this.state = State::Subscribed { subscriber, subscription };
                            return Poll::Ready(Some(Err(RpcError::ParseError(err))));
                        },

                        // Stream was close, resubscribe.
                        Poll::Ready(None) => {
                            tracing::warn!("subscription websocket closed.. resubscribing.");
                            *this.state = State::Idle(subscriber);
                        },

                        // Stream is pending
                        Poll::Pending => {
                            *this.state = State::Subscribed { subscriber, subscription };
                            return Poll::Pending;
                        },
                    }
                },

                /////////////
                // Waiting //
                /////////////
                State::ResubscribeAfterDelay { subscriber, mut delay } => {
                    match delay.poll_unpin(cx) {
                        Poll::Ready(()) => {
                            // Timeout elapsed, retry subscription.
                            *this.state = State::Idle(subscriber);
                        },
                        Poll::Pending => {
                            *this.state = State::ResubscribeAfterDelay { subscriber, delay };
                            return Poll::Pending;
                        },
                    }
                },

                /////////////////////
                // Unsubscribing.. //
                /////////////////////
                State::Unsubscribing { subscriber, mut fut } => match fut.poll_unpin(cx) {
                    Poll::Ready(res) => {
                        // Timeout elapsed, retry subscription.
                        *this.state = State::Unsubscribed { subscriber, result: res.err() };
                    },
                    Poll::Pending => {
                        *this.state = State::Unsubscribing { subscriber, fut };
                        return Poll::Pending;
                    },
                },

                //////////////////
                // Unsubscribed //
                //////////////////
                State::Unsubscribed { subscriber, mut result } => {
                    // Only return error if it wasn't requested to unsubscribe.
                    if !*this.unsubscribe {
                        if let Some(err) = result.take() {
                            *this.state = State::Unsubscribed { subscriber, result: None };
                            return Poll::Ready(Some(Err(err)));
                        }
                    }
                    *this.state = State::Unsubscribed { subscriber, result };
                    return Poll::Ready(None);
                },

                //////////////
                // Poisoned //
                //////////////
                State::Poisoned => {
                    panic!("Stream is poisoned");
                },
            };
        }
    }
}
