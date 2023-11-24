#![allow(dead_code)]
use crate::rstd::vec::Vec;
use primitive_types::H256;
use rlp::DecoderError;

pub type DBValue = Vec<u8>;

const LOG_TARGET: &str = "state-db";
const LOG_TARGET_PIN: &str = "state-db::pin";
const PRUNING_MODE: &[u8] = b"mode";
const PRUNING_MODE_ARCHIVE: &[u8] = b"archive";
const PRUNING_MODE_ARCHIVE_CANON: &[u8] = b"archive_canonical";
const PRUNING_MODE_CONSTRAINED: &[u8] = b"constrained";
const DEFAULT_MAX_BLOCK_CONSTRAINT: u32 = 256;

/// Error type.
#[derive(Eq, PartialEq)]
pub enum Error<E> {
    /// Database backend error.
    Db(E),
    StateDb(StateDbError),
}

#[derive(Eq, PartialEq)]
pub enum StateDbError {
    /// `Codec` decoding error.
    Decoding(DecoderError),
    /// Trying to canonicalize invalid block.
    InvalidBlock,
    /// Trying to insert block with invalid number.
    InvalidBlockNumber,
    /// Trying to insert block with unknown parent.
    InvalidParent,
    /// Invalid pruning mode specified. Contains expected mode.
    // IncompatiblePruningModes { stored: PruningMode, requested: PruningMode },
    /// Too many unfinalized sibling blocks inserted.
    TooManySiblingBlocks { number: u64 },
    /// Trying to insert existing block.
    BlockAlreadyExists,
    /// Invalid metadata
    Metadata(&'static str),
    /// Trying to get a block record from db while it is not commit to db yet
    BlockUnavailable,
    /// Block record is missing from the pruning window
    BlockMissing,
}

/// A set of state node changes.
#[derive(Default, Debug, Clone)]
pub struct ChangeSet {
    /// Inserted nodes.
    pub inserted: Vec<(H256, DBValue)>,
    /// Deleted nodes.
    pub deleted: Vec<H256>,
}

/// A set of changes to the backing database.
#[derive(Default, Debug, Clone)]
pub struct CommitSet {
    /// State node changes.
    pub data: ChangeSet,
    /// Metadata changes.
    pub meta: ChangeSet,
}

/// Pruning constraints. If none are specified pruning is
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Constraints {
    /// Maximum blocks. Defaults to 0 when unspecified, effectively keeping only non-canonical
    /// states.
    pub max_blocks: Option<u32>,
}

impl Default for Constraints {
    fn default() -> Self {
        Self { max_blocks: Some(DEFAULT_MAX_BLOCK_CONSTRAINT) }
    }
}

/// Pruning mode.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PruningMode {
    /// Maintain a pruning window.
    Constrained(Constraints),
    /// No pruning. Canonicalization is a no-op.
    ArchiveAll,
    /// Canonicalization discards non-canonical nodes. All the canonical nodes are kept in the DB.
    ArchiveCanonical,
}

impl PruningMode {
    /// Create a mode that keeps given number of blocks.
    #[must_use]
    pub const fn blocks_pruning(n: u32) -> Self {
        Self::Constrained(Constraints { max_blocks: Some(n) })
    }

    /// Is this an archive (either `ArchiveAll` or `ArchiveCanonical`) pruning mode?
    #[must_use]
    pub const fn is_archive(&self) -> bool {
        match *self {
            Self::ArchiveAll | Self::ArchiveCanonical => true,
            Self::Constrained(_) => false,
        }
    }

    /// Returns the pruning mode
    #[must_use]
    pub const fn id(&self) -> &[u8] {
        match self {
            Self::ArchiveAll => PRUNING_MODE_ARCHIVE,
            Self::ArchiveCanonical => PRUNING_MODE_ARCHIVE_CANON,
            Self::Constrained(_) => PRUNING_MODE_CONSTRAINED,
        }
    }

    #[must_use]
    pub fn from_id(id: &[u8]) -> Option<Self> {
        match id {
            PRUNING_MODE_ARCHIVE => Some(Self::ArchiveAll),
            PRUNING_MODE_ARCHIVE_CANON => Some(Self::ArchiveCanonical),
            PRUNING_MODE_CONSTRAINED => Some(Self::Constrained(Constraints::default())),
            _ => None,
        }
    }
}

// pub struct StateDbSync<BlockHash: Hash, Key: Hash, D: MetaDb> {
//     mode: PruningMode,
//     non_canonical: NonCanonicalOverlay<BlockHash, Key>,
//     pruning: Option<RefWindow<BlockHash, Key, D>>,
//     pinned: HashMap<BlockHash, u32>,
//     ref_counting: bool,
// }
