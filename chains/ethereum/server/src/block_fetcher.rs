#![allow(dead_code)]
use super::fns::FnMut1;
use std::{
    collections::btree_set::BTreeSet,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{stream::FuturesUnordered, Future, Stream};
use rosetta_config_ethereum::{ext::types::SealedBlock, H256};

use crate::multi_block::BlockRef;

pub trait RequestBlock: Unpin {
    type Error;
    type Future: Future<Output = Result<SealedBlock<H256>, Self::Error>>;
    fn get_block(&mut self, block_ref: BlockRef) -> Self::Future;
}

impl<F, ERR, Fut> RequestBlock for F
where
    F: FnMut1<BlockRef, Output = Fut> + Unpin,
    Fut: Future<Output = Result<SealedBlock<H256>, ERR>>,
{
    type Error = ERR;
    type Future = Fut;
    fn get_block(&mut self, block_ref: BlockRef) -> Self::Future {
        self.call_mut(block_ref)
    }
}

pub struct BlockFetcher<F, Fut> {
    callback: F,
    capacity: usize,
    pending: BTreeSet<BlockRef>,
    fut: FuturesUnordered<BlockFuture<Fut>>,
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
            self.fut
                .push(BlockFuture { block_ref, future: self.callback.get_block(block_ref) });
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn len(&self) -> usize {
        self.fut.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fut.is_empty()
    }
}

impl<F: RequestBlock> Stream for BlockFetcher<F, F::Future> {
    type Item = Result<SealedBlock<H256>, F::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.fut.is_empty() {
            return Poll::Ready(None);
        }
        let future = unsafe { Pin::new_unchecked(&mut self.fut) };
        match future.poll_next(cx) {
            Poll::Ready(Some((block_ref, result))) => {
                self.pending.remove(&block_ref);
                Poll::Ready(Some(result))
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.fut.len(), None)
    }
}

struct BlockFuture<Fut> {
    block_ref: BlockRef,
    future: Fut,
}

impl<Fut> Unpin for BlockFuture<Fut> where Fut: Unpin {}

impl<Fut> Future for BlockFuture<Fut>
where
    Fut: Future,
{
    type Output = (BlockRef, Fut::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        let pinned = unsafe { Pin::new_unchecked(&mut this.future) };
        match pinned.poll(cx) {
            Poll::Ready(output) => Poll::Ready((this.block_ref, output)),
            Poll::Pending => Poll::Pending,
        }
    }
}
