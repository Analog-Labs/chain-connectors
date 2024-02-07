use futures_util::{Stream, StreamExt};
use rosetta_config_ethereum::{ext::types::TransactionT, H256};
use std::{task::{Context, Poll}, pin::Pin};
use rosetta_ethereum_backend::{
    ext::types::{rpc::{RpcBlock, RpcTransaction}, AtBlock},
    EthereumRpc,
    jsonrpsee::core::client::Subscription,
};
use hashbrown::HashSet;
use crate::{
    log_filter::LogFilter,
    utils::{EthereumRpcExt, BlockFull}
};

pub struct BlockStream<B> {
    backend: B,
    log_filter: LogFilter,
    new_heads: Option<Subscription<BlockFull>>,
}

impl <B: EthereumRpc + EthereumRpcExt + Unpin + Send + Sync + 'static> Stream for BlockStream<B> {
    type Item = ();
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        
        if let Some(mut new_heads) = self.new_heads.take() {
            match new_heads.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(block))) => {
                    // New block

                },
                Poll::Ready(Some(Err(err))) => {
                    // Error handling
                },
                Poll::Ready(None) => {
                    // Stream ended
                },
                Poll::Pending => {
                    self.new_heads = Some(new_heads);
                },
            }
        }

        // let future = self.backend.block_with_uncles(AtBlock::Finalized);


        Poll::Pending
    }
}

fn get_events(filter: &LogFilter, block: &BlockFull) {
    let topics = filter.topics_from_bloom(block.header().header().logs_bloom);
    // Filter addresses which match at least one topic
    let logs = topics.filter_map(|(address, topics)| {
        if topics.next().is_some() {
            Some(address)
        } else {
            None
        }
    }).collect::<HashSet<_>>();
    let tx = block.body().transactions.iter().filter_map(|tx| {
        let Ok(logs) = tx.to() {

        }
        if logs.contains(&tx.to) {
            Some(tx.tx_hash)
        } else {
            None
        }
    });
}
