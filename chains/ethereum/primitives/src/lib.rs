#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod block;
mod bytes;
mod log;
mod storage_proof;
mod tx_receipt;

pub use block::Block;
pub use bytes::Bytes;
pub use ethereum_types::{Address, Bloom, H256, H64, U128, U256, U64};
pub use log::Log;
pub use storage_proof::{EIP1186ProofResponse, StorageProof};
pub use tx_receipt::TransactionReceipt;

pub type TxHash = H256;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
pub enum BlockIdentifier {
    Hash(H256),
    Number(u64),
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
