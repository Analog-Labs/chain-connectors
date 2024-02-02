use crate::{
    bytes::Bytes,
    eth_hash::H256,
    eth_uint::U256,
    header::{Header, SealedHeader},
    rstd::vec::Vec,
};

#[cfg(feature = "serde")]
use crate::serde_utils::{default_empty_vec, deserialize_null_default, uint_to_hex};

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
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct BlockBody<TX, OMMERS = H256> {
    /// Total difficulty
    #[cfg_attr(feature = "serde", serde(default))]
    pub total_difficulty: Option<U256>,

    /// Seal fields
    #[cfg_attr(
        feature = "serde",
        serde(
            default = "default_empty_vec",
            rename = "sealFields",
            deserialize_with = "deserialize_null_default",
            skip_serializing_if = "Vec::is_empty",
        )
    )]
    pub seal_fields: Vec<Bytes>,

    /// Transactions
    #[cfg_attr(
        feature = "serde",
        serde(
            default = "default_empty_vec",
            bound(
                serialize = "TX: serde::Serialize",
                deserialize = "TX: serde::de::DeserializeOwned"
            ),
            deserialize_with = "deserialize_null_default",
            skip_serializing_if = "Vec::is_empty",
        )
    )]
    pub transactions: Vec<TX>,

    /// Uncles' hashes
    #[cfg_attr(
        feature = "serde",
        serde(
            default = "default_empty_vec",
            bound(
                serialize = "OMMERS: serde::Serialize",
                deserialize = "OMMERS: serde::de::DeserializeOwned"
            ),
            deserialize_with = "deserialize_null_default",
            skip_serializing_if = "Vec::is_empty",
        )
    )]
    pub uncles: Vec<OMMERS>,

    /// Size in bytes
    #[cfg_attr(feature = "serde", serde(default, with = "uint_to_hex"))]
    pub size: Option<u64>,
}

impl<TX, OMMERS> BlockBody<TX, OMMERS> {
    #[must_use]
    pub fn with_transactions<T>(self, transactions: Vec<T>) -> BlockBody<T, OMMERS> {
        BlockBody {
            total_difficulty: self.total_difficulty,
            seal_fields: self.seal_fields,
            transactions,
            uncles: self.uncles,
            size: self.size,
        }
    }

    pub fn map_transactions<T>(self, cb: impl FnMut(TX) -> T) -> BlockBody<T, OMMERS> {
        BlockBody {
            total_difficulty: self.total_difficulty,
            seal_fields: self.seal_fields,
            transactions: self.transactions.into_iter().map(cb).collect(),
            uncles: self.uncles,
            size: self.size,
        }
    }

    #[must_use]
    pub fn with_ommers<T>(self, uncles: Vec<T>) -> BlockBody<TX, T> {
        BlockBody {
            total_difficulty: self.total_difficulty,
            seal_fields: self.seal_fields,
            transactions: self.transactions,
            uncles,
            size: self.size,
        }
    }

    pub fn map_ommers<T>(self, cb: impl FnMut(OMMERS) -> T) -> BlockBody<TX, T> {
        BlockBody {
            total_difficulty: self.total_difficulty,
            seal_fields: self.seal_fields,
            transactions: self.transactions,
            uncles: self.uncles.into_iter().map(cb).collect(),
            size: self.size,
        }
    }
}

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
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Block<TX, OMMERS = H256> {
    /// Block header.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub header: Header,

    /// Block body.
    #[cfg_attr(
        feature = "serde",
        serde(
            flatten,
            bound(
                serialize = "TX: serde::Serialize, OMMERS: serde::Serialize",
                deserialize = "TX: serde::de::DeserializeOwned, OMMERS: serde::de::DeserializeOwned"
            )
        )
    )]
    pub body: BlockBody<TX, OMMERS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct SealedBlock<TX, OMMERS = H256> {
    /// Locked block header.
    #[cfg_attr(feature = "serde", serde(flatten))]
    header: SealedHeader,

    /// Locked block
    #[cfg_attr(
        feature = "serde",
        serde(
            flatten,
            bound(
                serialize = "TX: serde::Serialize, OMMERS: serde::Serialize",
                deserialize = "TX: serde::de::DeserializeOwned, OMMERS: serde::de::DeserializeOwned"
            )
        )
    )]
    body: BlockBody<TX, OMMERS>,
}

impl<TX, OMMERS> SealedBlock<TX, OMMERS> {
    pub const fn new(header: SealedHeader, body: BlockBody<TX, OMMERS>) -> Self {
        Self { header, body }
    }

    pub fn unseal(self) -> (SealedHeader, BlockBody<TX, OMMERS>) {
        (self.header, self.body)
    }

    pub const fn header(&self) -> &SealedHeader {
        &self.header
    }

    pub const fn body(&self) -> &BlockBody<TX, OMMERS> {
        &self.body
    }
}
