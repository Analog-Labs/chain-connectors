#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "jsonrpsee")]
pub mod jsonrpsee;

extern crate alloc;
use core::pin::Pin;

use alloc::{borrow::Cow, boxed::Box};
use futures_core::{future::BoxFuture, Future};
use rosetta_ethereum_types::{
    rpc::{CallRequest, RpcTransaction},
    AccessListWithGasUsed, Address, Block, BlockIdentifier, Bytes, EIP1186ProofResponse, Header,
    Log, TransactionReceipt, TxHash, H256, U256,
};

/// Re-exports for proc-macro library to not require any additional
/// dependencies to be explicitly added on the client side.
#[doc(hidden)]
pub mod __reexports {
    pub use async_trait::async_trait;
    #[cfg(feature = "with-codec")]
    pub use parity_scale_codec;
    pub use rosetta_ethereum_types as types;
    #[cfg(feature = "serde")]
    pub use serde;
}

/// Exit reason
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitReason {
    /// Machine has succeeded.
    Succeed(Bytes),
    /// Machine encountered an explicit revert.
    Revert(Bytes),
    /// Machine returns a normal EVM error.
    Error(Cow<'static, str>),
}

impl ExitReason {
    pub const fn bytes(&self) -> Option<&Bytes> {
        match self {
            Self::Succeed(bytes) | Self::Revert(bytes) => Some(bytes),
            Self::Error(_) => None,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
pub enum AtBlock {
    /// Latest block
    #[default]
    Latest,
    /// Finalized block accepted as canonical
    Finalized,
    /// Safe head block
    Safe,
    /// Earliest block (genesis)
    Earliest,
    /// Pending block (not yet part of the blockchain)
    Pending,
    /// Specific Block
    At(BlockIdentifier),
}

impl alloc::fmt::Display for AtBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Latest => f.write_str("latest"),
            Self::Finalized => f.write_str("finalized"),
            Self::Safe => f.write_str("safe"),
            Self::Earliest => f.write_str("earliest"),
            Self::Pending => f.write_str("ending"),
            Self::At(BlockIdentifier::Hash(hash)) => alloc::fmt::Display::fmt(&hash, f),
            Self::At(BlockIdentifier::Number(number)) => alloc::fmt::Display::fmt(&number, f),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for AtBlock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::Latest => <str as serde::Serialize>::serialize("latest", serializer),
            Self::Finalized => <str as serde::Serialize>::serialize("finalized", serializer),
            Self::Safe => <str as serde::Serialize>::serialize("safe", serializer),
            Self::Earliest => <str as serde::Serialize>::serialize("earliest", serializer),
            Self::Pending => <str as serde::Serialize>::serialize("pending", serializer),
            Self::At(at) => <BlockIdentifier as serde::Serialize>::serialize(at, serializer),
        }
    }
}

impl From<BlockIdentifier> for AtBlock {
    fn from(block: BlockIdentifier) -> Self {
        Self::At(block)
    }
}

/// EVM backend.
#[async_trait::async_trait]
#[auto_impl::auto_impl(&, Arc, Box)]
pub trait EthereumRpc {
    type Error: core::fmt::Display;

    /// Returns the balance of the account.
    async fn get_balance(&self, account: Address, at: AtBlock) -> Result<U256, Self::Error>;

    /// Returns the number of transactions sent from an address.
    async fn get_transaction_count(
        &self,
        account: Address,
        at: AtBlock,
    ) -> Result<u64, Self::Error>;

    /// Returns code at a given account
    async fn get_code(&self, address: Address, at: AtBlock) -> Result<Bytes, Self::Error>;

    /// Executes a new message call immediately without creating a transaction on the blockchain.
    fn call<'life0, 'life1, 'async_trait>(
        &'life0 self,
        tx: &'life1 CallRequest,
        at: AtBlock,
    ) -> Pin<Box<dyn Future<Output = Result<ExitReason, Self::Error>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait;

    /// Returns an estimate of how much gas is necessary to allow the transaction to complete.
    fn estimate_gas<'life0, 'life1, 'async_trait>(
        &'life0 self,
        tx: &'life1 CallRequest,
        at: AtBlock,
    ) -> Pin<Box<dyn Future<Output = Result<U256, Self::Error>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait;

    /// Returns the current gas price in wei.
    async fn gas_price(&self) -> Result<U256, Self::Error>;

    /// Submits a pre-signed transaction for broadcast to the Ethereum network.
    async fn send_raw_transaction(&self, tx: Bytes) -> Result<TxHash, Self::Error>;

    /// Returns the receipt of a transaction by transaction hash.
    async fn transaction_receipt(
        &self,
        tx: TxHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error>;

    /// Returns information about a transaction for a given hash.
    async fn transaction_by_hash(&self, tx: TxHash) -> Result<Option<RpcTransaction>, Self::Error>;

    /// Creates an EIP-2930 access list that you can include in a transaction.
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    fn create_access_list<'life0, 'life1, 'async_trait>(
        &'life0 self,
        tx: &'life1 CallRequest,
        at: AtBlock,
    ) -> Pin<
        Box<dyn Future<Output = Result<AccessListWithGasUsed, Self::Error>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait;

    /// Returns the account and storage values, including the Merkle proof, of the specified
    /// account.
    fn get_proof<'life0, 'life1, 'async_trait>(
        &'life0 self,
        address: Address,
        storage_keys: &'life1 [H256],
        at: AtBlock,
    ) -> Pin<
        Box<dyn Future<Output = Result<EIP1186ProofResponse, Self::Error>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait;

    /// Get storage value of address at index.
    async fn storage(
        &self,
        address: Address,
        index: H256,
        at: AtBlock,
    ) -> Result<H256, Self::Error>;

    /// Returns information about a block.
    async fn block(&self, at: AtBlock) -> Result<Option<Block<H256>>, Self::Error>;

    /// Returns information about a block.
    async fn block_full(
        &self,
        at: AtBlock,
    ) -> Result<Option<Block<RpcTransaction, Header>>, Self::Error>;

    /// Returns the currently configured chain ID, a value used in replay-protected
    /// transaction signing as introduced by EIP-155.
    async fn chain_id(&self) -> Result<u64, Self::Error>;
}

