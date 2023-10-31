#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "jsonrpsee")]
pub mod jsonrpsee;
mod transaction;

extern crate alloc;
use alloc::{borrow::Cow, boxed::Box, string::String, vec::Vec};
use rosetta_ethereum_primitives::{
    Address, Block, BlockIdentifier, Bytes, EIP1186ProofResponse, TransactionReceipt, TxHash, H256,
    U256, U64,
};
pub use transaction::TransactionCall;

/// Re-exports for proc-macro library to not require any additional
/// dependencies to be explicitly added on the client side.
#[doc(hidden)]
pub mod __reexports {
    pub use async_trait::async_trait;
    #[cfg(feature = "with-codec")]
    pub use parity_scale_codec;
    pub use rosetta_ethereum_primitives as primitives;
    #[cfg(feature = "with-serde")]
    pub use serde;
}

/// Exit reason
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
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

#[cfg(feature = "with-serde")]
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

/// Access list item
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct AccessListItem {
    /// Accessed address
    pub address: Address,
    /// Accessed storage keys
    pub storage_keys: Vec<H256>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct AccessListWithGasUsed {
    pub access_list: Vec<AccessListItem>,
    pub gas_used: U256,
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub error: Option<String>,
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
    ) -> Result<U64, Self::Error>;

    /// Returns code at a given account
    async fn get_code(&self, address: Address, at: AtBlock) -> Result<Bytes, Self::Error>;

    /// Executes a new message call immediately without creating a transaction on the blockchain.
    async fn call(&self, tx: &TransactionCall, at: AtBlock) -> Result<ExitReason, Self::Error>;

    /// Returns an estimate of how much gas is necessary to allow the transaction to complete.
    async fn estimate_gas(&self, tx: &TransactionCall, at: AtBlock) -> Result<U256, Self::Error>;

    /// Returns the current gas price in wei.
    async fn gas_price(&self) -> Result<U256, Self::Error>;

    /// Submits a pre-signed transaction for broadcast to the Ethereum network.
    async fn send_raw_transaction(&self, tx: Bytes) -> Result<TxHash, Self::Error>;

    /// Returns the receipt of a transaction by transaction hash.
    async fn transaction_receipt(
        &self,
        tx: TxHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error>;

    /// Creates an EIP-2930 access list that you can include in a transaction.
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    async fn create_access_list(
        &self,
        tx: &TransactionCall,
        at: AtBlock,
    ) -> Result<AccessListWithGasUsed, Self::Error>;

    /// Returns the account and storage values, including the Merkle proof, of the specified
    /// account.
    async fn get_proof(
        &self,
        address: Address,
        storage_keys: &[H256],
        at: AtBlock,
    ) -> Result<EIP1186ProofResponse, Self::Error>;

    /// Get storage value of address at index.
    async fn storage(
        &self,
        address: Address,
        index: H256,
        at: AtBlock,
    ) -> Result<H256, Self::Error>;

    /// Returns information about a block.
    async fn block(&self, at: AtBlock) -> Result<Option<Block<H256>>, Self::Error>;

    /// Returns the currently configured chain ID, a value used in replay-protected
    /// transaction signing as introduced by EIP-155.
    async fn chain_id(&self) -> Result<U64, Self::Error>;
}
