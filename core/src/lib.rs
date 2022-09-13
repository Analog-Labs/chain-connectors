#![allow(
    missing_docs,
    trivial_casts,
    unused_variables,
    unused_mut,
    unused_imports,
    unused_extern_crates,
    non_camel_case_types
)]
#![allow(unused_imports)]

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::task::{Context, Poll};
use swagger::{ApiError, ContextWrapper};

type ServiceError = Box<dyn Error + Send + Sync + 'static>;

pub const BASE_PATH: &str = "";
pub const API_VERSION: &str = "1.4.13";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum AccountBalanceResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::AccountBalanceResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum AccountCoinsResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::AccountCoinsResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum BlockResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::BlockResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum BlockTransactionResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::BlockTransactionResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum CallResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::CallResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ConstructionCombineResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::ConstructionCombineResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ConstructionDeriveResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::ConstructionDeriveResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ConstructionHashResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::TransactionIdentifierResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ConstructionMetadataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::ConstructionMetadataResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ConstructionParseResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::ConstructionParseResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ConstructionPayloadsResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::ConstructionPayloadsResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ConstructionPreprocessResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::ConstructionPreprocessResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum ConstructionSubmitResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::TransactionIdentifierResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum EventsBlocksResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::EventsBlocksResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum MempoolResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::MempoolResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum MempoolTransactionResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::MempoolTransactionResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum NetworkListResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::NetworkListResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum NetworkOptionsResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::NetworkOptionsResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum NetworkStatusResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::NetworkStatusResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum SearchTransactionsResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::SearchTransactionsResponse),
    /// unexpected error
    UnexpectedError(models::Error),
}

/// API
#[async_trait]
pub trait Api<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    /// Get an Account's Balance
    async fn account_balance(
        &self,
        account_balance_request: models::AccountBalanceRequest,
        context: &C,
    ) -> Result<AccountBalanceResponse, ApiError>;

    /// Get an Account's Unspent Coins
    async fn account_coins(
        &self,
        account_coins_request: models::AccountCoinsRequest,
        context: &C,
    ) -> Result<AccountCoinsResponse, ApiError>;

    /// Get a Block
    async fn block(
        &self,
        block_request: models::BlockRequest,
        context: &C,
    ) -> Result<BlockResponse, ApiError>;

    /// Get a Block Transaction
    async fn block_transaction(
        &self,
        block_transaction_request: models::BlockTransactionRequest,
        context: &C,
    ) -> Result<BlockTransactionResponse, ApiError>;

    /// Make a Network-Specific Procedure Call
    async fn call(
        &self,
        call_request: models::CallRequest,
        context: &C,
    ) -> Result<CallResponse, ApiError>;

    /// Create Network Transaction from Signatures
    async fn construction_combine(
        &self,
        construction_combine_request: models::ConstructionCombineRequest,
        context: &C,
    ) -> Result<ConstructionCombineResponse, ApiError>;

    /// Derive an AccountIdentifier from a PublicKey
    async fn construction_derive(
        &self,
        construction_derive_request: models::ConstructionDeriveRequest,
        context: &C,
    ) -> Result<ConstructionDeriveResponse, ApiError>;

    /// Get the Hash of a Signed Transaction
    async fn construction_hash(
        &self,
        construction_hash_request: models::ConstructionHashRequest,
        context: &C,
    ) -> Result<ConstructionHashResponse, ApiError>;

    /// Get Metadata for Transaction Construction
    async fn construction_metadata(
        &self,
        construction_metadata_request: models::ConstructionMetadataRequest,
        context: &C,
    ) -> Result<ConstructionMetadataResponse, ApiError>;

    /// Parse a Transaction
    async fn construction_parse(
        &self,
        construction_parse_request: models::ConstructionParseRequest,
        context: &C,
    ) -> Result<ConstructionParseResponse, ApiError>;

    /// Generate an Unsigned Transaction and Signing Payloads
    async fn construction_payloads(
        &self,
        construction_payloads_request: models::ConstructionPayloadsRequest,
        context: &C,
    ) -> Result<ConstructionPayloadsResponse, ApiError>;

    /// Create a Request to Fetch Metadata
    async fn construction_preprocess(
        &self,
        construction_preprocess_request: models::ConstructionPreprocessRequest,
        context: &C,
    ) -> Result<ConstructionPreprocessResponse, ApiError>;

    /// Submit a Signed Transaction
    async fn construction_submit(
        &self,
        construction_submit_request: models::ConstructionSubmitRequest,
        context: &C,
    ) -> Result<ConstructionSubmitResponse, ApiError>;

    /// [INDEXER] Get a range of BlockEvents
    async fn events_blocks(
        &self,
        events_blocks_request: models::EventsBlocksRequest,
        context: &C,
    ) -> Result<EventsBlocksResponse, ApiError>;

    /// Get All Mempool Transactions
    async fn mempool(
        &self,
        network_request: models::NetworkRequest,
        context: &C,
    ) -> Result<MempoolResponse, ApiError>;

    /// Get a Mempool Transaction
    async fn mempool_transaction(
        &self,
        mempool_transaction_request: models::MempoolTransactionRequest,
        context: &C,
    ) -> Result<MempoolTransactionResponse, ApiError>;

    /// Get List of Available Networks
    async fn network_list(
        &self,
        metadata_request: models::MetadataRequest,
        context: &C,
    ) -> Result<NetworkListResponse, ApiError>;

    /// Get Network Options
    async fn network_options(
        &self,
        network_request: models::NetworkRequest,
        context: &C,
    ) -> Result<NetworkOptionsResponse, ApiError>;

    /// Get Network Status
    async fn network_status(
        &self,
        network_request: models::NetworkRequest,
        context: &C,
    ) -> Result<NetworkStatusResponse, ApiError>;

    /// [INDEXER] Search for Transactions
    async fn search_transactions(
        &self,
        search_transactions_request: models::SearchTransactionsRequest,
        context: &C,
    ) -> Result<SearchTransactionsResponse, ApiError>;
}

