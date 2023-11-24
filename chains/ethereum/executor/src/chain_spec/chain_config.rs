#[cfg(feature = "with-serde")]
use rosetta_ethereum_primitives::serde_utils::uint_hex_or_decimal;
use rosetta_ethereum_primitives::{H256, U256};

use super::chain_id::ChainId;

/// Represents a node's chain configuration.
///
/// See [geth's `ChainConfig`
/// struct](https://github.com/ethereum/go-ethereum/blob/64dccf7aa411c5c7cd36090c3d9b9892945ae813/params/config.go#L349)
/// for the source of each field.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, rename_all = "camelCase")
)]
pub struct ChainConfig {
    /// The network's chain ID.
    #[cfg_attr(feature = "with-serde", serde(default = "mainnet_id"))]
    pub chain_id: ChainId,

    /// The homestead switch block (None = no fork, 0 = already homestead).
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub homestead_block: Option<u64>,

    /// The DAO fork switch block (None = no fork).
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub dao_fork_block: Option<u64>,

    /// Whether or not the node supports the DAO hard-fork.
    pub dao_fork_support: bool,

    /// The EIP-150 hard fork block (None = no fork).
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub eip150_block: Option<u64>,

    /// The EIP-150 hard fork hash.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub eip150_hash: Option<H256>,

    /// The EIP-155 hard fork block.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub eip155_block: Option<u64>,

    /// The EIP-158 hard fork block.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub eip158_block: Option<u64>,

    /// The Byzantium hard fork block.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub byzantium_block: Option<u64>,

    /// The Constantinople hard fork block.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub constantinople_block: Option<u64>,

    /// The Petersburg hard fork block.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub petersburg_block: Option<u64>,

    /// The Istanbul hard fork block.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub istanbul_block: Option<u64>,

    /// The Muir Glacier hard fork block.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub muir_glacier_block: Option<u64>,

    /// The Berlin hard fork block.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub berlin_block: Option<u64>,

    /// The London hard fork block.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub london_block: Option<u64>,

    /// The Arrow Glacier hard fork block.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub arrow_glacier_block: Option<u64>,

    /// The Gray Glacier hard fork block.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub gray_glacier_block: Option<u64>,

    /// Virtual fork after the merge to use as a network splitter.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub merge_netsplit_block: Option<u64>,

    /// Shanghai switch time.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub shanghai_time: Option<u64>,

    /// Cancun switch time.
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub cancun_time: Option<u64>,

    /// Total difficulty reached that triggers the merge consensus upgrade.
    #[cfg_attr(feature = "with-serde", serde(
        skip_serializing_if = "Option::is_none",
        // deserialize_with = "deserialize_json_ttd_opt"
    ))]
    pub terminal_total_difficulty: Option<U256>,

    /// A flag specifying that the network already passed the terminal total difficulty. Its
    /// purpose is to disable legacy sync without having seen the TTD locally.
    pub terminal_total_difficulty_passed: bool,
    // /// Ethash parameters.
    // #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    // pub ethash: Option<EthashConfig>,

    // /// Clique parameters.
    // #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    // pub clique: Option<CliqueConfig>,
}

// used only for serde
#[cfg(feature = "with-serde")]
#[inline]
const fn mainnet_id() -> ChainId {
    ChainId::MAINNET
}
