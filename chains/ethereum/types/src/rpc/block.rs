use crate::{
    block::{Block, BlockBody, SealedBlock},
    bytes::Bytes,
    crypto::Crypto,
    eth_hash::H256,
    eth_uint::U256,
    header::Header,
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
pub struct RpcBlock<TX, OMMERS = H256> {
    /// Hash of the block
    #[cfg_attr(feature = "serde", serde(default))]
    pub hash: Option<H256>,

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
            skip_serializing_if = "Vec::is_empty",
        )
    )]
    pub uncles: Vec<OMMERS>,

    /// Size in bytes
    #[cfg_attr(feature = "serde", serde(default, with = "uint_to_hex"))]
    pub size: Option<u64>,
}

impl<TX, OMMERS> RpcBlock<TX, OMMERS> {
    /// Seal the header with the given hash.
    pub fn seal_slow<C: Crypto>(self) -> SealedBlock<TX, OMMERS> {
        let header = self.header.seal_slow::<C>();
        let body = BlockBody {
            transactions: self.transactions,
            uncles: self.uncles,
            total_difficulty: self.total_difficulty,
            seal_fields: self.seal_fields,
            size: self.size,
        };
        SealedBlock::new(header, body)
    }
}

impl<TX, OMMERS> TryFrom<RpcBlock<TX, OMMERS>> for SealedBlock<TX, OMMERS> {
    type Error = &'static str;

    fn try_from(block: RpcBlock<TX, OMMERS>) -> Result<Self, Self::Error> {
        let Some(hash) = block.hash else {
            return Err("No hash in block");
        };
        let header = block.header.seal(hash);
        let body = BlockBody {
            transactions: block.transactions,
            uncles: block.uncles,
            total_difficulty: block.total_difficulty,
            seal_fields: block.seal_fields,
            size: block.size,
        };
        Ok(Self::new(header, body))
    }
}

impl<TX, OMMERS> From<RpcBlock<TX, OMMERS>> for Block<TX, OMMERS> {
    fn from(block: RpcBlock<TX, OMMERS>) -> Self {
        let header = block.header;
        let body = BlockBody {
            transactions: block.transactions,
            uncles: block.uncles,
            total_difficulty: block.total_difficulty,
            seal_fields: block.seal_fields,
            size: block.size,
        };
        Self { header, body }
    }
}
