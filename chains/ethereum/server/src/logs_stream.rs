use crate::{finalized_block_stream::FinalizedBlockStream, utils::LogErrorExt, state::State};
use futures_util::{future::BoxFuture, FutureExt, Stream, StreamExt};
use rosetta_ethereum_backend::{
    ext::types::{crypto::DefaultCrypto, rpc::RpcBlock, SealedBlock, H256},
    jsonrpsee::core::client::{Error as RpcError, Subscription},
    EthereumPubSub, EthereumRpc,
};
use rosetta_utils::jsonrpsee::{AutoSubscribe, RetrySubscription};
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

/// Default polling interval for checking for new finalized blocks.
const DEFAULT_POLLING_INTERVAL: Duration = Duration::from_secs(5);

type BlockRef = SealedBlock<H256, H256>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewBlockEvent {
    Pending(SealedBlock<H256>),
    Finalized(BlockRef),
}

struct RetryNewHeadsSubscription<RPC> {
    backend: RPC,
}

impl<RPC> RetryNewHeadsSubscription<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
{
    pub const fn new(backend: RPC) -> Self {
        Self { backend }
    }
}

/// A stream which emits new blocks and logs matching a filter.
pub struct LogStream<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
{
    /// Configuration for the stream.
    state: State,

    /// Timestamp when the last block was received.
    last_block_timestamp: Option<Instant>,

    /// Subscription to new block headers.
    /// Obs: This only emits pending blocks headers, not latest or finalized ones.
    new_heads_sub: Option<AutoSubscribe<RetryNewHeadsSubscription<RPC>>>,

    /// Subscription to new finalized blocks, the stream guarantees that new finalized blocks are
    /// monotonically increasing.
    finalized_blocks_stream: FinalizedBlockStream<RPC>,

    /// Count of consecutive errors.
    consecutive_errors: u32,
}

impl<RPC> LogStream<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + EthereumRpc
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
{
    pub fn new(backend: RPC, config: Config) -> Self {
        let subscriber = RetryNewHeadsSubscription::new(backend.clone());
        Self {
            config,
            last_block_timestamp: None,
            new_heads_sub: Some(AutoSubscribe::new(Duration::from_secs(5), subscriber)),
            finalized_blocks_stream: FinalizedBlockStream::new(backend),
            consecutive_errors: 0,
        }
    }
}

impl<RPC> Stream for LogStream<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + EthereumRpc
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
{
    type Item = NewBlockEvent;

    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // 1 - Poll finalized blocks
        // 2 - Poll new heads subscription.

        // Poll latest finalized block
        match self.finalized_blocks_stream.poll_next_unpin(cx) {
            Poll::Ready(Some(finalized_block)) => {
                return Poll::Ready(Some(NewBlockEvent::Finalized(finalized_block)));
            },
            Poll::Ready(None) => {
                tracing::error!(
                    "[report this bug] finalized block stream should never return none"
                );
            },
            Poll::Pending => {},
        }

        // Check if the new heads subscription has been terminated.
        let terminated = self.new_heads_sub.as_ref().is_some_and(AutoSubscribe::terminated);
        if terminated {
            self.new_heads_sub = None;
            return Poll::Pending;
        }

        // Poll new heads subscription
        let Some(Poll::Ready(result)) =
            self.new_heads_sub.as_mut().map(|sub| sub.poll_next_unpin(cx))
        else {
            return Poll::Pending;
        };
        match result {
            // New block header
            Some(Ok(block)) => {
                // Reset error counter.
                self.consecutive_errors = 0;

                // Update last block timestamp.
                self.last_block_timestamp = Some(Instant::now());

                // Calculate header hash and return it.
                let block = block.seal_slow::<DefaultCrypto>();
                Poll::Ready(Some(NewBlockEvent::Pending(block)))
            },

            // Subscription returned an error
            Some(Err(err)) => {
                self.consecutive_errors += 1;
                if self.consecutive_errors >= self.config.stream_error_threshold {
                    // Consecutive error threshold reached, unsubscribe and close
                    // the stream.
                    tracing::error!(
                        "new heads stream returned too many consecutive errors: {}",
                        err.truncate()
                    );
                    if let Some(sub) = self.new_heads_sub.as_mut() {
                        sub.unsubscribe();
                    };
                } else {
                    tracing::error!("new heads stream error: {}", err.truncate());
                }
                Poll::Pending
            },

            // Stream ended
            None => {
                tracing::warn!(
                    "new heads subscription terminated, will poll new blocks every {:?}",
                    self.config.polling_interval
                );
                Poll::Pending
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MaybeWsEthereumClient;
    use futures_util::StreamExt;
    use rosetta_core::BlockchainConfig;
    use rosetta_docker::{run_test, Env};

    pub async fn client_from_config(
        config: BlockchainConfig,
    ) -> anyhow::Result<MaybeWsEthereumClient> {
        let url = config.node_uri.to_string();
        MaybeWsEthereumClient::from_config(config, url.as_str(), None).await
    }

    #[tokio::test]
    async fn block_stream_works() -> anyhow::Result<()> {
        let config = rosetta_config_ethereum::config("dev").unwrap();
        let env = Env::new("block-stream", config.clone(), client_from_config).await.unwrap();

        run_test(env, |env| async move {
            let client = match env.node().as_ref() {
                MaybeWsEthereumClient::Http(_) => panic!("the connections must be ws"),
                MaybeWsEthereumClient::Ws(client) => client.backend.clone(),
            };
            let config = Config {
                polling_interval: Duration::from_secs(1),
                stream_error_threshold: 5,
                unfinalized_cache_capacity: 10,
            };
            let mut sub = BlockSubscription::new(client, config);

            let mut best_finalized_block: Option<SealedBlock<H256>> = None;
            let mut latest_block: Option<SealedBlock<H256>> = None;
            for _ in 0..30 {
                let Some(new_block) = sub.next().await else {
                    panic!("stream ended");
                };
                match new_block {
                    NewBlockEvent::Finalized(new_block) => {
                        // println!("new finalized block: {:?}", new_block.header().number());
                        if let Some(best_finalized_block) = best_finalized_block.as_ref() {
                            let last_number = best_finalized_block.header().number();
                            let new_number = new_block.header().number();
                            assert!(new_number > last_number);
                            if new_number == (last_number + 1) {
                                assert_eq!(
                                    best_finalized_block.header().hash(),
                                    new_block.header().header().parent_hash
                                );
                            }
                        }
                        best_finalized_block = Some(new_block);
                    },
                    NewBlockEvent::Pending(new_block) => {
                        // println!("new pending block: {:?}", new_block.header().number());
                        if let Some(latest_block) = latest_block.as_ref() {
                            let last_number = latest_block.header().number();
                            let new_number = new_block.header().number();
                            assert!(new_number > last_number);
                            if new_number == (last_number + 1) {
                                assert_eq!(
                                    latest_block.header().hash(),
                                    new_block.header().header().parent_hash
                                );
                            }
                        }
                        latest_block = Some(new_block);
                    },
                }
            }
        })
        .await;
        Ok(())
    }
}
