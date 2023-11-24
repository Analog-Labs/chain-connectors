use rosetta_ethereum_primitives::U256;

/// The name of an Ethereum hardfork.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
#[repr(u8)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Hardfork {
    /// Frontier.
    Frontier = 0,
    /// Homestead.
    Homestead = 2,
    /// The DAO fork.
    Dao = 3,
    /// Tangerine.
    Tangerine = 4,
    /// Spurious Dragon.
    SpuriousDragon = 5,
    /// Byzantium.
    Byzantium = 6,
    /// Constantinople.
    Constantinople = 7,
    /// Petersburg.
    Petersburg = 8,
    /// Istanbul.
    Istanbul = 9,
    /// Muir Glacier.
    MuirGlacier = 10,
    /// Berlin.
    Berlin = 11,
    /// London.
    London = 12,
    /// Arrow Glacier.
    ArrowGlacier = 13,
    /// Gray Glacier.
    GrayGlacier = 14,
    /// Paris / Merge.
    Paris = 15,
    /// Shanghai.
    Shanghai = 16,
    /// Cancun.
    Cancun = 17,
    // TODO: Support optimism hardforks
    // Bedrock,
    // Regolith,
}

impl Hardfork {
    #[must_use]
    pub const fn enabled(self, other: Self) -> bool {
        (self as u8) >= (other as u8)
    }
}

/// The condition at which a fork is activated.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ForkCondition {
    /// The fork is activated after a certain block.
    Block(u64),
    /// The fork is activated after a total difficulty has been reached.
    TTD {
        /// The block number at which TTD is reached, if it is known.
        ///
        /// This should **NOT** be set unless you want this block advertised as [EIP-2124][eip2124]
        /// `FORK_NEXT`. This is currently only the case for Sepolia and Holesky.
        ///
        /// [eip2124]: https://eips.ethereum.org/EIPS/eip-2124
        fork_block: Option<u64>,
        /// The total difficulty after which the fork is activated.
        total_difficulty: U256,
    },
    /// The fork is activated after a specific timestamp.
    Timestamp(u64),
    /// The fork is never activated
    #[default]
    Never,
}

impl ForkCondition {
    /// Returns true if the fork condition is timestamp based.
    #[must_use]
    pub const fn is_timestamp(&self) -> bool {
        matches!(self, Self::Timestamp(_))
    }

    /// Checks whether the fork condition is satisfied at the given block.
    ///
    /// For TTD conditions, this will only return true if the activation block is already known.
    ///
    /// For timestamp conditions, this will always return false.
    #[must_use]
    pub const fn active_at_block(&self, current_block: u64) -> bool {
        match self {
            Self::Block(block) | Self::TTD { fork_block: Some(block), .. } => {
                current_block >= *block
            },
            _ => false,
        }
    }

    /// Checks if the given block is the first block that satisfies the fork condition.
    ///
    /// This will return false for any condition that is not block based.
    #[must_use]
    pub const fn transitions_at_block(&self, current_block: u64) -> bool {
        match self {
            Self::Block(block) => current_block == *block,
            _ => false,
        }
    }

    /// Checks whether the fork condition is satisfied at the given total difficulty and difficulty
    /// of a current block.
    ///
    /// The fork is considered active if the _previous_ total difficulty is above the threshold.
    /// To achieve that, we subtract the passed `difficulty` from the current block's total
    /// difficulty, and check if it's above the Fork Condition's total difficulty (here:
    /// `58_750_000_000_000_000_000_000`)
    ///
    /// This will return false for any condition that is not TTD-based.
    #[must_use]
    pub fn active_at_ttd(&self, ttd: U256, difficulty: U256) -> bool {
        if let Self::TTD { total_difficulty, .. } = self {
            ttd.saturating_sub(difficulty) >= *total_difficulty
        } else {
            false
        }
    }

    /// Checks whether the fork condition is satisfied at the given timestamp.
    ///
    /// This will return false for any condition that is not timestamp-based.
    #[must_use]
    pub const fn active_at_timestamp(&self, timestamp: u64) -> bool {
        if let Self::Timestamp(time) = self {
            timestamp >= *time
        } else {
            false
        }
    }

    // /// Checks whether the fork condition is satisfied at the given head block.
    // ///
    // /// This will return true if:
    // ///
    // /// - The condition is satisfied by the block number;
    // /// - The condition is satisfied by the timestamp;
    // /// - or the condition is satisfied by the total difficulty
    // pub fn active_at_head(&self, head: &Head) -> bool {
    //     self.active_at_block(head.number) ||
    //         self.active_at_timestamp(head.timestamp) ||
    //         self.active_at_ttd(head.total_difficulty, head.difficulty)
    // }

    /// Get the total terminal difficulty for this fork condition.
    ///
    /// Returns `None` for fork conditions that are not TTD based.
    #[must_use]
    pub const fn ttd(&self) -> Option<U256> {
        match self {
            Self::TTD { total_difficulty, .. } => Some(*total_difficulty),
            _ => None,
        }
    }

    /// Returns the timestamp of the fork condition, if it is timestamp based.
    #[must_use]
    pub const fn as_timestamp(&self) -> Option<u64> {
        match self {
            Self::Timestamp(timestamp) => Some(*timestamp),
            _ => None,
        }
    }
}
