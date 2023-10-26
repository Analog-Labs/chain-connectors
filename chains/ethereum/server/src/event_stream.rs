use crate::{client::EthereumClient, utils::NonPendingBlock};
use ethers::{prelude::*, providers::PubsubClient};
use futures_util::{future::BoxFuture, FutureExt};
use rosetta_core::{stream::Stream, BlockOrIdentifier, ClientEvent};
use std::{pin::Pin, task::Poll};

// Maximum number of failures in sequence before closing the stream
const FAILURE_THRESHOLD: u32 = 10;

#[pin_project::pin_project(project=EthereumEventStreamProjection)]
pub struct EthereumEventStream<'a, P: PubsubClient> {
    /// Ethereum client
    pub client: &'a crate::EthereumClient<P>,
    /// Ethereum subscription for new heads
    pub new_head: Option<SubscriptionStream<'a, P, Block<H256>>>,
    /// Count the number of failed attempts to retrieve the finalized block
    pub finalized_block_failures: u32,
    /// Count the number of failed attempts to retrieve the latest block
    pub latest_block_failures: u32,
    /// Cache the best finalized block, we use this to avoid emitting two
    /// [`ClientEvent::NewFinalized`] for the same block
    pub best_finalized_block: Option<NonPendingBlock>,
    /// Cache the latest block, used for retrieve the latest finalized block
    /// see [`BlockFinalityStrategy`]
    pub latest_block: Option<NonPendingBlock>,
    /// Ethereum client doesn't support subscribing for finalized blocks, as workaround
    /// everytime we receive a new head, we query the latest finalized block
    pub finalized_block_future: Option<BoxFuture<'a, anyhow::Result<NonPendingBlock>>>,
}

impl<P> EthereumEventStream<'_, P>
where
    P: PubsubClient + 'static,
{
    pub fn new<'a>(
        client: &'a crate::EthereumClient<P>,
        subscription: SubscriptionStream<'a, P, Block<H256>>,
    ) -> EthereumEventStream<'a, P> {
        EthereumEventStream {
            client,
            new_head: Some(subscription),
            finalized_block_failures: 0,
            latest_block_failures: 0,
            best_finalized_block: None,
            finalized_block_future: None,
            latest_block: None,
        }
    }
}

impl<'a, 'b, P> EthereumEventStreamProjection<'a, 'b, P>
where
    P: PubsubClient + 'static,
{
    fn finalized_block(&self) -> BoxFuture<'b, anyhow::Result<NonPendingBlock>> {
        let latest_block_number = self.latest_block.as_ref().map(|block| block.number);
        let client = &self.client;
        EthereumClient::finalized_block(client, latest_block_number).boxed()
    }
}

impl<P> Stream for EthereumEventStream<'_, P>
where
    P: PubsubClient + 'static,
{
    type Item = ClientEvent;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // Check if the stream is close
        if self.new_head.is_none() && self.finalized_block_future.is_none() {
            return Poll::Ready(None);
        }

        let this = self.project();

        // Query the latest finalized block
        loop {
            let Some(mut finalized_block_future) = this.finalized_block_future.take() else {
                break;
            };
            match finalized_block_future.poll_unpin(cx) {
                Poll::Ready(Ok(block)) => {
                    // Reset failure counter
                    *this.finalized_block_failures = 0;

                    // Skip if the finalized block is equal to the best finalized block
                    if let Some(best_finalized_block) = this.best_finalized_block.take() {
                        if block.hash == best_finalized_block.hash {
                            tracing::debug!("finalized block unchanged");
                            *this.best_finalized_block = Some(best_finalized_block);
                        }
                    }

                    // Cache the new best finalized block
                    *this.best_finalized_block = Some(block.clone());

                    // Return the best finalized block
                    return Poll::Ready(Some(ClientEvent::NewFinalized(
                        BlockOrIdentifier::Identifier(block.identifier),
                    )));
                },
                Poll::Ready(Err(error)) => {
                    // Check failure count
                    *this.finalized_block_failures += 1;
                    tracing::error!("failed to retrieve finalized block: {error:?}");
                    if *this.finalized_block_failures >= FAILURE_THRESHOLD {
                        *this.finalized_block_future = None;
                        return Poll::Ready(Some(ClientEvent::Close(
                            "More than 10 failures in sequence".into(),
                        )));
                    }

                    // Retry to retrieve the latest finalized block.
                    *this.finalized_block_future = Some(this.finalized_block());
                    continue;
                },
                Poll::Pending => {
                    *this.finalized_block_future = Some(finalized_block_future);
                    break;
                },
            }
        }

        let Some(mut new_head_stream) = this.new_head.take() else {
            *this.finalized_block_future = None;
            return Poll::Ready(None);
        };

        // Query new heads
        loop {
            if *this.latest_block_failures >= FAILURE_THRESHOLD {
                return Poll::Ready(Some(ClientEvent::Close(
                    "More than 10 failures in sequence".into(),
                )));
            }

            match new_head_stream.poll_next_unpin(cx) {
                Poll::Ready(Some(block)) => {
                    // Convert raw block to block identifier
                    let block = match NonPendingBlock::try_from(block) {
                        Ok(block) => block,
                        Err(error) => {
                            *this.latest_block_failures += 1;
                            tracing::error!("[RPC BUG] invalid latest block: {error}");
                            continue;
                        },
                    };

                    // Reset failure counter
                    *this.latest_block_failures = 0;

                    // Store the new latest block
                    *this.latest_block = Some(block.clone());

                    // Query latest finalized block
                    if this.finalized_block_future.is_none() {
                        *this.finalized_block_future = Some(this.finalized_block());
                    }

                    *this.new_head = Some(new_head_stream);
                    return Poll::Ready(Some(ClientEvent::NewHead(BlockOrIdentifier::Identifier(
                        block.identifier,
                    ))));
                },
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => {
                    *this.new_head = Some(new_head_stream);
                    return Poll::Pending;
                },
            };
        }
    }
}
