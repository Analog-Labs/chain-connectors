#![cfg_attr(not(feature = "std"), no_std)]

mod block_range;

#[cfg(feature = "jsonrpsee")]
pub mod jsonrpsee;

#[cfg(feature = "serde")]
pub mod serde_util;

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

use async_trait::async_trait;
pub use block_range::{BlockRange, FilterBlockOption};
use futures_core::{future::BoxFuture, Stream};
use rosetta_ethereum_types::{
    rpc::{CallRequest, RpcBlock, RpcTransaction},
    AccessListWithGasUsed, Address, AtBlock, Bytes, EIP1186ProofResponse, FeeHistory, Log,
    SealedHeader, TransactionReceipt, TxHash, H256, U256,
};

/// Re-exports for proc-macro library to not require any additional
/// dependencies to be explicitly added on the client side.
#[doc(hidden)]
pub mod ext {
    pub use async_trait::async_trait;
    #[cfg(feature = "with-codec")]
    pub use parity_scale_codec;
    pub use rosetta_ethereum_types as types;
    #[cfg(feature = "serde")]
    pub use serde;
}

#[cfg(feature = "std")]
pub(crate) mod rstd {
    #[cfg(feature = "jsonrpsee")]
    pub use std::{ops, string};

    pub mod vec {
        pub use std::vec::*;
    }

    pub mod sync {
        pub use std::sync::Arc;
    }
    pub use std::{borrow, boxed, fmt, marker, str};
}

#[cfg(not(feature = "std"))]
pub(crate) mod rstd {
    #[cfg(feature = "jsonrpsee")]
    pub use alloc::string;

    #[cfg(feature = "jsonrpsee")]
    pub use core::ops;

    pub mod vec {
        pub use alloc::vec::*;
    }

    pub mod sync {
        pub use alloc::sync::Arc;
    }
    pub use alloc::{borrow, boxed, fmt};
    pub use core::{marker, str};
}

use rstd::{
    borrow::Cow,
    boxed::Box,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    marker::Sized,
    str,
    sync::Arc,
    vec::Vec,
};

/// Exit reason
#[derive(Clone, PartialEq, Eq)]
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

    /// Returns the revert message if the revert data is encoded as Error(string)
    pub fn revert_msg(&self) -> Option<&str> {
        let Self::Revert(bytes) = self else {
            return None;
        };
        let bytes = bytes.as_ref();
        // Check if the revert message starts with the selector for `Error(string)`
        if bytes.len() <= 68 || !bytes.starts_with(&[0x08, 0xc3, 0x79, 0xa0]) {
            return None;
        }
        // Check if the length of the string is valid
        let offset = usize::try_from(U256::from_big_endian(&bytes[4..36])).ok()? + 36;
        let len = usize::try_from(U256::from_big_endian(&bytes[36..68])).ok()?;
        if bytes.len() < (offset + len) {
            return None;
        }
        // Try to convert the bytes to a string
        str::from_utf8(&bytes[offset..offset + len]).ok()
    }
}

impl Debug for ExitReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(revert_msg) = self.revert_msg() {
            return f.debug_tuple("Revert").field(&revert_msg).finish();
        }
        match self {
            Self::Succeed(arg0) => f.debug_tuple("Succeed").field(arg0).finish(),
            Self::Revert(arg0) => f.debug_tuple("Revert").field(arg0).finish(),
            Self::Error(arg0) => f.debug_tuple("Error").field(arg0).finish(),
        }
    }
}

#[cfg(feature = "serde")]
pub trait MaybeDeserializeOwned: serde::de::DeserializeOwned {}
#[cfg(feature = "serde")]
impl<T: serde::de::DeserializeOwned> MaybeDeserializeOwned for T {}

#[cfg(not(feature = "serde"))]
pub trait MaybeDeserializeOwned {}
#[cfg(not(feature = "serde"))]
impl<T> MaybeDeserializeOwned for T {}

/// EVM backend.
#[async_trait]
#[auto_impl::auto_impl(&, Arc, Box)]
pub trait EthereumRpc {
    type Error: Display;

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

    /// Returns code at a given account
    async fn get_logs(&self, range: BlockRange) -> Result<Vec<Log>, Self::Error>;

