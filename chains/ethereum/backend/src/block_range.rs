use rosetta_ethereum_types::{Address, AtBlock, H256};

#[cfg(feature = "serde")]
use crate::serde_util::opt_value_or_array;

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct BlockRange {
    /// A list of addresses from which logs should originate.
    #[cfg_attr(
        feature = "serde",
        serde(with = "opt_value_or_array", skip_serializing_if = "Vec::is_empty")
    )]
    pub address: Vec<Address>,
    /// Array of topics. topics are order-dependent.
    pub topics: Vec<H256>,
    /// Array of topics. topics are order-dependent.
    #[cfg_attr(feature = "serde", serde(rename = "fromBlock"))]
    pub from: AtBlock,
    /// A hexadecimal block number, or the string latest, earliest or pending
    #[cfg_attr(feature = "serde", serde(rename = "toBlock"))]
    pub to: AtBlock,
    #[cfg_attr(feature = "serde", serde(rename = "blockHash"))]
    blockhash: Option<AtBlock>,
}

impl Default for BlockRange {
    fn default() -> Self {
        Self {
            address: Vec::new(),
            from: AtBlock::Latest,
            to: AtBlock::Latest,
            topics: Vec::new(),
            blockhash: None,
        }
    }
}
