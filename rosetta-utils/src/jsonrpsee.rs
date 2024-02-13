mod auto_subscribe;
mod circuit_breaker;
mod polling_interval;

pub use auto_subscribe::AutoSubscribe;
pub use circuit_breaker::{CircuitBreaker, ErrorHandler};
use futures_util::Future;
pub use polling_interval::PollingInterval;

pub trait FutureFactory: Send + 'static {
    type Output: Send + Sync + 'static;
    type Future<'a>: Future<Output = Self::Output> + Send;
    fn new_future(&mut self) -> Self::Future<'_>;
}

impl<R, Fut, F> FutureFactory for F
where
    R: Send + Sync + 'static,
    for<'a> Fut: Future<Output = R> + Send + 'a,
    F: FnMut() -> Fut + Send + Sync + 'static,
{
    type Output = <Fut as Future>::Output;
    type Future<'a> = Fut;
    fn new_future(&mut self) -> Self::Future<'_> {
        self()
    }
}
