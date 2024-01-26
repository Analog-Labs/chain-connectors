use crate::{
    bytes::Bytes,
    eth_hash::{Address, H256},
    eth_uint::U256,
    rstd::vec::Vec,
};

#[cfg(feature = "serde")]
use crate::serde_utils::uint_to_hex;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageProof {
    pub key: U256,
    pub proof: Vec<Bytes>,
    pub value: U256,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct EIP1186ProofResponse {
    pub address: Address,
    pub balance: U256,
    pub code_hash: H256,
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub nonce: u64,
    pub storage_hash: H256,
    pub account_proof: Vec<Bytes>,
    pub storage_proof: Vec<StorageProof>,
}
