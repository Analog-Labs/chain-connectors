use crate::bytes::Bytes;
use alloc::vec::Vec;
use ethereum_types::{Address, H256, U256, U64};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageProof {
    pub key: U256,
    pub proof: Vec<Bytes>,
    pub value: U256,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct EIP1186ProofResponse {
    pub address: Address,
    pub balance: U256,
    pub code_hash: H256,
    pub nonce: U64,
    pub storage_hash: H256,
    pub account_proof: Vec<Bytes>,
    pub storage_proof: Vec<StorageProof>,
}
