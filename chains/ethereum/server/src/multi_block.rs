use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use crate::utils::{FullBlock, PartialBlock};
use rosetta_core::{types::BlockIdentifier, BlockOrIdentifier};
use rosetta_ethereum_backend::ext::types::{
    crypto::DefaultCrypto, Header, SealedHeader, H256, U256,
};

/// A `MultiBlock` can be either a `FullBlock`, a `PartialBlock` or a Header
/// The ethereum RPC API returns blocks in different formats, this enum is used to store them
#[derive(Debug, Clone)]
pub enum MultiBlock {
    // Full block data, including transactions and ommers
    Full(FullBlock),
    // Partial block data, including the header and the transactions hashes
    Partial(PartialBlock),
    // Only the block header
    Header(SealedHeader),
}

impl MultiBlock {
    #[must_use]
    pub const fn header(&self) -> &SealedHeader {
        match self {
            Self::Full(block) => block.header(),
            Self::Partial(block) => block.header(),
            Self::Header(header) => header,
        }
    }

    #[must_use]
    pub const fn hash(&self) -> H256 {
        match self {
            Self::Full(block) => block.header().hash(),
            Self::Partial(block) => block.header().hash(),
            Self::Header(header) => header.hash(),
        }
    }

    #[must_use]
    pub const fn number(&self) -> u64 {
        match self {
            Self::Full(block) => block.header().header().number,
            Self::Partial(block) => block.header().header().number,
            Self::Header(header) => header.header().number,
        }
    }

    #[must_use]
    pub const fn parent_hash(&self) -> H256 {
        match self {
            Self::Full(block) => block.header().header().parent_hash,
            Self::Partial(block) => block.header().header().parent_hash,
            Self::Header(header) => header.header().parent_hash,
        }
    }

    #[must_use]
    pub fn parent_ref(&self) -> BlockRef {
        let header = self.header().header();
        if header.number == 0 || header.parent_hash.is_zero() {
            // This is the genesis block
            BlockRef { number: 0, hash: H256::zero() }
        } else {
            BlockRef { number: header.number - 1, hash: header.parent_hash }
        }
    }

    pub fn eq_headers(&self, other: &Self) -> bool {
        let this_header = self.header();
        let other_header = other.header();
        this_header == other_header
    }

    pub fn upgrade(&mut self, other: Self) -> Self {
        let should_upgrade = matches!(
            (&self, &other),
            (Self::Partial(_), Self::Full(_)) | (Self::Header(_), Self::Full(_) | Self::Partial(_))
        );
        if should_upgrade {
            std::mem::replace(self, other)
        } else {
            other
        }
    }

    #[must_use]
    pub const fn as_block_ref(&self) -> BlockRef {
        BlockRef { number: self.number(), hash: self.hash() }
    }
}

impl PartialEq for MultiBlock {
    fn eq(&self, other: &Self) -> bool {
        self.hash() == other.hash()
    }
}

impl Eq for MultiBlock {}

impl PartialOrd for MultiBlock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for MultiBlock {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_block_ref().cmp(&other.as_block_ref())
    }
}

impl From<FullBlock> for MultiBlock {
    fn from(block: FullBlock) -> Self {
        Self::Full(block)
    }
}

impl From<PartialBlock> for MultiBlock {
    fn from(block: PartialBlock) -> Self {
        Self::Partial(block)
    }
}

impl From<SealedHeader> for MultiBlock {
    fn from(header: SealedHeader) -> Self {
        Self::Header(header)
    }
}

impl From<Header> for MultiBlock {
    fn from(header: Header) -> Self {
        let sealed_header = header.seal_slow::<DefaultCrypto>();
        Self::Header(sealed_header)
    }
}

// A reference to a block
#[derive(Debug, Clone, Copy)]
pub struct BlockRef {
    pub number: u64,
    pub hash: H256,
}

impl Hash for BlockRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        <H256 as Hash>::hash(&self.hash, state);
    }
}

impl PartialEq for BlockRef {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for BlockRef {}

impl PartialOrd for BlockRef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for BlockRef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.hash == other.hash {
            return std::cmp::Ordering::Equal;
        }

        // First order by block number
        match self.number.cmp(&other.number) {
            Ordering::Equal => {
                // If the block number is the same, order by hash
                let this_parent = U256::from_big_endian(&self.hash.0);
                let other_parent = U256::from_big_endian(&other.hash.0);
                this_parent.cmp(&other_parent)
            },
            ordering => ordering,
        }
    }
}

impl From<&'_ MultiBlock> for BlockRef {
    fn from(block: &'_ MultiBlock) -> Self {
        block.as_block_ref()
    }
}

impl From<&'_ SealedHeader> for BlockRef {
    fn from(block: &'_ SealedHeader) -> Self {
        Self { number: block.number(), hash: block.hash() }
    }
}

impl From<&'_ PartialBlock> for BlockRef {
    fn from(block: &'_ PartialBlock) -> Self {
        Self::from(block.header())
    }
}

impl From<&'_ FullBlock> for BlockRef {
    fn from(block: &'_ FullBlock) -> Self {
        Self::from(block.header())
    }
}

impl From<&'_ BlockIdentifier> for BlockRef {
    fn from(identifier: &'_ BlockIdentifier) -> Self {
        Self { number: identifier.index, hash: H256(identifier.hash) }
    }
}

impl From<&'_ BlockOrIdentifier<BlockIdentifier>> for BlockRef {
    fn from(identifier: &'_ BlockOrIdentifier<BlockIdentifier>) -> Self {
        match identifier {
            BlockOrIdentifier::Identifier(id) => Self::from(id),
            BlockOrIdentifier::Block(block) => Self::from(&block.block_identifier),
        }
    }
}
