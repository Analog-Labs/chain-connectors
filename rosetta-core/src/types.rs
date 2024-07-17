use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    default::Default,
    fmt::Debug,
};
use serde::{Deserialize, Serialize};

pub use rosetta_types::{
    AccountIdentifier, CallRequest, CurveType, Operation, OperationIdentifier, PublicKey,
    SignatureType, TransactionIdentifier,
};

use std::{fmt::Display, vec::Vec};

/// Block : Blocks contain an array of Transactions that occurred at a particular `BlockIdentifier`.
/// A hard requirement for blocks returned by Rosetta implementations is that they MUST be
/// _inalterable_: once a client has requested and received a block identified by a specific
/// `BlockIndentifier`, all future calls for that same `BlockIdentifier` must return the same block
/// contents.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Block {
    #[serde(rename = "block_identifier")]
    pub block_identifier: BlockIdentifier,
    #[serde(rename = "parent_block_identifier")]
    pub parent_block_identifier: BlockIdentifier,
    /// The timestamp of the block in milliseconds since the Unix Epoch. The timestamp is stored in
    /// milliseconds because some blockchains produce blocks more often than once a second.
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "transactions")]
    pub transactions: Vec<Transaction>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// `BlockIdentifier` : The `block_identifier` uniquely identifies a block in a particular network.
#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BlockIdentifier {
    /// This is also known as the block height.
    #[serde(rename = "index")]
    pub index: u64,
    /// This should be normalized according to the case specified in the `block_hash` case network
    /// options.
    #[serde(skip_serializing)]
    pub hash: [u8; 32],
}

impl BlockIdentifier {
    /// The `block_identifier` uniquely identifies a block in a particular network.
    #[must_use]
    pub const fn new(index: u64, hash: [u8; 32]) -> Self {
        Self { index, hash }
    }
}

impl Display for BlockIdentifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let hash_hex = const_hex::encode_prefixed(self.hash);
        write!(f, "{}: {}", self.index, hash_hex)
    }
}

impl Debug for BlockIdentifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let hash_hex = const_hex::encode_prefixed(self.hash);
        f.debug_struct("BlockIdentifier")
            .field("index", &self.index)
            .field("hash", &hash_hex)
            .finish()
    }
}

/// `PartialBlockIdentifier` : When fetching data by `BlockIdentifier`, it may be possible to only
/// specify the index or hash. If neither property is specified, it is assumed that the client is
/// making a request at the current block.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PartialBlockIdentifier {
    #[serde(rename = "index", skip_serializing_if = "Option::is_none")]
    pub index: Option<u64>,
    #[serde(skip_serializing)]
    pub hash: Option<[u8; 32]>,
}

impl From<u64> for PartialBlockIdentifier {
    fn from(block_number: u64) -> Self {
        Self { index: Some(block_number), hash: None }
    }
}

impl From<[u8; 32]> for PartialBlockIdentifier {
    fn from(block_hash: [u8; 32]) -> Self {
        Self { index: None, hash: Some(block_hash) }
    }
}

impl From<&[u8; 32]> for PartialBlockIdentifier {
    fn from(block_hash: &[u8; 32]) -> Self {
        Self { index: None, hash: Some(*block_hash) }
    }
}

impl From<BlockIdentifier> for PartialBlockIdentifier {
    fn from(block_identifier: BlockIdentifier) -> Self {
        Self { index: Some(block_identifier.index), hash: Some(block_identifier.hash) }
    }
}

impl PartialBlockIdentifier {
    /// When fetching data by `BlockIdentifier`, it may be possible to only specify the index or
    /// hash. If neither property is specified, it is assumed that the client is making a request at
    /// the current block.
    #[must_use]
    pub const fn new() -> Self {
        Self { index: None, hash: None }
    }
}

/// `Transaction` contain an array of Operations that are attributable to the same
/// `TransactionIdentifier`.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_identifier: TransactionIdentifier,

    /// Raw transaction bytes
    pub raw_tx: Vec<u8>,

    /// Raw transaction bytes
    pub raw_tx_receipt: Option<Vec<u8>>,
}
