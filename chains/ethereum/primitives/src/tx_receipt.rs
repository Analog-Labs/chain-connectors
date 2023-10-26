use crate::log::Log;
use alloc::vec::Vec;
use core::cmp::Ordering;
use ethereum_types::{Address, Bloom, H256, U256, U64};

/// "Receipt" of an executed transaction: details of its execution.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct TransactionReceipt {
    /// Transaction hash.
    pub transaction_hash: H256,

    /// Index within the block.
    pub transaction_index: U64,

    /// Hash of the block this transaction was included within.
    pub block_hash: Option<H256>,

    /// Number of the block this transaction was included within.
    pub block_number: Option<U64>,

    /// address of the sender.
    pub from: Address,

    // address of the receiver. null when its a contract creation transaction.
    pub to: Option<Address>,

    /// Cumulative gas used within the block after this was executed.
    pub cumulative_gas_used: U256,

    /// Gas used by this transaction alone.
    ///
    /// Gas used is `None` if the the client is running in light client mode.
    pub gas_used: Option<U256>,

    /// Contract address created, or `None` if not a deployment.
    pub contract_address: Option<Address>,

    /// Logs generated within this transaction.
    pub logs: Vec<Log>,

    /// Status: either 1 (success) or 0 (failure). Only present after activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub status: Option<U64>,

    /// State root. Only present before activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    #[cfg_attr(feature = "with-serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub root: Option<H256>,

    /// Logs bloom
    pub logs_bloom: Bloom,

    /// Transaction type, Some(1) for AccessList transaction, None for Legacy
    #[cfg_attr(
        feature = "with-serde",
        serde(rename = "type", default, skip_serializing_if = "Option::is_none")
    )]
    pub transaction_type: Option<U64>,

    /// The price paid post-execution by the transaction (i.e. base fee + priority fee).
    /// Both fields in 1559-style transactions are *maximums* (max fee + max priority fee), the
    /// amount that's actually paid by users can only be determined post-execution
    #[cfg_attr(feature = "with-serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub effective_gas_price: Option<U256>,
}

// Compares the transaction receipt against another receipt by checking the blocks first and then
// the transaction index in the block
impl Ord for TransactionReceipt {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.block_number, other.block_number) {
            (Some(number), Some(other_number)) => match number.cmp(&other_number) {
                Ordering::Equal => self.transaction_index.cmp(&other.transaction_index),
                ord => ord,
            },
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => self.transaction_index.cmp(&other.transaction_index),
        }
    }
}

impl PartialOrd<Self> for TransactionReceipt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
