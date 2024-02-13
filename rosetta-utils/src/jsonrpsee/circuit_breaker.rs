use futures_util::{Stream, TryStream};
use pin_project::pin_project;
use std::{pin::Pin, task::{Context, Poll}};

pub enum Action {
    Ignore,
    Return,
    Terminate,
}

pub trait ErrorHandler<E> {
    fn on_error(&mut self, error: &E) -> Action;
}

impl<E, F> ErrorHandler<E> for F
where
    E: Send,
    F: FnMut(&E) -> Action,
{
    fn on_error(&mut self, error: &E) -> Action {
        self(error)
    }
}

impl<E> ErrorHandler<E> for () {
    fn on_error(&mut self, _error: &E) -> Action {
        Action::Return
    }
}

/// Polls a future at a fixed interval, adjusting the interval based on the actual time it took to
/// complete the future.
#[pin_project]
pub struct CircuitBreaker<S: TryStream, H: ErrorHandler<S::Error>> {
    #[pin]
    stream: S,
    handler: H,
    last_error: Option<S::Error>,
    error_threshold: u32,
    consecutive_errors: u32,
}

impl<S: TryStream, H: ErrorHandler<S::Error>> CircuitBreaker<S, H> {
    #[must_use]
    pub const fn new(stream: S, threshold: u32, handler: H) -> Self {
        Self {
            stream,
            handler,
            error_threshold: threshold,
            consecutive_errors: 0,
            last_error: None,
        }
    }

    #[must_use]
    pub const fn inner(&self) -> &S {
        &self.stream
    }

    #[must_use]
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    pub fn into_inner(self) -> (S, H) {
        (self.stream, self.handler)
    }
}

impl<S: TryStream, E: ErrorHandler<S::Error>> Stream for CircuitBreaker<S, E> {
    type Item = Result<S::Ok, S::Error>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            // If the number of consecutive errors exceeds the threshold, return None
            if *this.consecutive_errors >= *this.error_threshold {
                return Poll::Ready(None);
            }
            let Poll::Ready(result) = TryStream::try_poll_next(this.stream.as_mut(), cx) else {
                return Poll::Pending;
            };
            let Some(result) = result else {
                *this.consecutive_errors = *this.error_threshold;
                return Poll::Ready(None);
            };
            match result {
                Ok(value) => {
                    *this.consecutive_errors = 0;
                    return Poll::Ready(Some(Ok(value)));
                },
                Err(error) => match this.handler.on_error(&error) {
                    Action::Ignore => {
                        *this.consecutive_errors += 1;
                        *this.last_error = Some(error);
                    },
                    Action::Return => {
                        *this.consecutive_errors += 1;
                        return Poll::Ready(Some(Err(error)));
                    },
                    Action::Terminate => {
                        *this.consecutive_errors = *this.error_threshold;
                        return Poll::Ready(Some(Err(error)));
                    },
                },
            }
        }
    }
}
