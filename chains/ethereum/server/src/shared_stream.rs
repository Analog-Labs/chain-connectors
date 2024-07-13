use futures_util::{future::Shared, Future, FutureExt, Stream, StreamExt};
use std::{
    pin::Pin,
    sync::{Arc, Weak},
    task::{Context, Poll},
};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

pub struct SharedStream<T>
where
    T: Stream + Unpin,
    T::Item: Clone + Send + Sync + 'static,
{
    inner: Inner<T>,
    stream: Option<BroadcastStream<<T as Stream>::Item>>,
}

impl<T> SharedStream<T>
where
    T: Stream + Unpin,
    T::Item: Clone + Send + Sync + 'static,
{
    #[must_use]
    pub fn new(stream: T, capacity: usize) -> Self {
        let (tx, rx) = tokio::sync::broadcast::channel::<<T as Stream>::Item>(capacity);
        let inner = Inner::new(stream, tx);
        Self { inner, stream: Some(BroadcastStream::new(rx)) }
    }
}

impl<T> Stream for SharedStream<T>
where
    T: Stream + Unpin,
    T::Item: Clone + Send + Sync + 'static,
{
    type Item = <T as Stream>::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Some(mut stream) = self.stream.take() else {
            panic!("stream polled after completion");
        };

        // Poll the transmitter
        match self.inner.future.poll_unpin(cx) {
            Poll::Ready(()) => return Poll::Ready(None),
            Poll::Pending => {},
        }

        // Poll the receiver
        loop {
            match stream.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(value))) => {
                    self.stream = Some(stream);
                    break Poll::Ready(Some(value));
                },
                Poll::Ready(Some(Err(value))) => match value {
                    BroadcastStreamRecvError::Lagged(gap) => {
                        tracing::warn!("broadcast stream lagged by {gap} messages");
                        continue;
                    },
                },
                Poll::Ready(None) => {
                    // Stream has ended
                    break Poll::Ready(None);
                },
                Poll::Pending => {
                    self.stream = Some(stream);
                    break Poll::Pending;
                },
            }
        }
    }
}

impl<T> Clone for SharedStream<T>
where
    T: Stream + Unpin,
    T::Item: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            stream: self.inner.outbound_channel.upgrade().map(|channel| {
                let receiver = channel.subscribe();
                BroadcastStream::new(receiver)
            }),
        }
    }
}

struct Inner<T>
where
    T: Stream + Unpin,
    T::Item: Clone + Send + Sync + 'static,
{
    future: Shared<BroadcastFuture<T>>,
    /// Map of listener IDs to their respective channels
    outbound_channel: Weak<tokio::sync::broadcast::Sender<<T as Stream>::Item>>,
}

impl<T> Inner<T>
where
    T: Stream + Unpin,
    T::Item: Clone + Send + Sync + 'static,
{
    #[must_use]
    pub fn new(
        stream: T,
        outbound_channel: tokio::sync::broadcast::Sender<<T as Stream>::Item>,
    ) -> Self {
        let outbound_channel = Arc::new(outbound_channel);
        let outbound_channel_ref = Arc::downgrade(&outbound_channel);
        let future = BroadcastFuture { stream, outbound_channel: Some(outbound_channel) };
        Self { future: future.shared(), outbound_channel: outbound_channel_ref }
    }
}

impl<T> Clone for Inner<T>
where
    T: Stream + Unpin,
    T::Item: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            future: Shared::clone(&self.future),
            outbound_channel: self.outbound_channel.clone(),
        }
    }
}

#[pin_project::pin_project]
struct BroadcastFuture<T>
where
    T: Stream + Unpin,
    T::Item: Clone + Send + Sync + 'static,
{
    #[pin]
    stream: T,
    /// Map of listener IDs to their respective channels
    outbound_channel: Option<Arc<tokio::sync::broadcast::Sender<<T as Stream>::Item>>>,
}

impl<T> Future for BroadcastFuture<T>
where
    T: Stream + Unpin,
    T::Item: Clone + Send + Sync + 'static,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        // Check if the stream has ended
        let Some(outbound_channel) = this.outbound_channel.take() else {
            panic!("future polled after completion");
        };

        // Poll the stream
        loop {
            match this.stream.poll_next_unpin(cx) {
                Poll::Ready(Some(value)) => {
                    // Broadcast the message to all listeners
                    // SAFETY: this should never happen, there must be always at least one listener
                    assert!(
                        outbound_channel.send(value).is_ok(),
                        "[report this bug] failed to broadcast message, no one is listening."
                    );
                },
                Poll::Ready(None) => {
                    // Stream has ended
                    break Poll::Ready(());
                },
                Poll::Pending => {
                    *this.outbound_channel = Some(outbound_channel);
                    break Poll::Pending;
                },
            }
        }
    }
}
