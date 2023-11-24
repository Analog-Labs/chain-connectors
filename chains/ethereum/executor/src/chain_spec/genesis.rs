use super::{chain_config::ChainConfig, chain_id::ChainId};
use hex_literal::hex;
use rosetta_ethereum_primitives::{Address, Bytes, H256, U256};

#[cfg(feature = "with-serde")]
use rosetta_ethereum_primitives::serde_utils::uint_hex_or_decimal;

/// The genesis block specification.
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
pub struct Genesis {
    /// The fork configuration for this network.
    #[cfg_attr(feature = "with-serde", serde(default))]
    pub config: ChainConfig,
    /// The genesis header nonce.
    #[cfg_attr(feature = "with-serde", serde(with = "uint_hex_or_decimal"))]
    pub nonce: u64,
    /// The genesis header timestamp.
    #[cfg_attr(feature = "with-serde", serde(with = "uint_hex_or_decimal"))]
    pub timestamp: u64,
    /// The genesis header extra data.
    pub extra_data: Bytes,
    /// The genesis header gas limit.
    #[cfg_attr(feature = "with-serde", serde(with = "uint_hex_or_decimal"))]
    pub gas_limit: u64,
    /// The genesis header difficulty.
    pub difficulty: U256,
    /// The genesis header mix hash.
    pub mix_hash: H256,
    /// The genesis header coinbase address.
    pub coinbase: Address,
    // /// The initial state of accounts in the genesis block.
    // pub alloc: BTreeMap<Address, GenesisAccount>,

    // NOTE: the following fields:
    // * base_fee_per_gas
    // * excess_blob_gas
    // * blob_gas_used
    // should NOT be set in a real genesis file, but are included here for compatibility with
    // consensus tests, which have genesis files with these fields populated.
    /// The genesis header base fee
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub base_fee_per_gas: Option<u64>,
    /// The genesis header excess blob gas
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub excess_blob_gas: Option<u64>,
    /// The genesis header blob gas used
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_hex_or_decimal")
    )]
    pub blob_gas_used: Option<u64>,
}

pub const MAINNET_CONFIG: ChainConfig = ChainConfig {
    chain_id: ChainId::MAINNET,
    homestead_block: None,
    dao_fork_block: None,
    dao_fork_support: false,
    eip150_block: None,
    eip150_hash: None,
    eip155_block: None,
    eip158_block: None,
    byzantium_block: None,
    constantinople_block: None,
    petersburg_block: None,
    istanbul_block: None,
    muir_glacier_block: None,
    berlin_block: None,
    london_block: None,
    arrow_glacier_block: None,
    gray_glacier_block: None,
    merge_netsplit_block: None,
    shanghai_time: None,
    cancun_time: None,
    terminal_total_difficulty: None,
    terminal_total_difficulty_passed: false,
};

#[allow(clippy::declare_interior_mutable_const)]
pub const MAINNET_GENESIS: Genesis = Genesis {
    config: MAINNET_CONFIG,
    nonce: 0x42,
    timestamp: 0x0,
    extra_data: Bytes::from_static(&hex!(
        "11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa"
    )),
    gas_limit: 0x1388,
    difficulty: U256([0, 0, 0, 0x04_00_00_00_00]),
    mix_hash: H256::zero(),
    coinbase: Address::zero(),
    base_fee_per_gas: None,
    excess_blob_gas: None,
    blob_gas_used: None,
};
