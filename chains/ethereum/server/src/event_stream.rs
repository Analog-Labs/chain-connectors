use crate::{client::EthereumClient, utils::NonPendingBlock};
use ethers::{prelude::*, providers::PubsubClient};
use futures_util::{future::BoxFuture, FutureExt};
use rosetta_core::{stream::Stream, BlockOrIdentifier, ClientEvent};
use std::{cmp::Ordering, pin::Pin, task::Poll};

// Maximum number of failures in sequence before closing the stream
const FAILURE_THRESHOLD: u32 = 10;

pub struct EthereumEventStream<'a, P: PubsubClient + 'static> {
    /// Ethereum subscription for new heads
    new_head_stream: Option<SubscriptionStream<'a, P, Block<H256>>>,
    /// Finalized blocks stream
    finalized_stream: Option<FinalizedBlockStream<'a, P>>,
    /// Count the number of failed attempts to retrieve the latest block
    failures: u32,
}

impl<P> EthereumEventStream<'_, P>
where
    P: PubsubClient + 'static,
{
    pub fn new<'a>(
        client: &'a EthereumClient<P>,
        subscription: SubscriptionStream<'a, P, Block<H256>>,
    ) -> EthereumEventStream<'a, P> {
        EthereumEventStream {
            new_head_stream: Some(subscription),
            finalized_stream: Some(FinalizedBlockStream::new(client)),
            failures: 0,
        }
    }
}

impl<P> Stream for EthereumEventStream<'_, P>
where
    P: PubsubClient + 'static,
{
    type Item = ClientEvent;

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
            Poll::Ready(Some(Ok(block))) => {
                self.finalized_stream = Some(finalized_stream);
                return Poll::Ready(Some(ClientEvent::NewFinalized(block.identifier.into())));
            },
            Poll::Ready(Some(Err(error))) => {
                self.new_head_stream = None;
                return Poll::Ready(Some(ClientEvent::Close(error)));
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
                    let block = match NonPendingBlock::try_from(block) {
                        Ok(block) => block,
                        Err(error) => {
                            self.failures += 1;
                            tracing::error!("[RPC BUG] invalid latest block: {error}");
                            continue;
                        },
                    };

                    // Reset failure counter
                    self.failures = 0;

                    // Store the new latest block
                    if let Some(finalized_stream) = self.finalized_stream.as_mut() {
                        finalized_stream.update_latest_block(block.number);
                    }

                    self.new_head_stream = Some(new_head_stream);
                    return Poll::Ready(Some(ClientEvent::NewHead(BlockOrIdentifier::Identifier(
                        block.identifier,
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

struct FinalizedBlockStream<'a, P>
where
    P: PubsubClient + 'static,
{
    /// Ethereum client used to retrieve the finalized block
    client: &'a EthereumClient<P>,
    /// Cache the latest block, used for retrieve the latest finalized block
    /// see [`BlockFinalityStrategy`]
    latest_block: Option<u64>,
    /// Ethereum client doesn't support subscribing for finalized blocks, as workaround
    /// everytime we receive a new head, we query the latest finalized block
    future: Option<BoxFuture<'a, anyhow::Result<NonPendingBlock>>>,
    /// Cache the best finalized block, we use this to avoid emitting two
    /// [`ClientEvent::NewFinalized`] for the same block
    best_finalized_block: Option<NonPendingBlock>,
    /// Count the number of failed attempts to retrieve the finalized block
    failures: u32,
    /// Waker used to wake up the stream when a new block is available
    waker: Option<std::task::Waker>,
}

impl<'a, P> FinalizedBlockStream<'a, P>
where
    P: PubsubClient + 'static,
{
    pub fn new(client: &EthereumClient<P>) -> FinalizedBlockStream<'_, P> {
        FinalizedBlockStream {
            client,
            latest_block: None,
            future: None,
            best_finalized_block: None,
            failures: 0,
            waker: None,
        }
    }

    pub fn update_latest_block(&mut self, number: u64) {
        if Some(number) == self.latest_block {
            return;
        }
        self.latest_block = Some(number);
        if self.future.is_none() {
            self.future = Some(self.finalized_block());
        }
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    fn finalized_block<'c>(&'c self) -> BoxFuture<'a, anyhow::Result<NonPendingBlock>> {
        self.client.finalized_block(self.latest_block).boxed()
    }
}

impl<P> Stream for FinalizedBlockStream<'_, P>
where
    P: PubsubClient + 'static,
{
    type Item = Result<NonPendingBlock, String>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            // Check the failure count
            match self.failures.cmp(&FAILURE_THRESHOLD) {
                Ordering::Greater => return Poll::Ready(None),
                Ordering::Equal => {
                    self.failures += 1;
                    self.future = None;
                    return Poll::Ready(Some(Err(format!(
                        "More than {FAILURE_THRESHOLD} failures in sequence",
                    ))));
                },
                Ordering::Less => {},
            }

            // If the future is not ready, store the waker and return pending
            let Some(mut future) = self.future.take() else {
                self.waker = Some(cx.waker().clone());
                return Poll::Pending;
            };

            match future.poll_unpin(cx) {
                Poll::Ready(Ok(block)) => {
                    // Store the waker
                    self.waker = Some(cx.waker().clone());

                    // Skip if the finalized block is equal to the best finalized block
                    if let Some(best_finalized_block) = self.best_finalized_block.take() {
                        if block.hash == best_finalized_block.hash {
                            tracing::debug!("finalized block unchanged");
                            self.best_finalized_block = Some(best_finalized_block);
                            break Poll::Pending;
                        }
                    }

                    // Cache the new best finalized block
                    self.best_finalized_block = Some(block.clone());

                    // Return the best finalized block
                    break Poll::Ready(Some(Ok(block)));
                },
                Poll::Ready(Err(error)) => {
                    // Increment failure count
                    self.failures += 1;
                    tracing::error!(
                        "failed to retrieve finalized block: {error:?} {}",
                        self.failures
                    );

                    // Retry to retrieve the latest finalized block.
                    self.future = Some(self.finalized_block());
                    continue;
                },
                Poll::Pending => {
                    self.future = Some(future);
                    break Poll::Pending;
                },
            }
        }
    }
}
