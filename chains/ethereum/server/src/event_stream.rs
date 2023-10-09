use ethers::{prelude::*, providers::PubsubClient};
use futures_util::{future::BoxFuture, FutureExt};
use rosetta_core::{stream::Stream, types::BlockIdentifier, BlockOrIdentifier, ClientEvent};
use std::{pin::Pin, sync::Arc, task::Poll};

// Maximum number of failures in sequence before closing the stream
const FAILURE_THRESHOLD: u32 = 10;

pub struct EthereumEventStream<'a, P: PubsubClient> {
    /// Ethereum client
    pub client: Arc<Provider<P>>,
    /// Ethereum subscription for new heads
    pub new_head: Option<SubscriptionStream<'a, P, Block<H256>>>,
    /// Count the number of failed attempts to retrieve the finalized block
    pub finalized_block_failures: u32,
    /// Count the number of failed attempts to retrieve the latest block
    pub latest_block_failures: u32,
    /// Cache the best finalized block, we use this to avoid emitting two
    /// [`ClientEvent::NewFinalized`] for the same block
    pub best_finalized_block: Option<BlockIdentifier>,
    /// Ethereum client doesn't support subscribing for finalized blocks, as workaround
    /// everytime we receive a new head, we query the latest finalized block
    pub finalized_block_future:
        Option<BoxFuture<'static, Result<Option<Block<TxHash>>, ProviderError>>>,
}

impl<'a, P> EthereumEventStream<'a, P>
where
    P: PubsubClient + 'static,
{
    pub fn new(
        client: Arc<Provider<P>>,
        subscription: SubscriptionStream<'a, P, Block<H256>>,
    ) -> Self {
        Self {
            client,
            new_head: Some(subscription),
            finalized_block_failures: 0,
            latest_block_failures: 0,
            best_finalized_block: None,
            finalized_block_future: None,
        }
    }

    fn finalized_block(&self) -> BoxFuture<'static, Result<Option<Block<TxHash>>, ProviderError>> {
        // Clone client to make BoxFuture 'static
        let client = Arc::clone(&self.client);
        async move { client.get_block(BlockId::Number(BlockNumber::Finalized)).await }.boxed()
    }
}

/// Converts [`Block`] to [`BlockIdentifier`]
fn block_to_identifier(block: &Block<TxHash>) -> Result<BlockIdentifier, &'static str> {
    let Some(number) = block.number else { return Err("block number is missing") };

    let Some(hash) = block.hash else { return Err("block hash is missing") };

    Ok(BlockIdentifier::new(number.as_u64(), hex::encode(hash)))
}

impl<'a, P> Unpin for EthereumEventStream<'a, P> where P: PubsubClient {}

impl<P> Stream for EthereumEventStream<'_, P>
where
    P: PubsubClient + 'static,
{
    type Item = ClientEvent;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = &mut *self;

        // Query the latest finalized block
        if let Some(mut finalized_block_future) = this.finalized_block_future.take() {
            loop {
                if this.finalized_block_failures >= FAILURE_THRESHOLD {
                    return Poll::Ready(Some(ClientEvent::Close(
                        "More than 10 failures in sequence".into(),
                    )));
                }

                match finalized_block_future.poll_unpin(cx) {
                    Poll::Ready(Ok(Some(block))) => {
                        // Convert raw block to block identifier
                        let block_identifier = match block_to_identifier(&block) {
                            Ok(block_identifier) => block_identifier,
                            Err(error) => {
                                this.finalized_block_failures += 1;
                                tracing::error!("finalized block: {error}");
                                break;
                            },
                        };

                        // Reset failure counter
                        this.finalized_block_failures = 0;

                        // Skip if the finalized block is equal to the best finalized block
                        if let Some(best_finalized_block) = this.best_finalized_block.take() {
                            if block_identifier == best_finalized_block {
                                tracing::debug!("finalized block unchanged");
                                this.best_finalized_block = Some(best_finalized_block);
                                break;
                            }
                        }

                        // Cache the new best finalized block
                        this.best_finalized_block = Some(block_identifier.clone());

                        // Return the best finalized block
                        return Poll::Ready(Some(ClientEvent::NewFinalized(
                            BlockOrIdentifier::Identifier(block_identifier),
                        )));
                    },
                    Poll::Ready(Ok(None)) => {
                        // Retry to retrieve the latest finalized block.
                        this.finalized_block_future = Some(this.finalized_block());
                        tracing::error!("finalized block not found");
                        this.finalized_block_failures += 1;
                        continue;
                    },
                    Poll::Ready(Err(error)) => {
                        // Retry to retrieve the latest finalized block.
                        this.finalized_block_future = Some(this.finalized_block());
                        tracing::error!("failed to retrieve finalized block: {error:?}");
                        this.finalized_block_failures += 1;
                        continue;
                    },
                    Poll::Pending => {
                        this.finalized_block_future = Some(finalized_block_future);
                        break;
                    },
                }
            }
        }

        let Some(mut new_head_stream) = this.new_head.take() else {
            return Poll::Ready(None);
        };

        // Query new heads
        loop {
            if this.latest_block_failures >= FAILURE_THRESHOLD {
                return Poll::Ready(Some(ClientEvent::Close(
                    "More than 10 failures in sequence".into(),
                )));
            }

            match new_head_stream.poll_next_unpin(cx) {
                Poll::Ready(Some(block)) => {
                    // Convert raw block to block identifier
                    let block_identifier = match block_to_identifier(&block) {
                        Ok(block_identifier) => block_identifier,
                        Err(error) => {
                            this.latest_block_failures += 1;
                            tracing::error!("latest block: {error}");
                            continue;
                        },
                    };

                    // Reset failure counter
                    this.latest_block_failures = 0;

                    // Query latest finalized block
                    if this.finalized_block_future.is_none() {
                        this.finalized_block_future = Some(this.finalized_block());
                    }

                    this.new_head = Some(new_head_stream);
                    return Poll::Ready(Some(ClientEvent::NewHead(BlockOrIdentifier::Identifier(
                        block_identifier,
                    ))));
                },
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => {
                    this.new_head = Some(new_head_stream);
                    return Poll::Pending;
                },
            };
        }
    }
}
