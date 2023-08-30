use ethers::types::U256;
use futures_util::future::BoxFuture;
use futures_util::{FutureExt, Stream, StreamExt};
use jsonrpsee::core::client::Subscription;
use jsonrpsee::core::error::Error as JsonRpseeError;
use pin_project::pin_project;
use serde_json::value::RawValue;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

/// Max number of failures in sequence before unsubscribing
/// A Failure occur when the publisher submits an invalid json
const MAX_FAILURES_THRESHOLD: u32 = 5;

pub enum SubscriptionStreamState {
    Idle(Subscription<serde_json::Value>),
    Receiving(Subscription<serde_json::Value>),
    Unsubscribing(BoxFuture<'static, Result<(), JsonRpseeError>>),
}

#[pin_project(project = SubscriptionStreamProj)]
pub struct SubscriptionStream {
    id: U256,
    should_unsubscribe: Arc<AtomicBool>,
    failure_count: u32,
    state: Option<SubscriptionStreamState>,
    span: tracing::Span,
}

impl SubscriptionStream {
    pub fn new(
        id: U256,
        stream: Subscription<serde_json::Value>,
        unsubscribe: Arc<AtomicBool>,
    ) -> Self {
        Self {
            id,
            should_unsubscribe: unsubscribe,
            failure_count: 0,
            state: Some(SubscriptionStreamState::Idle(stream)),
            span: tracing::info_span!("eth_subscription", id = %id, failures = 0),
        }
    }
}

impl Stream for SubscriptionStream {
    type Item = Box<RawValue>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // For streams and futures, the span is entered when the stream/future is polled
        // https://docs.rs/tracing/0.1.37/tracing/span/index.html#closing-spans
        let this = self.project();
        let _enter = this.span.enter();

        loop {
            match this.state.take() {
                // Guarantee that the stream isn't processing any more events
                Some(SubscriptionStreamState::Idle(stream)) => {
                    if this.should_unsubscribe.load(Ordering::SeqCst) {
                        tracing::info!("unsubscribing...");
                        *this.state = Some(SubscriptionStreamState::Unsubscribing(
                            stream.unsubscribe().boxed(),
                        ));
                    } else {
                        *this.state = Some(SubscriptionStreamState::Receiving(stream));
                    }
                    continue;
                }
                Some(SubscriptionStreamState::Receiving(mut stream)) => {
                    let result = match stream.poll_next_unpin(cx) {
                        Poll::Ready(result) => result,
                        Poll::Pending => {
                            *this.state = Some(SubscriptionStreamState::Receiving(stream));
                            return Poll::Pending;
                        }
                    };

                    // Stream is close, no unsubscribe needed
                    let Some(result) = result else {
                        tracing::info!("Stream closed unexpectedly, no unsubscribe needed");
                        return Poll::Ready(None);
                    };

                    // Parse the json result
                    let result = result.and_then(|value| {
                        serde_json::value::to_raw_value(&value).map_err(JsonRpseeError::ParseError)
                    });

                    match result {
                        Ok(value) => {
                            *this.state = Some(SubscriptionStreamState::Idle(stream));
                            return Poll::Ready(Some(value));
                        }
                        Err(error) => {
                            *this.failure_count += 1;
                            this.span.record("failures", *this.failure_count);
                            tracing::error!(
                                "Invalid response from eth_subscription {}: {:?}",
                                *this.failure_count,
                                error
                            );

                            if *this.failure_count > MAX_FAILURES_THRESHOLD {
                                tracing::error!(
                                    "failure limit reached, unsubscribing and closing stream"
                                );
                                this.should_unsubscribe.store(true, Ordering::SeqCst);
                                *this.state = Some(SubscriptionStreamState::Unsubscribing(
                                    stream.unsubscribe().boxed(),
                                ));
                            } else {
                                // Reset failure count
                                *this.failure_count = 0;
                                this.span.record("failures", 0);
                                *this.state = Some(SubscriptionStreamState::Idle(stream));
                            }
                            continue;
                        }
                    }
                }
                Some(SubscriptionStreamState::Unsubscribing(mut future)) => {
                    return match future.poll_unpin(cx) {
                        Poll::Ready(Ok(_)) => Poll::Ready(None),
                        Poll::Ready(Err(error)) => {
                            tracing::error!("Failed to unsubscribe: {:?}", error);
                            Poll::Ready(None)
                        }
                        Poll::Pending => {
                            *this.state = Some(SubscriptionStreamState::Unsubscribing(future));
                            Poll::Pending
                        }
                    };
                }
                None => {
                    tracing::error!("stream must not be polled after being closed`");
                    return Poll::Ready(None);
                }
            }
        }
    }
}