    /// Executes a new message call immediately without creating a transaction on the blockchain.
    fn call<'life0, 'life1, 'async_trait>(
        &'life0 self,
        tx: &'life1 CallRequest,
        at: AtBlock,
    ) -> BoxFuture<'async_trait, Result<ExitReason, Self::Error>>
    where
        'life0: 'async_trait,
        Self: 'async_trait;

    /// Returns an estimate of how much gas is necessary to allow the transaction to complete.
    fn estimate_gas<'life0, 'life1, 'async_trait>(
        &'life0 self,
        tx: &'life1 CallRequest,
        at: AtBlock,
    ) -> BoxFuture<'async_trait, Result<U256, Self::Error>>
    where
        'life0: 'async_trait,
        Self: 'async_trait;

    /// Returns the current gas price in wei.
    async fn gas_price(&self) -> Result<U256, Self::Error>;

    /// Submits a pre-signed transaction for broadcast to the Ethereum network.
    async fn send_raw_transaction(&self, tx: Bytes) -> Result<TxHash, Self::Error>;

    /// Submits an unsigned transaction which will be signed by the node
    fn send_transaction<'life0, 'life1, 'async_trait>(
        &'life0 self,
        tx: &'life1 CallRequest,
    ) -> BoxFuture<'async_trait, Result<TxHash, Self::Error>>
    where
        'life0: 'async_trait,
        Self: 'async_trait;

    /// Returns the receipt of a transaction by transaction hash.
    async fn transaction_receipt(
        &self,
        tx: TxHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error>;

    /// Returns information about a transaction for a given hash.
    async fn transaction_by_hash(&self, tx: TxHash) -> Result<Option<RpcTransaction>, Self::Error>;

    /// Creates an EIP-2930 access list that you can include in a transaction.
    /// [EIP-2930]: <https://eips.ethereum.org/EIPS/eip-2930>
    fn create_access_list<'life0, 'life1, 'async_trait>(
        &'life0 self,
        tx: &'life1 CallRequest,
        at: AtBlock,
    ) -> BoxFuture<'async_trait, Result<AccessListWithGasUsed, Self::Error>>
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
    ) -> BoxFuture<'async_trait, Result<EIP1186ProofResponse, Self::Error>>
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
    async fn block(&self, at: AtBlock) -> Result<Option<RpcBlock<H256, H256>>, Self::Error>;

    /// Returns information about a block.
    async fn block_full<T: MaybeDeserializeOwned + Send>(
        &self,
        at: AtBlock,
    ) -> Result<Option<RpcBlock<T, H256>>, Self::Error>;

    /// Returns the current latest block number.
    async fn block_number(&self) -> Result<u64, Self::Error>;

    /// Returns information about a uncle of a block given the block hash and the uncle index
    /// position.
    async fn uncle_by_blockhash(
        &self,
        block_hash: H256,
        index: u32,
    ) -> Result<Option<SealedHeader>, Self::Error>;

    /// Returns the currently configured chain ID, a value used in replay-protected
    /// transaction signing as introduced by EIP-155.
    async fn chain_id(&self) -> Result<u64, Self::Error>;

    /// Returns a list of addresses owned by client.
    async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error>;

    /// Returns historical gas information, allowing you to track trends over time.
    async fn fee_history(
        &self,
        block_count: u64,
        last_block: AtBlock,
        reward_percentiles: &[f64],
    ) -> Result<FeeHistory, Self::Error>;
}

/// EVM backend.
#[async_trait]
pub trait EthereumPubSub: EthereumRpc {
    type SubscriptionError: Display + Send + 'static;
    type NewHeadsStream<'a>: Stream<Item = Result<RpcBlock<H256>, Self::SubscriptionError>>
        + Send
        + Unpin
        + 'a
    where
        Self: 'a;
    type LogsStream<'a>: Stream<Item = Result<Log, Self::SubscriptionError>> + Send + Unpin + 'a
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

impl<'b, T: 'b + EthereumPubSub + ?Sized> EthereumPubSub for &'b T {
    type SubscriptionError = T::SubscriptionError;
    type NewHeadsStream<'a>
        = T::NewHeadsStream<'a>
    where
        Self: 'a;
    type LogsStream<'a>
        = T::LogsStream<'a>
    where
        Self: 'a;
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
impl<T: EthereumPubSub + ?Sized> EthereumPubSub for Arc<T> {
    type SubscriptionError = T::SubscriptionError;
    type NewHeadsStream<'a>
        = T::NewHeadsStream<'a>
    where
        Self: 'a;
    type LogsStream<'a>
        = T::LogsStream<'a>
    where
        Self: 'a;

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

impl<T: EthereumPubSub + ?Sized> EthereumPubSub for Box<T> {
    type SubscriptionError = T::SubscriptionError;
    type NewHeadsStream<'a>
        = T::NewHeadsStream<'a>
    where
        Self: 'a;
    type LogsStream<'a>
        = T::LogsStream<'a>
    where
        Self: 'a;

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

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    #[test]
    fn test_exit_reason_revert_msg() {
        use super::ExitReason;
        let revert = ExitReason::Revert(hex!("08c379a000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000012736f6d657468696e672069732077726f6e670000000000000000000000000000").into());
        assert_eq!(revert.revert_msg(), Some("something is wrong"));
        assert_eq!(format!("{revert:?}"), "Revert(\"something is wrong\")");
    }
}
