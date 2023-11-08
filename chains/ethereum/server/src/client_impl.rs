#![allow(clippy::missing_errors_doc)]
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::executor::QueryExecutor;
use futures_util::{
    future::BoxFuture, stream::FuturesUnordered, Future, FutureExt, Stream, StreamExt,
};
use rosetta_config_ethereum::types::config::{EthereumConfig, Query, QueryResult};
use rosetta_core::traits::{Client, ClientEvent, Config};
use rosetta_ethereum_backend::__reexports::primitives::TxHash;

type QueryFuture<ERR> = IdentifiableFuture<u32, BoxFuture<'static, Result<QueryResult, ERR>>>;

pub struct EthereumClient<T: QueryExecutor> {
    executor: T,
    id_sequence: u32,
    pending_requests: FuturesUnordered<QueryFuture<T::Error>>,
}

impl<T: QueryExecutor> EthereumClient<T> {
    pub fn new(executor: T) -> Self {
        Self { executor, id_sequence: 1, pending_requests: FuturesUnordered::new() }
    }
}

impl<T> Client for EthereumClient<T>
where
    T: QueryExecutor + Unpin,
    T::Error: std::error::Error,
{
    type Config = EthereumConfig;
    type TransactionId = TxHash;
    type QueryId = u32;
    type Error = T::Error;

    fn submit(
        &mut self,
        _tx: <Self::Config as Config>::Transaction,
    ) -> Result<TxHash, Self::Error> {
        unimplemented!()
    }

    fn query(&mut self, query: Query) -> Result<Self::QueryId, Self::Error> {
        self.id_sequence += 1;
        let id = self.id_sequence;
        let future = self.executor.execute(query);
        self.pending_requests.push(QueryFuture { id, future });
        Ok(id)
    }

    /// Internal function used by everything event-related.
    ///
    /// Polls the `Client` for the next event.
    fn poll_next_event<'a, 'b, 'c, 'd: 'a>(
        mut self: Pin<&'a mut Self>,
        cx: &'b mut Context<'c>,
    ) -> Poll<ClientEvent<Self>>
    where
        Self: 'd,
    {
        // We use a `this` variable because the compiler can't mutably borrow multiple times
        // across a `Deref`.
        let this = &mut *self;

        // Process pending requests
        if !this.pending_requests.is_empty() {
            match futures_util::ready!(this.pending_requests.poll_next_unpin(cx)) {
                Some((id, result)) => {
                    let result = ClientEvent::<Self>::Query { id, result };
                    return Poll::Ready(result);
                },
                None => return Poll::Pending,
            }
        }
        Poll::Pending
    }
}

impl<T> Stream for EthereumClient<T>
where
    T: QueryExecutor + Unpin,
    T::Error: std::error::Error,
{
    type Item = ClientEvent<Self>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.as_mut().poll_next_event(cx).map(Some)
    }
}

/// A Future which have an ID associated with it.
#[pin_project::pin_project]
struct IdentifiableFuture<T, F> {
    id: T,
    future: F,
}

impl<T, F> Future for IdentifiableFuture<T, F>
where
    F: Future + Unpin,
    T: Clone,
{
    type Output = (T, F::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.future.poll_unpin(cx) {
            Poll::Ready(output) => Poll::Ready((this.id.clone(), output)),
            Poll::Pending => Poll::Pending,
        }
    }
}
