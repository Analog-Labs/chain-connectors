mod chain_config;
mod chain_id;
mod genesis;
mod hard_fork;

use alloc::collections::BTreeMap;
pub use chain_config::ChainConfig;
pub use chain_id::ChainId;
pub use genesis::{Genesis, MAINNET_GENESIS};
pub use hard_fork::{ForkCondition, Hardfork};
use rosetta_ethereum_primitives::H256;

/// An Ethereum chain specification.
///
/// A chain specification describes:
///
/// - Meta-information about the chain (the chain ID)
/// - The genesis block of the chain ([`Genesis`])
/// - What hardforks are activated, and under which conditions
#[derive(Debug, Clone)]
pub struct ChainSpec {
    /// The chain ID
    pub chain_id: ChainId,

    /// The hash of the genesis block.
    ///
    /// This acts as a small cache for known chains. If the chain is known, then the genesis hash
    /// is also known ahead of time, and this will be `Some`.
    // #[serde(skip, default)]
    pub genesis_hash: Option<H256>,

    /// The genesis block
    pub genesis: Genesis,

    // /// The block at which [Hardfork::Paris] was activated and the final difficulty at this
    // block. // #[serde(skip, default)]
    // pub paris_block_and_final_difficulty: Option<(u64, U256)>,

    // // #[serde(skip, default)]
    // /// Timestamps of various hardforks
    // ///
    // /// This caches entries in `hardforks` map
    // pub fork_timestamps: ForkTimestamps,
    /// The active hard forks and their activation conditions
    pub hardforks: BTreeMap<Hardfork, ForkCondition>,

    // #[serde(skip, default)]
    /// The deposit contract deployed for PoS
    // pub deposit_contract: Option<DepositContract>,

    // /// The parameters that configure how a block's base fee is computed
    // pub base_fee_params: BaseFeeParams,

    // #[serde(default)]
    /// The delete limit for pruner, per block. In the actual pruner run it will be multiplied by
    /// the amount of blocks between pruner runs to account for the difference in amount of new
    /// data coming in.
    pub prune_delete_limit: usize,

    /// The block interval for creating snapshots. Each snapshot will have that much blocks in it.
    pub snapshot_block_interval: u64,
}

impl ChainSpec {
    /// Get the fork condition for the given fork.
    pub fn fork(&self, fork: Hardfork) -> ForkCondition {
        self.hardforks.get(&fork).copied().unwrap_or(ForkCondition::Never)
    }
}