/// API where `Context` isn't passed on every API call
#[async_trait]
pub trait ApiNoContext<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    /// Get an Account's Balance
    async fn account_balance(
        &self,
        account_balance_request: models::AccountBalanceRequest,
    ) -> Result<AccountBalanceResponse, ApiError>;

    /// Get an Account's Unspent Coins
    async fn account_coins(
        &self,
        account_coins_request: models::AccountCoinsRequest,
    ) -> Result<AccountCoinsResponse, ApiError>;

    /// Get a Block
    async fn block(&self, block_request: models::BlockRequest) -> Result<BlockResponse, ApiError>;

    /// Get a Block Transaction
    async fn block_transaction(
        &self,
        block_transaction_request: models::BlockTransactionRequest,
    ) -> Result<BlockTransactionResponse, ApiError>;

    /// Make a Network-Specific Procedure Call
    async fn call(&self, call_request: models::CallRequest) -> Result<CallResponse, ApiError>;

    /// Create Network Transaction from Signatures
    async fn construction_combine(
        &self,
        construction_combine_request: models::ConstructionCombineRequest,
    ) -> Result<ConstructionCombineResponse, ApiError>;

    /// Derive an AccountIdentifier from a PublicKey
    async fn construction_derive(
        &self,
        construction_derive_request: models::ConstructionDeriveRequest,
    ) -> Result<ConstructionDeriveResponse, ApiError>;

    /// Get the Hash of a Signed Transaction
    async fn construction_hash(
        &self,
        construction_hash_request: models::ConstructionHashRequest,
    ) -> Result<ConstructionHashResponse, ApiError>;

    /// Get Metadata for Transaction Construction
    async fn construction_metadata(
        &self,
        construction_metadata_request: models::ConstructionMetadataRequest,
    ) -> Result<ConstructionMetadataResponse, ApiError>;

    /// Parse a Transaction
    async fn construction_parse(
        &self,
        construction_parse_request: models::ConstructionParseRequest,
    ) -> Result<ConstructionParseResponse, ApiError>;

    /// Generate an Unsigned Transaction and Signing Payloads
    async fn construction_payloads(
        &self,
        construction_payloads_request: models::ConstructionPayloadsRequest,
    ) -> Result<ConstructionPayloadsResponse, ApiError>;

    /// Create a Request to Fetch Metadata
    async fn construction_preprocess(
        &self,
        construction_preprocess_request: models::ConstructionPreprocessRequest,
    ) -> Result<ConstructionPreprocessResponse, ApiError>;

    /// Submit a Signed Transaction
    async fn construction_submit(
        &self,
        construction_submit_request: models::ConstructionSubmitRequest,
    ) -> Result<ConstructionSubmitResponse, ApiError>;

    /// [INDEXER] Get a range of BlockEvents
    async fn events_blocks(
        &self,
        events_blocks_request: models::EventsBlocksRequest,
    ) -> Result<EventsBlocksResponse, ApiError>;

    /// Get All Mempool Transactions
    async fn mempool(
        &self,
        network_request: models::NetworkRequest,
    ) -> Result<MempoolResponse, ApiError>;

    /// Get a Mempool Transaction
    async fn mempool_transaction(
        &self,
        mempool_transaction_request: models::MempoolTransactionRequest,
    ) -> Result<MempoolTransactionResponse, ApiError>;

    /// Get List of Available Networks
    async fn network_list(
        &self,
        metadata_request: models::MetadataRequest,
    ) -> Result<NetworkListResponse, ApiError>;

    /// Get Network Options
    async fn network_options(
        &self,
        network_request: models::NetworkRequest,
    ) -> Result<NetworkOptionsResponse, ApiError>;

    /// Get Network Status
    async fn network_status(
        &self,
        network_request: models::NetworkRequest,
    ) -> Result<NetworkStatusResponse, ApiError>;

    /// [INDEXER] Search for Transactions
    async fn search_transactions(
        &self,
        search_transactions_request: models::SearchTransactionsRequest,
    ) -> Result<SearchTransactionsResponse, ApiError>;
}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<C: Send + Sync>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
        ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ApiNoContext<C> for ContextWrapper<T, C> {
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
        self.api().poll_ready(cx)
    }

    fn context(&self) -> &C {
        ContextWrapper::context(self)
    }

    /// Get an Account's Balance
    async fn account_balance(
        &self,
        account_balance_request: models::AccountBalanceRequest,
    ) -> Result<AccountBalanceResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .account_balance(account_balance_request, &context)
            .await
    }

    /// Get an Account's Unspent Coins
    async fn account_coins(
        &self,
        account_coins_request: models::AccountCoinsRequest,
    ) -> Result<AccountCoinsResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .account_coins(account_coins_request, &context)
            .await
    }

    /// Get a Block
    async fn block(&self, block_request: models::BlockRequest) -> Result<BlockResponse, ApiError> {
        let context = self.context().clone();
        self.api().block(block_request, &context).await
    }

    /// Get a Block Transaction
    async fn block_transaction(
        &self,
        block_transaction_request: models::BlockTransactionRequest,
    ) -> Result<BlockTransactionResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .block_transaction(block_transaction_request, &context)
            .await
    }

    /// Make a Network-Specific Procedure Call
    async fn call(&self, call_request: models::CallRequest) -> Result<CallResponse, ApiError> {
        let context = self.context().clone();
        self.api().call(call_request, &context).await
    }

    /// Create Network Transaction from Signatures
    async fn construction_combine(
        &self,
        construction_combine_request: models::ConstructionCombineRequest,
    ) -> Result<ConstructionCombineResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .construction_combine(construction_combine_request, &context)
            .await
    }

    /// Derive an AccountIdentifier from a PublicKey
    async fn construction_derive(
        &self,
        construction_derive_request: models::ConstructionDeriveRequest,
    ) -> Result<ConstructionDeriveResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .construction_derive(construction_derive_request, &context)
            .await
    }

    /// Get the Hash of a Signed Transaction
    async fn construction_hash(
        &self,
        construction_hash_request: models::ConstructionHashRequest,
    ) -> Result<ConstructionHashResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .construction_hash(construction_hash_request, &context)
            .await
    }

    /// Get Metadata for Transaction Construction
    async fn construction_metadata(
        &self,
        construction_metadata_request: models::ConstructionMetadataRequest,
    ) -> Result<ConstructionMetadataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .construction_metadata(construction_metadata_request, &context)
            .await
    }

    /// Parse a Transaction
    async fn construction_parse(
        &self,
        construction_parse_request: models::ConstructionParseRequest,
    ) -> Result<ConstructionParseResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .construction_parse(construction_parse_request, &context)
            .await
    }

    /// Generate an Unsigned Transaction and Signing Payloads
    async fn construction_payloads(
        &self,
        construction_payloads_request: models::ConstructionPayloadsRequest,
    ) -> Result<ConstructionPayloadsResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .construction_payloads(construction_payloads_request, &context)
            .await
    }

    /// Create a Request to Fetch Metadata
    async fn construction_preprocess(
        &self,
        construction_preprocess_request: models::ConstructionPreprocessRequest,
    ) -> Result<ConstructionPreprocessResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .construction_preprocess(construction_preprocess_request, &context)
            .await
    }

    /// Submit a Signed Transaction
    async fn construction_submit(
        &self,
        construction_submit_request: models::ConstructionSubmitRequest,
    ) -> Result<ConstructionSubmitResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .construction_submit(construction_submit_request, &context)
            .await
    }

    /// [INDEXER] Get a range of BlockEvents
    async fn events_blocks(
        &self,
        events_blocks_request: models::EventsBlocksRequest,
    ) -> Result<EventsBlocksResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .events_blocks(events_blocks_request, &context)
            .await
    }

    /// Get All Mempool Transactions
    async fn mempool(
        &self,
        network_request: models::NetworkRequest,
    ) -> Result<MempoolResponse, ApiError> {
        let context = self.context().clone();
        self.api().mempool(network_request, &context).await
    }

    /// Get a Mempool Transaction
    async fn mempool_transaction(
        &self,
        mempool_transaction_request: models::MempoolTransactionRequest,
    ) -> Result<MempoolTransactionResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .mempool_transaction(mempool_transaction_request, &context)
            .await
    }

    /// Get List of Available Networks
    async fn network_list(
        &self,
        metadata_request: models::MetadataRequest,
    ) -> Result<NetworkListResponse, ApiError> {
        let context = self.context().clone();
        self.api().network_list(metadata_request, &context).await
    }

    /// Get Network Options
    async fn network_options(
        &self,
        network_request: models::NetworkRequest,
    ) -> Result<NetworkOptionsResponse, ApiError> {
        let context = self.context().clone();
        self.api().network_options(network_request, &context).await
    }

    /// Get Network Status
    async fn network_status(
        &self,
        network_request: models::NetworkRequest,
    ) -> Result<NetworkStatusResponse, ApiError> {
        let context = self.context().clone();
        self.api().network_status(network_request, &context).await
    }

    /// [INDEXER] Search for Transactions
    async fn search_transactions(
        &self,
        search_transactions_request: models::SearchTransactionsRequest,
    ) -> Result<SearchTransactionsResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .search_transactions(search_transactions_request, &context)
            .await
    }
}

#[cfg(feature = "client")]
pub mod client;

// Re-export Client as a top-level name
#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub mod server;

// Re-export router() as a top-level name
#[cfg(feature = "server")]
pub use self::server::Service;

#[cfg(feature = "server")]
pub mod context;

pub mod models;

#[cfg(any(feature = "client", feature = "server"))]
pub(crate) mod header;
