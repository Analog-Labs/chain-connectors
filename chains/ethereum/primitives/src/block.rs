use crate::{bytes::Bytes, eth_hash::H256, eth_uint::U256, header::Header};
use alloc::vec::Vec;

/// The block type returned from RPC calls.
///
/// This is generic over a `TX` type which will be either the hash or the full transaction,
/// i.e. `Block<TxHash>` or `Block<Transaction>`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Block<TX> {
    /// Hash of the block
    pub hash: H256,

    /// Block header.
    #[cfg_attr(feature = "with-serde", serde(flatten))]
    pub header: Header,

    /// Total difficulty
    #[cfg_attr(feature = "with-serde", serde(default))]
    pub total_difficulty: Option<U256>,

    /// Seal fields
    #[cfg_attr(
        feature = "with-serde",
        serde(default, rename = "sealFields", deserialize_with = "deserialize_null_default")
    )]
    pub seal_fields: Vec<Bytes>,

    /// Transactions
    #[cfg_attr(
        feature = "with-serde",
        serde(bound = "TX: serde::Serialize + serde::de::DeserializeOwned", default)
    )]
    pub transactions: Vec<TX>,

    /// Uncles' hashes
    #[cfg_attr(feature = "with-serde", serde(default))]
    pub uncles: Vec<H256>,

    /// Size in bytes
    pub size: Option<U256>,
}

#[cfg(feature = "with-serde")]
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    let opt = <Option<T> as serde::Deserialize<'de>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
