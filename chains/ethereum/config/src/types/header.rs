use crate::{
    constants::{EMPTY_OMMER_ROOT_HASH, EMPTY_ROOT_HASH},
    rstd::vec::Vec,
};
use ethbloom::Bloom;
use ethereum_types::{H160, H256, U256};

#[cfg(feature = "serde")]
use crate::serde_utils::{bytes_to_hex, uint_to_hex};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Header {
    /// The Keccak 256-bit hash of the parent
    /// block’s header, in its entirety; formally Hp.
    pub parent_hash: H256,

    /// The Keccak 256-bit hash of the ommers list portion of this block; formally Ho.
    #[cfg_attr(feature = "serde", serde(rename = "sha3Uncles"))]
    pub ommers_hash: H256,

    /// The 160-bit address to which all fees collected from the successful mining of this block
    /// be transferred; formally Hc.
    #[cfg_attr(feature = "serde", serde(rename = "miner", alias = "beneficiary"))]
    pub beneficiary: H160,

    /// The Keccak 256-bit hash of the root node of the state trie, after all transactions are
    /// executed and finalisations applied; formally Hr.
    pub state_root: H256,

    /// The Keccak 256-bit hash of the root node of the trie structure populated with each
    /// transaction in the transactions list portion of the block; formally Ht.
    pub transactions_root: H256,

    /// The Keccak 256-bit hash of the root node of the trie structure populated with the receipts
    /// of each transaction in the transactions list portion of the block; formally He.
    pub receipts_root: H256,

    /// The Bloom filter composed from indexable information (logger address and log topics)
    /// contained in each log entry from the receipt of each transaction in the transactions list;
    /// formally Hb.
    pub logs_bloom: Bloom,

    /// A scalar value corresponding to the difficulty level of this block. This can be calculated
    /// from the previous block’s difficulty level and the timestamp; formally Hd.
    pub difficulty: U256,

    /// A scalar value equal to the number of ancestor blocks. The genesis block has a number of
    /// zero; formally Hi.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub number: u64,

    /// A scalar value equal to the current limit of gas expenditure per block; formally Hl.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub gas_limit: u64,

    /// A scalar value equal to the total gas used in transactions in this block; formally Hg.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub gas_used: u64,

    /// A scalar value equal to the reasonable output of Unix’s time() at this block’s inception;
    /// formally Hs.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub timestamp: u64,

    /// An arbitrary byte array containing data relevant to this block. This must be 32 bytes or
    /// fewer; formally Hx.
    #[cfg_attr(feature = "serde", serde(default, with = "bytes_to_hex"))]
    pub extra_data: Vec<u8>,

    /// A 256-bit hash which, combined with the
    /// nonce, proves that a sufficient amount of computation has been carried out on this block;
    /// formally Hm.
    #[cfg_attr(feature = "serde", serde(default))]
    pub mix_hash: H256,

    /// A 64-bit value which, combined with the mixhash, proves that a sufficient amount of
    /// computation has been carried out on this block; formally Hn.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub nonce: u64,

    /// A scalar representing EIP1559 base fee which can move up or down each block according
    /// to a formula which is a function of gas used in parent block and gas target
    /// (block gas limit divided by elasticity multiplier) of parent block.
    /// The algorithm results in the base fee per gas increasing when blocks are
    /// above the gas target, and decreasing when blocks are below the gas target. The base fee per
    /// gas is burned.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none", with = "uint_to_hex")
    )]
    pub base_fee_per_gas: Option<u64>,

    /// The Keccak 256-bit hash of the withdrawals list portion of this block.
    /// <https://eips.ethereum.org/EIPS/eip-4895>
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub withdrawals_root: Option<H256>,

    /// The total amount of blob gas consumed by the transactions within the block, added in
    /// EIP-4844.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none", with = "uint_to_hex")
    )]
    pub blob_gas_used: Option<u64>,

    /// A running total of blob gas consumed in excess of the target, prior to the block. Blocks
    /// with above-target blob gas consumption increase this value, blocks with below-target blob
    /// gas consumption decrease it (bounded at 0). This was added in EIP-4844.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none", with = "uint_to_hex")
    )]
    pub excess_blob_gas: Option<u64>,

    /// The hash of the parent beacon block's root is included in execution blocks, as proposed by
    /// EIP-4788.
    ///
    /// This enables trust-minimized access to consensus state, supporting staking pools, bridges,
    /// and more.
    ///
    /// The beacon roots contract handles root storage, enhancing Ethereum's functionalities.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub parent_beacon_block_root: Option<H256>,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            parent_hash: H256::zero(),
            ommers_hash: EMPTY_OMMER_ROOT_HASH,
            beneficiary: H160::zero(),
            state_root: EMPTY_ROOT_HASH,
            transactions_root: EMPTY_ROOT_HASH,
            receipts_root: EMPTY_ROOT_HASH,
            logs_bloom: Bloom::zero(),
            difficulty: U256::zero(),
            number: 0,
            gas_limit: 0,
            gas_used: 0,
            timestamp: 0,
            extra_data: Vec::new(),
            mix_hash: H256::zero(),
            nonce: 0,
            base_fee_per_gas: None,
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
        }
    }
}
