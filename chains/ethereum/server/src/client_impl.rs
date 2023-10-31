#![allow(clippy::missing_errors_doc)]
use std::{
    collections::VecDeque,
    sync::Arc,
    task::{Context, Poll},
};

use futures_util::{future::BoxFuture, FutureExt};
use rosetta_config_ethereum::types::config::{EthereumConfig, Query, QueryResult};
use rosetta_core::traits::{Client, ClientEvent, Config};
use rosetta_ethereum_backend::{
    AtBlock, EthereumRpc,
    __reexports::primitives::{TxHash, U256},
};

#[pin_project::pin_project]
pub struct EthereumClient<T>
where
    T: EthereumRpc + Send + Sync + 'static,
{
    pending_queries: VecDeque<(u32, Query)>,
    id_sequence: u32,
    rpc_methods: Arc<T>,
    pending_request: Option<BoxFuture<'static, Result<U256, T::Error>>>,
}

impl<T> Client for EthereumClient<T>
where
    T: EthereumRpc + Send + Sync + 'static,
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
        Ok(id)
    }

    fn poll_next_event(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<ClientEvent<Self>> {
        let this = self.project();

        if this.pending_request.is_none() {
            if let Some((_, Query::GetBalance(query))) = this.pending_queries.pop_front() {
                let rpc_methods = Arc::clone(this.rpc_methods);
                let future = async move {
                    rpc_methods.get_balance(query.address, AtBlock::from(query.block)).await
                }
                .boxed();
                *this.pending_request = Some(future);
            }
        }

        if let Some(mut future) = this.pending_request.take() {
            match future.poll_unpin(cx) {
                std::task::Poll::Ready(Ok(balance)) => {
                    let result = ClientEvent::<Self>::Query {
                        id: 1,
                        result: Result::Ok(QueryResult::GetBalance(balance)),
                    };
                    return Poll::Ready(result);
                },
                std::task::Poll::Ready(Err(error)) => {
                    let result = ClientEvent::<Self>::Query { id: 1, result: Result::Err(error) };
                    return Poll::Ready(result);
                },
                std::task::Poll::Pending => {
                    *this.pending_request = Some(future);
                },
            }
        }

        Poll::Pending
    }
}
