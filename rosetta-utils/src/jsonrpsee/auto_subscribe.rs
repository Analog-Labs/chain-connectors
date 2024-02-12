use crate::error::LogErrorExt;
use futures_timer::Delay;
use futures_util::{future::BoxFuture, FutureExt, Stream, StreamExt};
use jsonrpsee_core::client::{Error as RpcError, Subscription};
use serde::de::DeserializeOwned;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

pub trait RetrySubscription: Unpin + Send + Sync + 'static {
    type Item: DeserializeOwned + Send + Sync + 'static;
    fn subscribe(&mut self) -> BoxFuture<'static, Result<Subscription<Self::Item>, RpcError>>;
}

impl<T, F> RetrySubscription for F
where
    T: DeserializeOwned + Send + Sync + 'static,
    F: FnMut() -> BoxFuture<'static, Result<Subscription<T>, RpcError>>
        + Unpin
        + Send
        + Sync
        + 'static,
{
    type Item = T;
    fn subscribe(&mut self) -> BoxFuture<'static, Result<Subscription<Self::Item>, RpcError>> {
        (self)()
    }
}

/// Manages the subscription's state
enum SubscriptionState<'a, T> {
    /// Currently subscribing
    Subscribing(BoxFuture<'a, Result<Subscription<T>, RpcError>>),
    /// Subscription is active.
    Subscribed(Subscription<T>),
    /// Previous subscribe attempt failed, retry after delay.
    ResubscribeAfterDelay(Delay),
    /// Previous subscribe attempt failed, retry after delay.
    Unsubscribing(BoxFuture<'a, Result<(), RpcError>>),
    /// Previous subscribe attempt failed, retry after delay.
    Unsubscribed(Option<RpcError>),
}

/// A stream which auto resubscribe when closed
pub struct AutoSubscribe<'a, T>
where
    T: RetrySubscription + 'a,
{
    /// Subscription logic
    subscriber: T,
    /// Subscription state
    state: Option<SubscriptionState<'a, T::Item>>,
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

impl<'a, T> AutoSubscribe<'a, T>
where
    T: RetrySubscription,
{
    pub fn new(retry_interval: Duration, mut subscriber: T) -> Self {
        let fut = subscriber.subscribe();
        Self {
            subscriber,
            state: Some(SubscriptionState::Subscribing(fut)),
            consecutive_subscription_errors: 0,
            total_subscriptions: 0,
            retry_interval,
            last_subscription_timestamp: None,
            unsubscribe: false,
        }
    }

    #[must_use]
    pub const fn is_initializing(&self) -> bool {
        matches!(self.state, Some(SubscriptionState::Subscribing(_))) &&
            self.total_subscriptions == 0
    }

    #[must_use]
    pub const fn is_subscribed(&self) -> bool {
        matches!(self.state, Some(SubscriptionState::Subscribed(_)))
    }

    /// Unsubscribe and consume the subscription.
    ///
    /// # Errors
    /// Return an error if the unsubscribe fails.
    pub fn unsubscribe(&mut self) {
        self.unsubscribe = true;
    }
}

impl<'a, T> Stream for AutoSubscribe<'a, T>
where
    T: RetrySubscription,
{
    type Item = Result<T::Item, RpcError>;

    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let Some(mut state) = self.state.take() else {
                return Poll::Ready(None);
            };

            // Handle unsubscribe
            if self.unsubscribe {
                state = match state {
                    // If the client is subscribing, wait for it to finish then unsubscribe.
                    SubscriptionState::Subscribing(mut fut) => match fut.poll_unpin(cx) {
                        Poll::Ready(Ok(subscription)) => {
                            // If the subscription succeeded, start the unsubscribe process.
                            SubscriptionState::Unsubscribing(subscription.unsubscribe().boxed())
                        },
                        Poll::Ready(Err(_)) => {
                            // if the subscription failed we don't need to unsubscribe.
                            SubscriptionState::Unsubscribed(None)
                        },
                        Poll::Pending => {
                            // Wait for the subscription to finish, so we can unsubscribe.
                            self.state = Some(SubscriptionState::Subscribing(fut));
                            return Poll::Pending;
                        },
                    },
                    // If the client is subscribed, start the unsubscribe process.
                    SubscriptionState::Subscribed(subscription) => {
                        SubscriptionState::Unsubscribing(subscription.unsubscribe().boxed())
                    },
                    // If the client is waiting to resubscribe, cancel the resubscribe and go to
                    // unsubscribed state.
                    SubscriptionState::ResubscribeAfterDelay(_delay) => {
                        SubscriptionState::Unsubscribed(None)
                    },
                    s => s,
                };
            }

            self.state = match state {
                /////////////////////
                // Subscribing ... //
                /////////////////////
                SubscriptionState::Subscribing(mut fut) => {
                    match fut.poll_unpin(cx) {
                        // Subscription succeeded
                        Poll::Ready(Ok(sub)) => {
                            let attempts = self.consecutive_subscription_errors;
                            if let Some(timestamp) = self.last_subscription_timestamp.take() {
                                let elapsed = timestamp.elapsed();
                                tracing::info!("succesfully resubscribed after {elapsed:?}, attemps: {attempts}", elapsed = elapsed);
                            } else if attempts > 0 {
                                tracing::info!(
                                    "succesfully subscribed after {attempts} attempt(s)"
                                );
                            }
                            // Reset error counter and update last subscription timestamp.
                            self.total_subscriptions += 1;
                            self.consecutive_subscription_errors = 0;
                            self.last_subscription_timestamp = Some(Instant::now());
                            Some(SubscriptionState::Subscribed(sub))
                        },

                        // Subscription failed
                        Poll::Ready(Err(err)) => {
                            if matches!(err, RpcError::HttpNotImplemented) {
                                // Http doesn't support subscriptions, return error and close the
                                // stream
                                return Poll::Ready(Some(Err(RpcError::HttpNotImplemented)));
                            }
                            // increment error counter and retry after delay.
                            let attempts = self.consecutive_subscription_errors + 1;
                            let msg = err.truncate();
                            tracing::error!("subscription attempt {attempts} failed: {msg}");
                            self.consecutive_subscription_errors = attempts;

                            // Schedule next subscription attempt.
                            Some(SubscriptionState::ResubscribeAfterDelay(Delay::new(
                                self.retry_interval,
                            )))
                        },

                        // Subscription is pending
                        Poll::Pending => {
                            self.state = Some(SubscriptionState::Subscribing(fut));
                            return Poll::Pending;
                        },
                    }
                },

                ////////////////////////////
                // Subscription is active //
                ////////////////////////////
                SubscriptionState::Subscribed(mut sub) => match sub.poll_next_unpin(cx) {
                    // Got a new item
                    Poll::Ready(Some(Ok(item))) => {
                        let fut = self.subscriber.subscribe();
                        self.state = Some(SubscriptionState::Subscribing(fut));
                        return Poll::Ready(Some(Ok(item)));
                    },

                    // Got an error
                    Poll::Ready(Some(Err(err))) => {
                        match err {
                            // Subscription terminated, resubscribe.
                            RpcError::RestartNeeded(msg) => {
                                tracing::error!("subscription terminated: {}", msg.truncate());
                                Some(SubscriptionState::Subscribing(self.subscriber.subscribe()))
                            },
                            // Http doesn't support subscriptions, return error and close the stream
                            RpcError::HttpNotImplemented => {
                                return Poll::Ready(Some(Err(RpcError::HttpNotImplemented)));
                            },
                            // Return error
                            err => {
                                let fut = self.subscriber.subscribe();
                                self.state = Some(SubscriptionState::Subscribing(fut));
                                return Poll::Ready(Some(Err(err)));
                            },
                        }
                    },

                    // Stream was close, resubscribe.
                    Poll::Ready(None) => {
                        Some(SubscriptionState::Subscribing(self.subscriber.subscribe()))
                    },

                    // Stream is pending
                    Poll::Pending => {
                        self.state = Some(SubscriptionState::Subscribed(sub));
                        return Poll::Pending;
                    },
                },

                /////////////
                // Waiting //
                /////////////
                SubscriptionState::ResubscribeAfterDelay(mut delay) => match delay.poll_unpin(cx) {
                    Poll::Ready(()) => {
                        // Timeout elapsed, retry subscription.
                        Some(SubscriptionState::Subscribing(self.subscriber.subscribe()))
                    },
                    Poll::Pending => {
                        self.state = Some(SubscriptionState::ResubscribeAfterDelay(delay));
                        return Poll::Pending;
                    },
                },

                /////////////////////
                // Unsubscribing.. //
                /////////////////////
                SubscriptionState::Unsubscribing(mut fut) => match fut.poll_unpin(cx) {
                    Poll::Ready(res) => {
                        // Timeout elapsed, retry subscription.
                        self.state = Some(SubscriptionState::Unsubscribed(res.err()));
                        return Poll::Ready(None);
                    },
                    Poll::Pending => {
                        self.state = Some(SubscriptionState::Unsubscribing(fut));
                        return Poll::Pending;
                    },
                },

                //////////////////
                // Unsubscribed //
                //////////////////
                SubscriptionState::Unsubscribed(maybe_err) => {
                    if self.unsubscribe {
                        self.state = Some(SubscriptionState::Unsubscribed(maybe_err));
                        return Poll::Ready(None);
                    }
                    Some(SubscriptionState::Subscribing(self.subscriber.subscribe()))
                },
            };
        }
    }
}
