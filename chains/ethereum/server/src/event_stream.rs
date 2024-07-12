use super::{finalized_block_stream::FinalizedBlockStream, new_heads::NewHeadsStream};
use futures_util::StreamExt;
use rosetta_config_ethereum::Event;
use rosetta_core::{stream::Stream, types::BlockIdentifier, BlockOrIdentifier, ClientEvent};
use rosetta_ethereum_backend::{
    ext::types::{rpc::RpcBlock, H256},
    jsonrpsee::core::client::{error::Error as RpcError, Subscription},
    EthereumPubSub,
};
use std::{pin::Pin, task::Poll};

pub struct EthereumEventStream<C>
where
    C: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
    C::SubscriptionError: Send + Sync,
{
    /// Latest block stream
    new_head_stream: Option<NewHeadsStream<C>>,
    /// Finalized blocks stream
    finalized_stream: Option<FinalizedBlockStream<C>>,
}

impl<C> EthereumEventStream<C>
where
    C: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
    C::SubscriptionError: Send + Sync,
{
    pub fn new(client: C) -> Self {
        Self {
            new_head_stream: Some(NewHeadsStream::new(client.clone())),
            finalized_stream: Some(FinalizedBlockStream::new(client)),
        }
    }
}

impl<C> Stream for EthereumEventStream<C>
where
    C: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
    C::SubscriptionError: Send + Sync,
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

        match new_head_stream.poll_next_unpin(cx) {
            Poll::Ready(Some(block)) => {
                // Convert block to block identifier
                let block = {
                    let header = block.header();
                    BlockIdentifier::new(header.number(), header.hash().0)
                };

                self.new_head_stream = Some(new_head_stream);
                Poll::Ready(Some(ClientEvent::NewHead(BlockOrIdentifier::Identifier(block))))
            },
            Poll::Ready(None) => {
                self.finalized_stream = None;
                Poll::Ready(None)
            },
            Poll::Pending => {
                self.new_head_stream = Some(new_head_stream);
                Poll::Pending
            },
        }
    }
}
