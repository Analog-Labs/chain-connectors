use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{Future, Stream, StreamExt};
use hashbrown::{HashMap, HashSet};
use rosetta_config_ethereum::{ext::types::SealedBlock, H256};
use tracing::Level;

use super::{
    block_fetcher::{BlockFetcher, RequestBlock},
    multi_block::{BlockRef, MultiBlock},
};

/// Maximum blocks to store in the import queue.
const MAX_IMPORTING_BLOCKS: usize = 2048;

/// Maximum blocks to download ahead of any gap.
const MAX_DOWNLOAD_AHEAD: u32 = 2048;

/// Status of a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockSyncStatus {
    Syncing,
    Complete,
    Queued,
}

/// Block data with status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockData {
    /// The Block Message from the wire
    pub block: MultiBlock,
    /// The peer, we received this from
    pub status: BlockSyncStatus,
}

pub struct ChainSync<F, Fut> {
    block_fetcher: BlockFetcher<F, Fut>,
    /// A collection of blocks that are being downloaded from peers
    blocks: HashMap<H256, BlockData>,
    /// The best block in our queue of blocks to import
    best_block: BlockRef,
    /// A set of hashes of blocks that are being downloaded or have been
    /// downloaded and are queued for import.
    queue_blocks: HashSet<BlockRef>,
}

impl<F, Fut> ChainSync<F, Fut> {
    pub fn new(block_fetcher: BlockFetcher<F, Fut>) -> Self {
        Self {
            block_fetcher,
            blocks: HashMap::new(),
            best_block: BlockRef::default(),
            queue_blocks: HashSet::new(),
        }
    }
}

impl<F> Stream for ChainSync<F, F::Future>
where
    F: RequestBlock + Unpin,
    F::Error: core::fmt::Debug,
    F::Future: Future<Output = Result<SealedBlock<H256>, F::Error>>,
{
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.block_fetcher.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(block))) => {
                let block_ref = BlockRef::from(&block);
                tracing::event!(
                    Level::DEBUG,
                    event = "ImportBlock",
                    best_block_number = self.best_block.number,
                    best_block_hash = %self.best_block.hash,
                    block_number = block_ref.number,
                    block_hash = %block_ref.hash,
                );
                if !self.queue_blocks.remove(&block_ref) {
                    tracing::warn!("block not in the");
                }
                self.blocks.insert(block_ref.hash, block.into());
            },
            Poll::Ready(Some(Err(err))) => {
                tracing::event!(Level::WARN, "Error fetching block: {:?}", err);
            },
            Poll::Ready(None) => {},
            Poll::Pending => {},
        };
        Poll::Pending
    }
}
