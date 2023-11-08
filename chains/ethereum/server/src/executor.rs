#![allow(clippy::missing_errors_doc)]
use std::sync::Arc;

use futures_util::future::{BoxFuture, FutureExt};
use rosetta_config_ethereum::types::config::{Query, QueryResult};
use rosetta_ethereum_backend::{
    AtBlock, EthereumRpc, ExitReason,
    __reexports::primitives::{AccessList, CallRequest},
};

pub trait QueryExecutor {
    type Error;

    fn execute(&self, query: Query) -> BoxFuture<'static, Result<QueryResult, Self::Error>>;
}

pub struct RpcQueryExecutor<T: EthereumRpc> {
    rpc_client: Arc<T>,
}

impl<T: EthereumRpc> RpcQueryExecutor<T> {
    pub fn new(rpc_client: Arc<T>) -> Self {
        Self { rpc_client }
    }

    #[must_use]
    pub fn rpc_client(&self) -> Arc<T> {
        Arc::clone(&self.rpc_client)
    }
}

impl<T> QueryExecutor for RpcQueryExecutor<T>
where
    T: EthereumRpc + Send + Sync + 'static,
{
    type Error = T::Error;

    fn execute(&self, query: Query) -> BoxFuture<'static, Result<QueryResult, Self::Error>> {
        let client = Arc::clone(&self.rpc_client);
        execute_query(client, query).boxed()
    }
}

async fn execute_query<T: EthereumRpc + Send + Sync>(
    client: Arc<T>,
    query: Query,
) -> Result<QueryResult, T::Error> {
    use rosetta_config_ethereum::types::queries::{
        CallContractQuery, CallResult, GetBalanceQuery, GetProofQuery, GetStorageAtQuery,
        GetTransactionReceiptQuery,
    };
    match query {
        Query::GetBalance(GetBalanceQuery { address, block }) => client
            .get_balance(address, AtBlock::At(block))
            .await
            .map(QueryResult::GetBalance),
        Query::GetStorageAt(GetStorageAtQuery { address, at, block }) => client
            .storage(address, at, AtBlock::At(block))
            .await
            .map(QueryResult::GetStorageAt),
        Query::GetTransactionReceipt(GetTransactionReceiptQuery { tx_hash }) => client
            .transaction_receipt(tx_hash)
            .await
            .map(QueryResult::GetTransactionReceipt),
        Query::CallContract(CallContractQuery { from, to, value, data, block }) => {
            let call = CallRequest {
                from,
                to: Some(to),
                gas_limit: None,
                gas_price: None,
                value: Some(value),
                data: Some(data),
                nonce: None,
                chain_id: None,
                max_priority_fee_per_gas: None,
                access_list: AccessList::default(),
                max_fee_per_gas: None,
                transaction_type: None,
            };
            client
                .call(&call, AtBlock::At(block))
                .await
                .map(|exit_reason| match exit_reason {
                    ExitReason::Succeed(bytes) => CallResult::Success(bytes),
                    ExitReason::Revert(bytes) => CallResult::Revert(bytes),
                    ExitReason::Error(_) => CallResult::Error,
                })
                .map(QueryResult::CallContract)
        },
        Query::GetProof(GetProofQuery { account, storage_keys, block }) => client
            .get_proof(account, storage_keys.as_ref(), AtBlock::At(block))
            .await
            .map(QueryResult::GetProof),
    }
}
