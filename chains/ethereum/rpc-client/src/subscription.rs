use ethers::types::U256;
use futures_util::future::BoxFuture;
use futures_util::{FutureExt, Stream, StreamExt};
use jsonrpsee::core::client::Subscription;
use jsonrpsee::core::error::Error as JsonRpseeError;
use pin_project::pin_project;
use serde_json::value::RawValue;
use std::pin::Pin;
use std::task::{Context, Poll};

pub enum SubscriptionStreamState {
    Subscribed(Subscription<serde_json::Value>),
    Unsubscribing(BoxFuture<'static, Result<(), JsonRpseeError>>),
}

#[pin_project]
pub struct SubscriptionStream {
    id: U256,
    failures: u32,
    state: Option<SubscriptionStreamState>,
}

impl SubscriptionStream {
    pub fn new(id: U256, stream: Subscription<serde_json::Value>) -> Self {
        Self {
            id,
            failures: 0,
            state: Some(SubscriptionStreamState::Subscribed(stream)),
        }
    }
}

impl Stream for SubscriptionStream {
    type Item = Box<RawValue>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        loop {
            match this.state.take() {
                Some(SubscriptionStreamState::Subscribed(mut stream)) => {
                    let result = match stream.poll_next_unpin(cx) {
                        Poll::Ready(result) => result,
                        Poll::Pending => {
                            *this.state = Some(SubscriptionStreamState::Subscribed(stream));
                            return Poll::Pending;
                        }
                    };

                    // Stream is close
                    let Some(result) = result else {
                        return Poll::Ready(None);
                    };

                    // Parse the result
                    let result = result.and_then(|value| {
                        serde_json::value::to_raw_value(&value).map_err(JsonRpseeError::ParseError)
                    });

                    match result {
                        Ok(value) => {
                            *this.state = Some(SubscriptionStreamState::Subscribed(stream));
                            return Poll::Ready(Some(value));
                        }
                        Err(error) => {
                            log::error!(
                                "Invalid response from subscription error {}: {:?}",
                                this.failures,
                                error
                            );
                            *this.failures += 1;

                            if *this.failures > 5 {
                                log::error!("Too many errors, unsubscribing...");
                                *this.state = Some(SubscriptionStreamState::Unsubscribing(
                                    stream.unsubscribe().boxed(),
                                ));
                            } else {
                                *this.state = Some(SubscriptionStreamState::Subscribed(stream));
                            }
                            continue;
                        }
                    }
                }
                Some(SubscriptionStreamState::Unsubscribing(mut future)) => {
                    return match future.poll_unpin(cx) {
                        Poll::Ready(Ok(_)) => Poll::Ready(None),
                        Poll::Ready(Err(error)) => {
                            log::error!("Failed to unsubscribe: {:?}", error);
                            Poll::Ready(None)
                        }
                        Poll::Pending => {
                            *this.state = Some(SubscriptionStreamState::Unsubscribing(future));
                            Poll::Pending
                        }
                    };
                }
                None => {
                    log::error!("stream must not be polled after being closed`");
                    return Poll::Ready(None);
                }
            }
        }
    }
}
