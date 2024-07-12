use super::finalized_block_stream::FinalizedBlockStream;
use futures_util::StreamExt;
use rosetta_config_ethereum::Event;
use rosetta_core::{stream::Stream, types::BlockIdentifier, BlockOrIdentifier, ClientEvent};
use rosetta_ethereum_backend::{
    ext::types::{crypto::DefaultCrypto, rpc::RpcBlock, H256},
    jsonrpsee::{
        core::client::{Subscription, SubscriptionClientT},
        Adapter,
    },
};
use std::{pin::Pin, task::Poll};

// Maximum number of failures in sequence before closing the stream
const FAILURE_THRESHOLD: u32 = 10;

pub struct EthereumEventStream<P: SubscriptionClientT + Unpin + Clone + Send + Sync + 'static> {
    /// Ethereum subscription for new heads
    new_head_stream: Option<Subscription<RpcBlock<H256>>>,
    /// Finalized blocks stream
    finalized_stream: Option<FinalizedBlockStream<Adapter<P>>>,
    /// Count the number of failed attempts to retrieve the latest block
    failures: u32,
}

impl<P> EthereumEventStream<P>
where
    P: SubscriptionClientT + Unpin + Clone + Send + Sync + 'static,
{
    pub fn new(client: P, subscription: Subscription<RpcBlock<H256>>) -> Self {
        Self {
            new_head_stream: Some(subscription),
            finalized_stream: Some(FinalizedBlockStream::new(Adapter(client))),
            failures: 0,
        }
    }
}

impl<P> Stream for EthereumEventStream<P>
where
    P: SubscriptionClientT + Unpin + Clone + Send + Sync + 'static,
{
    type Item = ClientEvent<BlockIdentifier, Event>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // Check if the stream is close
        let Some(mut finalized_stream) = self.finalized_stream.take() else {
            return Poll::Ready(None);
        };

        // Poll the finalized block stream
        match finalized_stream.poll_next_unpin(cx) {
            Poll::Ready(Some(block)) => {
                self.finalized_stream = Some(finalized_stream);
                return Poll::Ready(Some(ClientEvent::NewFinalized(
                    BlockOrIdentifier::Identifier(BlockIdentifier::new(
                        block.header().header().number,
                        block.header().hash().0,
                    )),
                )));
            },
            Poll::Ready(None) => {
                self.new_head_stream = None;
                return Poll::Ready(None);
            },
            Poll::Pending => {
                self.finalized_stream = Some(finalized_stream);
            },
        }

        // Poll the new head stream
        let Some(mut new_head_stream) = self.new_head_stream.take() else {
            self.finalized_stream = None;
            return Poll::Ready(None);
        };

        loop {
            if self.failures >= FAILURE_THRESHOLD {
                self.new_head_stream = None;
                self.finalized_stream = None;
                return Poll::Ready(Some(ClientEvent::Close(
                    "More than 10 failures in sequence".into(),
                )));
            }

            match new_head_stream.poll_next_unpin(cx) {
                Poll::Ready(Some(block)) => {
                    // Convert raw block to block identifier
                    let block = match block {
                        Ok(block) => {
                            let header = if let Some(hash) = block.hash {
                                block.header.seal(hash)
                            } else {
                                block.header.seal_slow::<DefaultCrypto>()
                            };
                            BlockIdentifier::new(header.number(), header.hash().0)
                        },
                        Err(error) => {
                            self.failures += 1;
                            tracing::error!("[RPC BUG] invalid latest block: {error}");
                            continue;
                        },
                    };

                    // Reset failure counter
                    self.failures = 0;

                    self.new_head_stream = Some(new_head_stream);
                    return Poll::Ready(Some(ClientEvent::NewHead(BlockOrIdentifier::Identifier(
                        block,
                    ))));
                },
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => {
                    self.new_head_stream = Some(new_head_stream);
                    break Poll::Pending;
                },
            };
        }
    }
}
