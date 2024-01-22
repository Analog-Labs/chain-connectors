use crate::{header::Header, rstd::vec::Vec};
use ethereum_types::{H256, U256};

#[cfg(feature = "serde")]
use crate::serde_utils::{bytes_to_hex, uint_to_hex};

/// The block type returned from RPC calls.
///
/// This is generic over a `TX` type which will be either the hash or the full transaction,
/// i.e. `Block<TxHash>` or `Block<Transaction>`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Block<TX, OMMERS = H256> {
    /// Hash of the block
    pub hash: H256,

    /// Block header.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub header: Header,

    /// Total difficulty
    #[cfg_attr(feature = "serde", serde(default))]
    pub total_difficulty: Option<U256>,

    /// Seal fields
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            rename = "sealFields",
            with = "bytes_to_hex",
            skip_serializing_if = "Vec::is_empty",
        )
    )]
    pub seal_fields: Vec<u8>,

    /// Transactions
    #[cfg_attr(
        feature = "serde",
        serde(bound = "TX: serde::Serialize + serde::de::DeserializeOwned")
    )]
    pub transactions: Vec<TX>,

    /// Uncles' hashes
    #[cfg_attr(
        feature = "serde",
        serde(bound = "OMMERS: serde::Serialize + serde::de::DeserializeOwned")
    )]
    pub uncles: Vec<OMMERS>,

    /// Size in bytes
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub size: Option<u64>,
}
