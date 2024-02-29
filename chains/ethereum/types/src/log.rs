#[cfg(feature = "serde")]
use crate::serde_utils::uint_to_hex;
use crate::{
    bytes::Bytes,
    eth_hash::{Address, H256},
    eth_uint::U256,
    rstd::{string::String, vec::Vec},
};

/// A log produced by a transaction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "serde",
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
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub block_hash: Option<H256>,

    /// Block Number
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none", with = "uint_to_hex",)
    )]
    pub block_number: Option<u64>,

    /// Transaction Hash
    #[cfg_attr(default, feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub transaction_hash: Option<H256>,

    /// Transaction Index
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none", with = "uint_to_hex",)
    )]
    pub transaction_index: Option<u64>,

    /// Integer of the log index position in the block. None if it's a pending log.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub log_index: Option<U256>,

    /// Integer of the transactions index position log was created from.
    /// None when it's a pending log.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub transaction_log_index: Option<U256>,

    /// Log Type
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub log_type: Option<String>,

    /// True when the log was removed, due to a chain reorganization.
    /// false if it's a valid log.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub removed: Option<bool>,
}