/// EVM backend.
#[async_trait::async_trait]
pub trait EthereumPubSub: EthereumRpc {
    type SubscriptionError: core::fmt::Display + Send + 'static;
    type NewHeadsStream<'a>: futures_core::Stream<Item = Result<Block<H256>, Self::SubscriptionError>>
        + Send
        + Unpin
        + 'a
    where
        Self: 'a;
    type LogsStream<'a>: futures_core::Stream<Item = Result<Log, Self::SubscriptionError>>
        + Send
        + Unpin
        + 'a
    where
        Self: 'a;

    /// Fires a notification each time a new header is appended to the chain, including chain
    /// reorganizations.
    /// Users can use the bloom filter to determine if the block contains logs that are interested
    /// to them. Note that if geth receives multiple blocks simultaneously, e.g. catching up after
    /// being out of sync, only the last block is emitted.
    async fn new_heads<'a>(&'a self) -> Result<Self::NewHeadsStream<'a>, Self::Error>;

    /// Returns logs that are included in new imported blocks and match the given filter criteria.
    /// In case of a chain reorganization previous sent logs that are on the old chain will be
    /// resent with the removed property set to true. Logs from transactions that ended up in
    /// the new chain are emitted. Therefore a subscription can emit logs for the same transaction
    /// multiple times.
    async fn logs<'a>(
        &'a self,
        contract: Address,
        topics: &[H256],
    ) -> Result<Self::LogsStream<'a>, Self::Error>;
}

impl<'b, T: 'b + EthereumPubSub + ?::core::marker::Sized> EthereumPubSub for &'b T {
    type SubscriptionError = T::SubscriptionError;
    type NewHeadsStream<'a> = T::NewHeadsStream<'a> where Self: 'a;
    type LogsStream<'a> = T::LogsStream<'a> where Self: 'a;
    fn new_heads<'a, 'async_trait>(
        &'a self,
    ) -> BoxFuture<'async_trait, Result<Self::NewHeadsStream<'a>, Self::Error>>
    where
        'a: 'async_trait,
        Self: 'async_trait,
    {
        T::new_heads(self)
    }
    fn logs<'a, 'life0, 'async_trait>(
        &'a self,
        contract: Address,
        topics: &'life0 [H256],
    ) -> BoxFuture<'async_trait, Result<Self::LogsStream<'a>, Self::Error>>
    where
        'a: 'async_trait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        T::logs(self, contract, topics)
    }
}

// #[auto_impl] doesn't work with generic associated types:
// https://github.com/auto-impl-rs/auto_impl/issues/93
impl<T: EthereumPubSub + ?::core::marker::Sized> EthereumPubSub for alloc::sync::Arc<T> {
    type SubscriptionError = T::SubscriptionError;
    type NewHeadsStream<'a> = T::NewHeadsStream<'a> where Self: 'a;
    type LogsStream<'a> = T::LogsStream<'a> where Self: 'a;

    fn new_heads<'a, 'async_trait>(
        &'a self,
    ) -> BoxFuture<'async_trait, Result<Self::NewHeadsStream<'a>, Self::Error>>
    where
        'a: 'async_trait,
        Self: 'async_trait,
    {
        T::new_heads(self)
    }
    fn logs<'a, 'life0, 'async_trait>(
        &'a self,
        contract: Address,
        topics: &'life0 [H256],
    ) -> BoxFuture<'async_trait, Result<T::LogsStream<'a>, T::Error>>
    where
        'a: 'async_trait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        T::logs(self, contract, topics)
    }
}

impl<T: EthereumPubSub + ?::core::marker::Sized> EthereumPubSub for alloc::boxed::Box<T> {
    type SubscriptionError = T::SubscriptionError;
    type NewHeadsStream<'a> = T::NewHeadsStream<'a> where Self: 'a;
    type LogsStream<'a> = T::LogsStream<'a> where Self: 'a;

    fn new_heads<'a, 'async_trait>(
        &'a self,
    ) -> BoxFuture<'async_trait, Result<T::NewHeadsStream<'a>, T::Error>>
    where
        'a: 'async_trait,
        Self: 'async_trait,
    {
        T::new_heads(self)
    }

    fn logs<'a, 'life0, 'async_trait>(
        &'a self,
        contract: Address,
        topics: &'life0 [H256],
    ) -> BoxFuture<'async_trait, Result<T::LogsStream<'a>, T::Error>>
    where
        'a: 'async_trait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        T::logs(self, contract, topics)
    }
}
