#![cfg_attr(not(feature = "std"), no_std)]

mod block;
mod bytes;
pub mod constants;
pub mod crypto;
mod eth_hash;
mod eth_uint;
pub mod header;
mod log;
#[cfg(feature = "with-rlp")]
pub mod rlp_utils;
pub mod rpc;
#[cfg(feature = "serde")]
pub mod serde_utils;
mod storage_proof;
pub mod transactions;
mod tx_receipt;

pub use block::Block;
pub use bytes::Bytes;
pub use eth_hash::{Address, Public, Secret, TxHash, H128, H256, H384, H512, H520};
pub use eth_uint::{U128, U256, U512};
pub use ethbloom::{Bloom, BloomRef, Input as BloomInput};
pub use header::Header;
pub use log::Log;
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
    pub use std::{format, option, result};

    pub use std::{borrow, cmp, fmt, ops, str, string, vec};
}

#[cfg(not(feature = "std"))]
pub(crate) mod rstd {
    #[cfg(feature = "serde")]
    pub use core::{option, result};

    #[cfg(feature = "serde")]
    pub use alloc::format;

    pub use alloc::{borrow, fmt, string, vec};
    pub use core::{cmp, ops, str};
}

use rstd::fmt::{Display, Formatter, Result as FmtResult};

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

impl From<BlockIdentifier> for AtBlock {
    fn from(block: BlockIdentifier) -> Self {
        Self::At(block)
    }
}
