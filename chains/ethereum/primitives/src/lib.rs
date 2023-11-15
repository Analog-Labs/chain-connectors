#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod block;
mod bytes;
pub mod constants;
pub mod crypto;
mod eth_hash;
mod eth_uint;
pub mod header;
mod log;
pub mod proof;
#[cfg(feature = "with-rlp")]
pub mod rlp_utils;
pub mod rpc;
#[cfg(feature = "with-serde")]
pub mod serde_utils;
mod storage_proof;
pub mod transactions;
mod tx_receipt;

pub use block::Block;
pub use bytes::Bytes;
pub use eth_hash::{Address, Public, Secret, TxHash, H128, H256, H384, H512, H520};
pub use eth_uint::{U128, U256, U512, U64};
pub use ethbloom::{Bloom, BloomRef, Input as BloomInput};
pub use log::Log;
pub use storage_proof::{EIP1186ProofResponse, StorageProof};
pub use transactions::{
    access_list::{AccessList, AccessListItem, AccessListWithGasUsed},
    signature::Signature,
    signed_transaction::SignedTransaction,
    typed_transaction::TypedTransaction,
};
pub use tx_receipt::TransactionReceipt;

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

impl From<U64> for BlockIdentifier {
    fn from(value: U64) -> Self {
        Self::Number(value.as_u64())
    }
}

impl From<H256> for BlockIdentifier {
    fn from(hash: H256) -> Self {
        Self::Hash(hash)
    }
}

#[cfg(feature = "with-serde")]
impl serde::Serialize for BlockIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::Hash(hash) => <H256 as serde::Serialize>::serialize(hash, serializer),
            Self::Number(number) => {
                <U64 as serde::Serialize>::serialize(&U64::from(*number), serializer)
            },
        }
    }
}
