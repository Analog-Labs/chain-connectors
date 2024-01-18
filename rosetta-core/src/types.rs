use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    default::Default,
    fmt::Debug,
};
use serde::{Deserialize, Serialize};

pub use rosetta_types::{
    AccountIdentifier, Amount, BlockIdentifier, NetworkIdentifier, Operation, OperationIdentifier,
    PartialBlockIdentifier, SignatureType, TransactionIdentifier,
};

#[cfg(feature = "std")]
use std::{string::ToString, vec::Vec};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

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

/// `Currency` is composed of a canonical Symbol and Decimals. This Decimals value is used to
/// convert an Amount.Value from atomic units (Satoshis) to standard units (Bitcoins).
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Currency {
    /// Canonical symbol associated with a currency.
    pub symbol: &'static str,
    /// Number of decimal places in the standard unit representation of the amount.  For example,
    /// BTC has 8 decimals. Note that it is not possible to represent the value of some currency in
    /// atomic units that is not base 10.
    pub decimals: u32,
}

/// `CurveType` is the type of cryptographic curve associated with a `PublicKey`.
/// * [secp256k1: SEC compressed - `33 bytes`](https://secg.org/sec1-v2.pdf#subsubsection.2.3.3)
/// * [secp256r1: SEC compressed - `33 bytes`](https://secg.org/sec1-v2.pdf#subsubsection.2.3.3)
/// * [edwards25519: `y (255-bits) || x-sign-bit (1-bit)` - `32 bytes`](https://ed25519.cr.yp.to/ed25519-20110926.pdf)
/// * [tweedle: 1st pk : Fq.t (32 bytes) || 2nd pk : Fq.t (32 bytes)](https://github.com/CodaProtocol/coda/blob/develop/rfcs/0038-rosetta-construction-api.md#marshal-keys)
/// * [pallas: `x (255 bits) || y-parity-bit (1-bit) - 32 bytes`](https://github.com/zcash/pasta)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum CurveType {
    #[serde(rename = "secp256k1")]
    Secp256k1,
    #[serde(rename = "secp256r1")]
    Secp256r1,
    #[serde(rename = "edwards25519")]
    Edwards25519,
    #[serde(rename = "tweedle")]
    Tweedle,
    #[serde(rename = "pallas")]
    Pallas,
    #[serde(rename = "schnorrkel")]
    Schnorrkel,
}

impl ToString for CurveType {
    fn to_string(&self) -> String {
        match self {
            Self::Secp256k1 => String::from("secp256k1"),
            Self::Secp256r1 => String::from("secp256r1"),
            Self::Edwards25519 => String::from("edwards25519"),
            Self::Tweedle => String::from("tweedle"),
            Self::Pallas => String::from("pallas"),
            Self::Schnorrkel => String::from("schnorrkel"),
        }
    }
}

impl Default for CurveType {
    fn default() -> Self {
        Self::Secp256k1
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
