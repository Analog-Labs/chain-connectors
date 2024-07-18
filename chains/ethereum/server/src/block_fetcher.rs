#![allow(dead_code)]
use super::fns::FnMut1;
use std::{
    collections::btree_set::BTreeSet,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{stream::FuturesUnordered, Future, Stream, StreamExt};
use rosetta_config_ethereum::{ext::types::SealedBlock, H256};

use crate::multi_block::BlockRef;

pub trait RequestBlock: Unpin {
    type Future: Future<Output = SealedBlock<H256>>;
    fn get_block(&mut self, block_ref: BlockRef) -> Self::Future;
}

impl<F, Fut> RequestBlock for F
where
    F: FnMut1<BlockRef, Output = Fut> + Unpin,
    Fut: Future<Output = SealedBlock<H256>>,
{
    type Future = Fut;
    fn get_block(&mut self, block_ref: BlockRef) -> Self::Future {
        self.call_mut(block_ref)
    }
}

pub struct BlockFetcher<F, Fut> {
    callback: F,
    capacity: usize,
    pending: BTreeSet<BlockRef>,
    fut: FuturesUnordered<Fut>,
}

impl<F: RequestBlock> BlockFetcher<F, F::Future> {
    pub fn new(callback: F, capacity: usize) -> Self {
        Self { callback, capacity, pending: BTreeSet::new(), fut: FuturesUnordered::new() }
    }

    pub fn fetch(&mut self, block_ref: BlockRef) -> Result<(), ()> {
        if self.pending.contains(&block_ref) {
            Ok(())
        } else if self.fut.len() < self.capacity {
            self.pending.insert(block_ref);
            self.fut.push(self.callback.get_block(block_ref));
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<F: RequestBlock> Stream for BlockFetcher<F, F::Future> {
    type Item = SealedBlock<H256>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.fut.is_empty() {
            return Poll::Ready(None);
        }
        match self.fut.poll_next_unpin(cx) {
            Poll::Ready(Some(block)) => {
                let block_ref = BlockRef::from(&block);
                self.pending.remove(&block_ref);
                Poll::Ready(Some(block))
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
