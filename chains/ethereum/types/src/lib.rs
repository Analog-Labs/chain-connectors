#![cfg_attr(not(feature = "std"), no_std)]

mod block;
mod bytes;
pub mod constants;
pub mod crypto;
mod eth_hash;
mod eth_uint;
mod fee_history;
pub mod header;
mod i256;
mod log;
#[cfg(feature = "with-rlp")]
pub mod rlp_utils;
pub mod rpc;
#[cfg(feature = "serde")]
pub mod serde_utils;
mod storage_proof;
pub mod transactions;
mod tx_receipt;

pub use block::{Block, BlockBody, SealedBlock};
pub use bytes::Bytes;
pub use eth_hash::{Address, Public, Secret, TxHash, H128, H160, H256, H384, H512, H520};
pub use eth_uint::{U128, U256, U512};
pub use ethbloom::{Bloom, BloomRef, Input as BloomInput};
pub use fee_history::FeeHistory;
pub use header::{Header, SealedHeader};
pub use i256::I256;
pub use log::Log;
pub use num_rational::Rational64;
use rstd::{
    cmp::{Ordering, PartialOrd},
    fmt::{Display, Formatter, Result as FmtResult},
};
pub use storage_proof::{EIP1186ProofResponse, StorageProof};
pub use transactions::{
    access_list::{AccessList, AccessListItem, AccessListWithGasUsed},
    signature::Signature,
    signed_transaction::SignedTransaction,
    typed_transaction::TypedTransaction,
    Transaction, TransactionT,
};
pub use tx_receipt::TransactionReceipt;

#[cfg(not(feature = "std"))]
#[cfg_attr(all(test, any(feature = "serde", feature = "with-rlp")), macro_use)]
extern crate alloc;

#[cfg(feature = "std")]
pub(crate) mod rstd {
    #[cfg(feature = "serde")]
    pub use std::{default, format, mem, option, result};

    pub use std::{borrow, cmp, fmt, ops, str, string, vec};
}

#[cfg(not(feature = "std"))]
pub(crate) mod rstd {
    #[cfg(feature = "serde")]
    pub use core::{default, mem, option, result};

    #[cfg(feature = "serde")]
    pub use alloc::format;

    pub use alloc::{borrow, fmt, string, vec};
    pub use core::{cmp, ops, str};
}

/// Re-exports for proc-macro library to not require any additional
/// dependencies to be explicitly added on the client side.
pub mod ext {
    pub use bytes;
    pub use ethbloom;
    #[cfg(feature = "with-crypto")]
    pub use libsecp256k1;
    #[cfg(feature = "with-codec")]
    pub use parity_scale_codec;
    pub use primitive_types;
    #[cfg(feature = "with-rlp")]
    pub use rlp;
    #[cfg(feature = "with-rlp")]
    pub use rlp_derive;
    #[cfg(feature = "with-codec")]
    pub use scale_info;
    #[cfg(feature = "serde")]
    pub use serde;
    #[cfg(feature = "with-crypto")]
    pub use sha3;
    #[cfg(feature = "with-crypto")]
    pub use trie_root;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
pub enum BlockIdentifier {
    Hash(H256),
    Number(u64),
}

impl From<u64> for BlockIdentifier {
    fn from(value: u64) -> Self {
        Self::Number(value)
    }
}

impl From<H256> for BlockIdentifier {
    fn from(hash: H256) -> Self {
        Self::Hash(hash)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for BlockIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use crate::serde_utils::uint_to_hex;
        match self {
            Self::Hash(hash) => <H256 as serde::Serialize>::serialize(hash, serializer),
            Self::Number(number) => uint_to_hex::serialize(number, serializer),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash)]
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

impl From<H256> for AtBlock {
    fn from(hash: H256) -> Self {
        Self::At(BlockIdentifier::Hash(hash))
    }
}

impl From<[u8; 32]> for AtBlock {
    fn from(hash: [u8; 32]) -> Self {
        Self::At(BlockIdentifier::Hash(H256(hash)))
    }
}

impl From<u64> for AtBlock {
    fn from(block_number: u64) -> Self {
        Self::At(BlockIdentifier::Number(block_number))
    }
}

impl From<u32> for AtBlock {
    fn from(block_number: u32) -> Self {
        Self::At(BlockIdentifier::Number(u64::from(block_number)))
    }
}

impl From<BlockIdentifier> for AtBlock {
    fn from(block: BlockIdentifier) -> Self {
        Self::At(block)
    }
}

impl PartialOrd for AtBlock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Convert AtBlock to a number
        const fn as_number(at: &AtBlock) -> Option<u8> {
            let n = match at {
                AtBlock::Pending => 50,
                AtBlock::Latest => 40,
                AtBlock::Safe => 30,
                AtBlock::Finalized => 20,
                AtBlock::Earliest => 10,
                AtBlock::At(_) => return None,
            };
            Some(n)
        }
        let this = as_number(self)?;
        let other = as_number(other)?;
        Some(<u8 as rstd::cmp::Ord>::cmp(&this, &other))
    }
}

impl Display for AtBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Latest => f.write_str("latest"),
            Self::Finalized => f.write_str("finalized"),
            Self::Safe => f.write_str("safe"),
            Self::Earliest => f.write_str("earliest"),
            Self::Pending => f.write_str("ending"),
            Self::At(BlockIdentifier::Hash(hash)) => Display::fmt(&hash, f),
            Self::At(BlockIdentifier::Number(number)) => Display::fmt(&number, f),
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

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AtBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::str::FromStr;
        let s = <rstd::string::String as serde::Deserialize>::deserialize(deserializer)?;
        match s.as_str() {
            "latest" => return Ok(Self::Latest),
            "finalized" => return Ok(Self::Finalized),
            "safe" => return Ok(Self::Safe),
            "earliest" => return Ok(Self::Earliest),
            "pending" => return Ok(Self::Pending),
            _ => {},
        }

        if let Some(hexdecimal) = s.strip_prefix("0x") {
            if s.len() == 66 {
                let hash = H256::from_str(hexdecimal).map_err(serde::de::Error::custom)?;
                Ok(Self::At(BlockIdentifier::Hash(hash)))
            } else if hexdecimal.is_empty() {
                Ok(Self::At(BlockIdentifier::Number(0)))
            } else {
                let number =
                    u64::from_str_radix(hexdecimal, 16).map_err(serde::de::Error::custom)?;
                Ok(Self::At(BlockIdentifier::Number(number)))
            }
        } else {
            let number = s.parse::<u64>().map_err(serde::de::Error::custom)?;
            Ok(Self::At(BlockIdentifier::Number(number)))
        }
    }
}
