#![allow(dead_code)]
use super::{
    event_stream::{EthereumEventStream, NewBlock},
    state::State,
};
use futures_util::StreamExt;
use rosetta_config_ethereum::Event as EthEvent;
use rosetta_core::{stream::Stream, types::BlockIdentifier, BlockOrIdentifier, ClientEvent};
use rosetta_ethereum_backend::{
    ext::types::{rpc::RpcBlock, H256},
    jsonrpsee::core::{client::Subscription, ClientError as RpcError},
    EthereumPubSub,
};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub struct BlockStream<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
    RPC::SubscriptionError: Send + Sync,
{
    block_stream: Option<EthereumEventStream<RPC>>,
    state: State,
}

impl<RPC> BlockStream<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
    RPC::SubscriptionError: Send + Sync,
{
    #[must_use]
    pub fn new(client: RPC, state: State) -> Self {
        Self { block_stream: Some(EthereumEventStream::new(client)), state }
    }
}

impl<RPC> Stream for BlockStream<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
    RPC::SubscriptionError: Send + Sync,
{
    type Item = ClientEvent<BlockIdentifier, EthEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Some(mut block_stream) = self.block_stream.take() else {
            return Poll::Ready(None);
        };

        let mut failures = 0;
        loop {
            match block_stream.poll_next_unpin(cx) {
                Poll::Ready(Some(new_block)) => {
                    let block_id = {
                        let header = new_block.sealed_block().header();
                        BlockOrIdentifier::Identifier(BlockIdentifier {
                            index: header.number(),
                            hash: header.hash().0,
                        })
                    };
                    let is_finalized = matches!(new_block, NewBlock::Finalized(_));
                    if let Err(err) = self.state.import(new_block.into_sealed_block()) {
                        failures += 1;
                        tracing::warn!("failed to import block {block_id} ({failures}): {:?}", err);
                        if failures >= 5 {
                            return Poll::Ready(None);
                        }
                        continue;
                    }

                    let event = if is_finalized {
                        ClientEvent::NewFinalized(block_id)
                    } else {
                        ClientEvent::NewHead(block_id)
                    };
                    self.block_stream = Some(block_stream);
                    break Poll::Ready(Some(event));
                },
                Poll::Ready(None) => break Poll::Ready(None),
                Poll::Pending => {
                    self.block_stream = Some(block_stream);
                    break Poll::Pending;
                },
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}
