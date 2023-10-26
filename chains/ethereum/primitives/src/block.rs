use crate::bytes::Bytes;
use alloc::vec::Vec;
use ethereum_types::{Address, Bloom, H256, H64, U256, U64};

/// The block type returned from RPC calls.
///
/// This is generic over a `TX` type which will be either the hash or the full transaction,
/// i.e. `Block<TxHash>` or `Block<Transaction>`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Block<TX> {
    /// Hash of the block
    pub hash: H256,
    /// Hash of the parent
    pub parent_hash: H256,
    /// Hash of the uncles
    pub sha3_uncles: H256,
    /// Miner/author's address.
    pub miner: Option<Address>,
    /// State root hash
    pub state_root: H256,
    /// Transactions root hash
    pub transactions_root: H256,
    /// Transactions receipts root hash
    pub receipts_root: H256,
    /// Block number.
    pub number: U64,
    /// Gas Used
    pub gas_used: U256,
    /// Gas Limit
    pub gas_limit: U256,
    /// Extra data
    pub extra_data: Bytes,
    /// Logs bloom
    pub logs_bloom: Option<Bloom>,
    /// Timestamp
    #[cfg_attr(feature = "with-serde", serde(default))]
    pub timestamp: U256,
    /// Difficulty
    #[cfg_attr(feature = "with-serde", serde(default))]
    pub difficulty: U256,
    /// Total difficulty
    pub total_difficulty: Option<U256>,
    /// Seal fields
    #[cfg_attr(
        feature = "with-serde",
        serde(default, rename = "sealFields", deserialize_with = "deserialize_null_default")
    )]
    pub seal_fields: Vec<Bytes>,
    /// Uncles' hashes
    #[cfg_attr(feature = "with-serde", serde(default))]
    pub uncles: Vec<H256>,
    /// Transactions
    #[cfg_attr(
        feature = "with-serde",
        serde(bound = "TX: serde::Serialize + serde::de::DeserializeOwned", default)
    )]
    pub transactions: Vec<TX>,
    /// Size in bytes
    pub size: Option<U256>,
    /// Mix Hash
    pub mix_hash: Option<H256>,
    /// Nonce
    pub nonce: Option<H64>,
    /// Base fee per unit of gas (if past London)
    pub base_fee_per_gas: Option<U256>,
    /// Withdrawals root hash (if past Shanghai)
    #[cfg_attr(feature = "with-serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub withdrawals_root: Option<H256>,
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
