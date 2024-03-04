use super::FutureFactory;
use futures_timer::Delay;
use futures_util::{future::BoxFuture, FutureExt, Stream};
use pin_project::pin_project;
use std::{
    mem,
    pin::Pin,
    task::Poll,
    time::{Duration, Instant},
};

enum State<F: FutureFactory> {
    Idle(F),
    Polling { fut: BoxFuture<'static, (F, F::Output)>, request_time: Instant },
    Waiting { factory: F, delay: Delay },
    Poisoned,
}

/// Polls a future at a fixed interval, adjusting the interval based on the actual time it took to
/// complete the future.
#[pin_project(project = PollingStreamProj)]
pub struct PollingInterval<F: FutureFactory> {
    state: State<F>,
    target_interval: Duration,
    interval: Duration,
    last_request_timestamp: Option<Instant>,
}

impl<F: FutureFactory> PollingInterval<F> {
    #[must_use]
    pub const fn new(factory: F, interval: Duration) -> Self {
        Self {
            state: State::Idle(factory),
            target_interval: interval,
            interval,
            last_request_timestamp: None,
        }
    }

    #[must_use]
    pub const fn factory(&self) -> Option<&F> {
        match &self.state {
            State::Idle(factory) | State::Waiting { factory, .. } => Some(factory),
            State::Polling { .. } | State::Poisoned => None,
        }
    }

    pub fn factory_mut(&mut self) -> Option<&mut F> {
        match &mut self.state {
            State::Idle(factory) | State::Waiting { factory, .. } => Some(factory),
            State::Polling { .. } | State::Poisoned => None,
        }
    }
}

impl<F: FutureFactory> Stream for PollingInterval<F> {
    type Item = F::Output;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();
        loop {
            match mem::replace(this.state, State::Poisoned) {
                State::Idle(mut factory) => {
                    *this.state = State::Polling {
                        fut: async move {
                            let result = factory.new_future().await;
                            (factory, result)
                        }
                        .boxed(),
                        request_time: Instant::now(),
                    };
                    continue;
                },
                State::Polling { mut fut, request_time } => match fut.poll_unpin(cx) {
                    Poll::Ready((mut factory, result)) => {
                        let target_interval = *this.target_interval;
                        if let Some(last_request_timestamp) = *this.last_request_timestamp {
                            // Adjust the polling interval by 10%
                            let actual_interval =
                                i64::try_from(last_request_timestamp.elapsed().as_millis())
                                    .unwrap_or(i64::MAX);
                            let target_interval =
                                i64::try_from(target_interval.as_millis()).unwrap_or(i64::MAX);
                            let interval =
                                i64::try_from(this.interval.as_millis()).unwrap_or(i64::MAX);
                            let error = actual_interval - target_interval;
                            let next_interval = ((interval * 9) + (interval - error).max(0)) / 10;
                            *this.interval =
                                Duration::from_millis(next_interval.max(0).unsigned_abs());
                        } else {
                            // If this is the first request, the interval is defined as
                            // target_interval - request_time
                            let elapsed = request_time.elapsed();
                            if elapsed >= target_interval {
                                *this.interval = Duration::ZERO;
                            } else {
                                *this.interval = target_interval - elapsed;
                            }
                        }
                        *this.last_request_timestamp = Some(Instant::now());

                        let interval = *this.interval;
                        if interval > Duration::ZERO {
                            *this.state = State::Waiting { factory, delay: Delay::new(interval) };
                        } else {
                            *this.state = State::Polling {
                                fut: async move {
                                    let result = factory.new_future().await;
                                    (factory, result)
                                }
                                .boxed(),
                                request_time: Instant::now(),
                            };
                        };
                        return Poll::Ready(Some(result));
                    },
                    Poll::Pending => {
                        *this.state = State::Polling { fut, request_time };
                        return Poll::Pending;
                    },
                },
                State::Waiting { mut factory, mut delay } => match delay.poll_unpin(cx) {
                    Poll::Ready(()) => {
                        *this.state = State::Polling {
                            fut: async move {
                                let result = factory.new_future().await;
                                (factory, result)
                            }
                            .boxed(),
                            request_time: Instant::now(),
                        };
                        continue;
                    },
                    Poll::Pending => {
                        *this.state = State::Waiting { factory, delay };
                        return Poll::Pending;
                    },
                },
                State::Poisoned => {
                    unreachable!("PollingInterval is poisoned")
                },
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{future::BoxFuture, StreamExt};
    use tokio;

    struct MockPolling {
        poll_count: u32,
        delay: Duration,
    }

    impl FutureFactory for MockPolling {
        type Output = Instant;
        type Future<'a> = BoxFuture<'static, Instant>;

        fn new_future(&mut self) -> Self::Future<'_> {
            self.poll_count += 1;
            let delay = self.delay;
            async move {
                if delay > Duration::ZERO {
                    tokio::time::sleep(delay).await;
                }
                Instant::now()
            }
            .boxed()
        }
    }

    #[tokio::test]
    async fn test_polling_stream() {
        let emitter = MockPolling { poll_count: 0, delay: Duration::from_millis(200) };
        let interval = Duration::from_millis(500);
        let mut stream = PollingInterval::new(emitter, interval);
        let mut prev = stream.next().await.unwrap();
        for _ in 0..10 {
            let now = stream.next().await.unwrap();
            let elapsed = now - prev;
            prev = now;
            // Difference between the actual interval and the target interval should be less than
            // 50ms
            let difference = i64::try_from(elapsed.as_millis())
                .unwrap()
                .abs_diff(i64::try_from(interval.as_millis()).unwrap());
            assert!(difference < 50, "{difference} > 50");
        }
        assert_eq!(stream.factory().unwrap().poll_count, 11);
    }
}
