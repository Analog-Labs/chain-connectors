use crate::bytes::Bytes;
use alloc::{string::String, vec::Vec};
use ethereum_types::{Address, H256, U256, U64};

/// A log produced by a transaction.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Log {
    /// H160. the contract that emitted the log
    pub address: Address,

    /// topics: Array of 0 to 4 32 Bytes of indexed log arguments.
    /// (In solidity: The first topic is the hash of the signature of the event
    /// (e.g. `Deposit(address,bytes32,uint256)`), except you declared the event
    /// with the anonymous specifier.)
    pub topics: Vec<H256>,

    /// Data
    pub data: Bytes,

    /// Block Hash
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub block_hash: Option<H256>,

    /// Block Number
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub block_number: Option<U64>,

    /// Transaction Hash
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub transaction_hash: Option<H256>,

    /// Transaction Index
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub transaction_index: Option<U64>,

    /// Integer of the log index position in the block. None if it's a pending log.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub log_index: Option<U256>,

    /// Integer of the transactions index position log was created from.
    /// None when it's a pending log.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub transaction_log_index: Option<U256>,

    /// Log Type
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub log_type: Option<String>,

    /// True when the log was removed, due to a chain reorganization.
    /// false if it's a valid log.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub removed: Option<bool>,
}
