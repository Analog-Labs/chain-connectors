#![allow(clippy::missing_errors_doc)]
use std::{
    collections::VecDeque,
    sync::Arc,
    task::{Context, Poll},
};

use futures_util::{future::BoxFuture, FutureExt};
use jsonrpsee::core::{client::ClientT, Error};
pub use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use rosetta_config_ethereum::types::{
    config::{EthereumConfig, Query, QueryResult},
    primitives::{Address, BlockIdentifier, Bytes, Call, TransactionReceipt, TxHash, U256},
};
use rosetta_core::traits::{Client, ClientEvent, Config};
use serde_json::value::RawValue;

/// Error type.
#[derive(Debug, thiserror::Error)]
pub enum EthereumRPCError {
    /// JSON-RPC error which can occur when a JSON-RPC call fails.
    #[error("{0}")]
    Call(ErrorObjectOwned),

    /// eth_call reverted with the given data.
    #[error("reverted")]
    Revert(Bytes),

    /// eth_call ran out of gas.
    #[error("out of gas")]
    OutOfGas,

    /// eth_call provided block not found
    #[error("header not found")]
    HeaderNotFound,

    /// eth_call parameters are invalid
    #[error("invalid call")]
    InvalidCall,

    /// Other unknown error.
    #[error("{0}")]
    Other(Error),
}

#[async_trait::async_trait]
pub trait EthereumRpcT {
    async fn get_balance(
        &self,
        account: Address,
        block: BlockIdentifier,
    ) -> Result<U256, EthereumRPCError>;

    async fn get_storage_at(
        &self,
        contract: Address,
        slot: U256,
        block: BlockIdentifier,
    ) -> Result<U256, EthereumRPCError>;

    async fn get_transaction_receipt(
        &self,
        tx_hash: TxHash,
    ) -> Result<TransactionReceipt, EthereumRPCError>;

    async fn call(&self, call: Call, block: BlockIdentifier) -> Result<Bytes, EthereumRPCError>;

    async fn get_code(
        &self,
        contract: Address,
        block: BlockIdentifier,
    ) -> Result<Bytes, EthereumRPCError>;
}

impl From<Error> for EthereumRPCError {
    fn from(error: Error) -> Self {
        match error {
            Error::Call(error) => Self::Call(error),
            error => Self::Other(error),
        }
    }
}

pub trait RpcErrorTransform {
    fn call(&self, error: ErrorObjectOwned) -> Result<Bytes, EthereumRPCError> {
        Err(EthereumRPCError::Call(error))
    }

    fn get_code(&self, error: ErrorObjectOwned) -> Result<Bytes, EthereumRPCError> {
        Err(EthereumRPCError::Call(error))
    }
}

pub struct GenericErrorTransform {
    pub revert_code: i32,
    pub header_not_found_code: i32,
    pub out_of_gas_code: i32,
}

impl GenericErrorTransform {
    pub fn parse_revert(
        &self,
        error: ErrorObjectOwned,
        check_message: bool,
    ) -> Result<EthereumRPCError, ErrorObjectOwned> {
        if error.code() != self.revert_code {
            return Err(error);
        }

        if check_message && !error.message().contains("revert") {
            return Err(error);
        }

        match error.data().map(RawValue::get).map(serde_json::from_str::<Bytes>) {
            // Revert with data
            Some(Ok(bytes)) => Ok(EthereumRPCError::Revert(bytes)),
            // Failed to parse revert data
            Some(Err(_)) => Err(error),
            // Revert without data
            None => Ok(EthereumRPCError::Revert(Bytes::new())),
        }
    }

    pub fn parse_header_not_found(
        &self,
        error: ErrorObjectOwned,
        check_message: bool,
    ) -> Result<EthereumRPCError, ErrorObjectOwned> {
        if error.code() != self.header_not_found_code {
            return Err(error);
        }

        if check_message && error.message() != "header not found" {
            return Err(error);
        }

        Ok(EthereumRPCError::HeaderNotFound)
    }

    pub fn parse_out_of_gas(
        &self,
        error: ErrorObjectOwned,
        check_message: bool,
    ) -> Result<EthereumRPCError, ErrorObjectOwned> {
        if error.code() != self.out_of_gas_code {
            return Err(error);
        }

        if check_message && !error.message().contains("out of gas") {
            return Err(error);
        }

        Ok(EthereumRPCError::OutOfGas)
    }
}

impl RpcErrorTransform for GenericErrorTransform {
    fn call(&self, error: ErrorObjectOwned) -> Result<Bytes, EthereumRPCError> {
        let check_message = self.out_of_gas_code == self.revert_code;

        // Revert
        let error = match self.parse_revert(error, check_message) {
            Ok(error) => return Err(error),
            Err(error) => error,
        };

        // Out of gas
        let error = match self.parse_out_of_gas(error, check_message) {
            Ok(error) => return Err(error),
            Err(error) => error,
        };

        // Header not found
        let error = match self.parse_header_not_found(error, check_message) {
            Ok(error) => return Err(error),
            Err(error) => error,
        };

        Err(EthereumRPCError::Call(error))
    }

    fn get_code(&self, error: ErrorObjectOwned) -> Result<Bytes, EthereumRPCError> {
        if error.code() == -32000 && error.message().starts_with("missing trie node") {
            Ok(Bytes::new())
        } else {
            Err(EthereumRPCError::Call(error))
        }
    }
}

pub struct EthereumRpc<T, H> {
    client: T,
    error_handler: H,
}

impl<T, H> EthereumRpc<T, H>
where
    T: ClientT + Send + Sync,
    H: RpcErrorTransform + Send + Sync,
{
    pub const fn new(client: T, error_handler: H) -> Self {
        Self { client, error_handler }
    }
}

#[pin_project::pin_project]
pub struct EthereumClient<T, H> {
    pending_queries: VecDeque<(u32, Query)>,
    id_sequence: u32,
    rpc_methods: Arc<EthereumRpc<T, H>>,
    pending_request: Option<BoxFuture<'static, Result<U256, EthereumRPCError>>>,
}

impl<T, H> Client for EthereumClient<T, H>
where
    T: ClientT + Send + Sync + 'static,
    H: RpcErrorTransform + Send + Sync + 'static,
{
    type Config = EthereumConfig;
    type TransactionId = TxHash;
    type QueryId = u32;
    type Error = EthereumRPCError;

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
                let future =
                    async move { rpc_methods.get_balance(query.address, query.block).await }
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
