#![allow(clippy::missing_errors_doc)]
use std::{
    collections::VecDeque,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use futures_util::{future::BoxFuture, Future, FutureExt, Stream};
use rosetta_config_ethereum::types::config::{EthereumConfig, Query, QueryResult};
use rosetta_core::traits::{Client, ClientEvent, Config};
use rosetta_ethereum_backend::{
    AtBlock, EthereumPubSub, EthereumRpc,
    __reexports::primitives::{CallRequest, TxHash},
};

type QueryFuture<ERR> = IdentifiableFuture<u32, BoxFuture<'static, Result<QueryResult, ERR>>>;

pub struct EthereumClient<T>
where
    T: EthereumRpc + Send + Sync + 'static,
{
    rpc_client: Arc<T>,
    id_sequence: u32,
    pending_queries: VecDeque<(u32, Query)>,
    pending_request: Option<QueryFuture<T::Error>>,
    waker: Option<Waker>,
}

impl<T> EthereumClient<T>
where
    T: EthereumPubSub + Send + Sync + 'static,
{
    pub fn execute_query(
        &self,
        query: Query,
    ) -> futures_util::future::BoxFuture<'static, Result<QueryResult, T::Error>> {
        use rosetta_config_ethereum::types::queries::{
            CallContractQuery, CallResult, GetBalanceQuery, GetProofQuery, GetStorageAtQuery,
            GetTransactionReceiptQuery,
        };
        use rosetta_ethereum_backend::ExitReason;

        let rpc_client = Arc::clone(&self.rpc_client);
        match query {
            Query::GetBalance(GetBalanceQuery { address, block }) => async move {
                rpc_client
                    .get_balance(address, AtBlock::At(block))
                    .await
                    .map(QueryResult::GetBalance)
            }
            .boxed(),
            Query::GetStorageAt(GetStorageAtQuery { address, at, block }) => async move {
                rpc_client
                    .storage(address, at, AtBlock::At(block))
                    .await
                    .map(QueryResult::GetStorageAt)
            }
            .boxed(),
            Query::GetTransactionReceipt(GetTransactionReceiptQuery { tx_hash }) => async move {
                rpc_client
                    .transaction_receipt(tx_hash)
                    .await
                    .map(QueryResult::GetTransactionReceipt)
            }
            .boxed(),
            Query::CallContract(CallContractQuery { from, to, value, data, block }) => async move {
                let tx: CallRequest = CallRequest {
                    from,
                    to: Some(to),
                    gas_limit: None,
                    gas_price: None,
                    value: Some(value),
                    data: Some(data),
                    nonce: None,
                    chain_id: None,
                    max_priority_fee_per_gas: None,
                    access_list: Vec::with_capacity(0),
                    max_fee_per_gas: None,
                    transaction_type: None,
                };

                rpc_client
                    .call(&tx, AtBlock::At(block))
                    .await
                    .map(|exit_reason| match exit_reason {
                        ExitReason::Succeed(bytes) => CallResult::Success(bytes),
                        ExitReason::Revert(bytes) => CallResult::Revert(bytes),
                        ExitReason::Error(_) => CallResult::Error,
                    })
                    .map(QueryResult::CallContract)
            }
            .boxed(),
            Query::GetProof(GetProofQuery { account, storage_keys, block }) => async move {
                rpc_client
                    .get_proof(account, &storage_keys, AtBlock::At(block))
                    .await
                    .map(QueryResult::GetProof)
            }
            .boxed(),
        }
    }
}

impl<T> Client for EthereumClient<T>
where
    T: EthereumPubSub + Send + Sync + 'static,
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
        self.pending_queries.push_back((id, query));

        // Wake up the poll_next_event task
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
        Ok(id)
    }

    /// Internal function used by everything event-related.
    ///
    /// Polls the `Client` for the next event.
    fn poll_next_event(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<ClientEvent<Self>> {
        // We use a `this` variable because the compiler can't mutably borrow multiple times
        // across a `Deref`.
        let this = &mut *self;

        // Peek a request from the queue
        if this.pending_request.is_none() {
            if let Some((id, query)) = this.pending_queries.pop_front() {
                let future = this.execute_query(query);
                this.pending_request = Some(QueryFuture { id, future });
            }
        }

        if let Some(mut future) = this.pending_request.take() {
            match future.poll_unpin(cx) {
                Poll::Ready((id, result)) => {
                    let result = ClientEvent::<Self>::Query { id, result };
                    return Poll::Ready(result);
                },
                Poll::Pending => {
                    this.pending_request = Some(future);
                },
            }
        } else {
            this.waker.replace(cx.waker().clone());
        }

        Poll::Pending
    }
}

impl<T> Stream for EthereumClient<T>
where
    T: EthereumPubSub + Send + Sync + 'static,
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
