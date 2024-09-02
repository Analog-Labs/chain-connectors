use std::{pin::Pin, task::{Context, Poll}, time::Duration};
use futures_util::{Future, FutureExt, future::Shared};
use futures_timer::Delay;
use super::FutureFactory;
use pin_project::pin_project;

/// Creates a throttled Future that invokes the callback at most once per every `delay` milliseconds
/// have elapsed sincethe last time the debounced function was invoked.
#[pin_project]
pub struct Throttle<F> where F: FutureFactory + 'static {
    #[pin]
    callback: F,
    delay: Duration,
    state: State<F::Future<'static>, <<F as FutureFactory>::Future<'static> as Future>::Output>,
}

impl<F: FutureFactory + 'static> Throttle<F> where F::Output: Clone {
    pub fn new(callback: F, delay: Duration) -> Self {
        Self { callback, delay, state: State::Idle }
    }
}

impl <'a, F: FutureFactory + 'a> Future for Throttle<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        *this.state = match this.state {
            State::Idle => {
                let future = this.callback.new_future();
                match std::pin::pin!(future).poll(cx) {
                    Poll::Ready(result) => {
                        State::Throttled {
                            result,
                            delay: Delay::new(*this.delay),
                        }
                    }
                    Poll::Pending => {
                        State::Pending(future)
                    }
                }
            },
            State::Pending(future) => {
                let ptr = unsafe { Pin::new_unchecked(future) };
                match ptr.poll(cx) {
                    Poll::Ready(result) => {
                        State::Throttled {
                            result,
                            delay: Delay::new(*this.delay),
                        }
                    }
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                }
            },
            State::Throttled { result, delay } => {
                match Pin::new(delay).poll(cx) {
                    Poll::Ready(()) => {
                        let future = this.callback.new_future();
                        match std::pin::pin!(future).poll(cx) {
                            Poll::Ready(result) => {
                                State::Throttled {
                                    result,
                                    delay: Delay::new(*this.delay),
                                }
                            }
                            Poll::Pending => {
                                State::Pending(future)
                            }
                        }
                    }
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                }
            },
        };
        
    }
}

enum State<Fut, R> {
    Idle,
    Pending(Fut),
    Throttled {
        result: R,
        delay: Delay,
    },
}
