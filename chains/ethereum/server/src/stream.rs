#![allow(dead_code)]
use crate::{finalized_block_stream::FinalizedBlockStream, utils::LogErrorExt};
use futures_timer::Delay;
use futures_util::{future::BoxFuture, FutureExt, Stream, StreamExt};
use rosetta_ethereum_backend::{
    ext::types::{crypto::DefaultCrypto, rpc::RpcBlock, SealedBlock, H256},
    jsonrpsee::core::client::{Error as RpcError, Subscription},
    EthereumPubSub, EthereumRpc,
};
use serde::de::DeserializeOwned;
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

pub trait RetrySubscription: Unpin + Send + Sync + 'static {
    type Item: DeserializeOwned + Send + Sync + 'static;
    fn retry_subscribe(&self) -> BoxFuture<'static, Result<Subscription<Self::Item>, RpcError>>;
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

impl<RPC> RetrySubscription for RetryNewHeadsSubscription<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
{
    type Item = RpcBlock<H256>;

    fn retry_subscribe(&self) -> BoxFuture<'static, Result<Subscription<Self::Item>, RpcError>> {
        let client = self.backend.clone();
        async move { client.new_heads().await }.boxed()
    }
}

/// Manages the subscription's state
enum SubscriptionState<T> {
    /// Currently subscribing
    Subscribing(BoxFuture<'static, Result<Subscription<T>, RpcError>>),
    /// Subscription is active.
    Subscribed(Subscription<T>),
    /// Previous subscribe attempt failed, retry after delay.
    ResubscribeAfterDelay(Delay),
}

/// A stream which auto resubscribe when closed
pub struct AutoSubscribe<T: RetrySubscription> {
    /// Retry subscription
    retry_subscription: T,
    /// Subscription state
    state: Option<SubscriptionState<T::Item>>,
    /// Count of consecutive errors.
    pub consecutive_subscription_errors: u32,
    /// Total number of successful subscriptions.
    pub total_subscriptions: u32,
    /// Min interval between subscription attemps
    pub retry_interval: Duration,
    /// The timestamp of the last successful subscription.
    pub last_subscription_timestamp: Option<Instant>,
}

impl<T> AutoSubscribe<T>
where
    T: RetrySubscription,
{
    pub fn new(retry_subscription: T, retry_interval: Duration) -> Self {
        let fut = retry_subscription.retry_subscribe();
        Self {
            retry_subscription,
            state: Some(SubscriptionState::Subscribing(fut)),
            consecutive_subscription_errors: 0,
            total_subscriptions: 0,
            retry_interval,
            last_subscription_timestamp: None,
        }
    }

    /// Unsubscribe and consume the subscription.
    pub async fn unsubscribe(mut self) -> Result<(), RpcError> {
        let Some(state) = self.state.take() else {
            return Ok(());
        };
        match state {
            // Subscribing... wait for it to finish then unsubscribe.
            SubscriptionState::Subscribing(fut) => {
                if let Ok(subscription) = fut.await {
                    subscription.unsubscribe().await?;
                }
            },
            // Subscribed, start unsubscribe.
            SubscriptionState::Subscribed(subscription) => {
                subscription.unsubscribe().await?;
            },
            SubscriptionState::ResubscribeAfterDelay(_) => {},
        }
        Ok(())
    }
}

impl<T> Stream for AutoSubscribe<T>
where
    T: RetrySubscription,
{
    type Item = Result<T::Item, RpcError>;

    #[allow(clippy::cognitive_complexity)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let Some(state) = self.state.take() else {
                return Poll::Ready(None);
            };

            self.state = match state {
                /////////////////////
                // Subscribing ... //
                /////////////////////
                SubscriptionState::Subscribing(mut fut) => {
                    match fut.poll_unpin(cx) {
                        // Subscription succeeded
                        Poll::Ready(Ok(sub)) => {
                            let attempts = self.consecutive_subscription_errors;
                            if let Some(timestamp) = self.last_subscription_timestamp.take() {
                                let elapsed = timestamp.elapsed();
                                tracing::info!("succesfully resubscribed after {elapsed:?}, attemps: {attempts}", elapsed = elapsed);
                            } else if attempts > 0 {
                                tracing::info!("succesfully subscribed after {attempts} attempt");
                            }
                            // Reset error counter and update last subscription timestamp.
                            self.total_subscriptions += 1;
                            self.consecutive_subscription_errors = 0;
                            self.last_subscription_timestamp = Some(Instant::now());
                            Some(SubscriptionState::Subscribed(sub))
                        },

                        // Subscription failed
                        Poll::Ready(Err(err)) => {
                            if matches!(err, RpcError::HttpNotImplemented) {
                                // Http doesn't support subscriptions, return error and close the
                                // stream
                                return Poll::Ready(Some(Err(RpcError::HttpNotImplemented)));
                            }
                            // increment error counter and retry after delay.
                            let attempts = self.consecutive_subscription_errors + 1;
                            let msg = err.truncate();
                            tracing::error!("subscription attempt {attempts} failed: {msg}");
                            self.consecutive_subscription_errors = attempts;

                            // Schedule next subscription attempt.
                            Some(SubscriptionState::ResubscribeAfterDelay(Delay::new(
                                self.retry_interval,
                            )))
                        },

                        // Subscription is pending
                        Poll::Pending => {
                            self.state = Some(SubscriptionState::Subscribing(fut));
                            return Poll::Pending;
                        },
                    }
                },

                ////////////////////////////
                // Subscription is active //
                ////////////////////////////
                SubscriptionState::Subscribed(mut sub) => match sub.poll_next_unpin(cx) {
                    // Got a new item
                    Poll::Ready(Some(Ok(item))) => {
                        self.state = Some(SubscriptionState::Subscribing(
                            self.retry_subscription.retry_subscribe(),
                        ));
                        return Poll::Ready(Some(Ok(item)));
                    },

                    // Got an error
                    Poll::Ready(Some(Err(err))) => {
                        match err {
                            // Subscription terminated, resubscribe.
                            RpcError::RestartNeeded(msg) => {
                                tracing::error!("subscription terminated: {}", msg.truncate());
                                Some(SubscriptionState::Subscribing(
                                    self.retry_subscription.retry_subscribe(),
                                ))
                            },
                            // Http doesn't support subscriptions, return error and close the stream
                            RpcError::HttpNotImplemented => {
                                return Poll::Ready(Some(Err(RpcError::HttpNotImplemented)));
                            },
                            // Return error
                            err => {
                                self.state = Some(SubscriptionState::Subscribing(
                                    self.retry_subscription.retry_subscribe(),
                                ));
                                return Poll::Ready(Some(Err(err)));
                            },
                        }
                    },

                    // Stream was close, resubscribe.
                    Poll::Ready(None) => Some(SubscriptionState::Subscribing(
                        self.retry_subscription.retry_subscribe(),
                    )),

                    // Stream is pending
                    Poll::Pending => {
                        self.state = Some(SubscriptionState::Subscribed(sub));
                        return Poll::Pending;
                    },
                },

                /////////////
                // Waiting //
                /////////////
                SubscriptionState::ResubscribeAfterDelay(mut delay) => match delay.poll_unpin(cx) {
                    Poll::Ready(()) => {
                        // Timeout elapsed, retry subscription.
                        Some(SubscriptionState::Subscribing(
                            self.retry_subscription.retry_subscribe(),
                        ))
                    },
                    Poll::Pending => {
                        self.state = Some(SubscriptionState::ResubscribeAfterDelay(delay));
                        return Poll::Pending;
                    },
                },
            };
        }
    }
}

pub enum SubscriptionStatus<T: RetrySubscription> {
    /// Subscribed
    Subscribed(AutoSubscribe<T>),
    /// Unsubscribing
    Unsubscribing(BoxFuture<'static, Result<(), RpcError>>),
}

pub struct Config {
    /// polling interval for check new blocks. Only used when the new_heads
    /// stream is close or not supported.
    pub polling_interval: Duration,

    /// Maximum number of consecutive errors before the stream is closed.
    pub stream_error_threshold: u32,

    /// Cached unfinalized blocks
    pub unfinalized_cache_capacity: usize,
}

/// A stream which emits new blocks and logs matching a filter.
pub struct BlockSubscription<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Clone
        + Unpin
        + Send
        + Sync
        + 'static,
{
    /// Stream of new block headers.
    config: Config,

    /// Timestamp when the last block was received.
    last_block_timestamp: Option<Instant>,

    /// Subscription to new block headers.
    /// Obs: This only emits pending blocks headers, not latest or finalized ones.
    new_heads_sub: Option<SubscriptionStatus<RetryNewHeadsSubscription<RPC>>>,

    /// Subscription to new finalized blocks, the stream guarantees that new finalized blocks are
    /// monotonically increasing.
    finalized_blocks_stream: FinalizedBlockStream<RPC>,

    /// Count of consecutive errors.
    consecutive_errors: u32,
}

impl<RPC> BlockSubscription<RPC>
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
        let new_heads_sub = RetryNewHeadsSubscription::new(backend.clone());
        Self {
            config,
            last_block_timestamp: None,
            new_heads_sub: Some(SubscriptionStatus::Subscribed(AutoSubscribe::new(
                new_heads_sub,
                Duration::from_secs(5),
            ))),
            finalized_blocks_stream: FinalizedBlockStream::new(backend),
            consecutive_errors: 0,
        }
    }
}

impl<RPC> Stream for BlockSubscription<RPC>
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

        // Fetch latest finalized block
        match self.finalized_blocks_stream.poll_next_unpin(cx) {
            Poll::Ready(Some(finalized_block)) => {
                return Poll::Ready(Some(NewBlockEvent::Finalized(finalized_block)));
            },
            Poll::Ready(None) => {
                tracing::error!("[report this bug] finalized block stream should never be closed");
            },
            Poll::Pending => {},
        }

        // Poll new heads subscription
        self.new_heads_sub = match self.new_heads_sub.take() {
            Some(SubscriptionStatus::Subscribed(mut new_heads_sub)) => {
                match new_heads_sub.poll_next_unpin(cx) {
                    // New block header
                    Poll::Ready(Some(Ok(block))) => {
                        // Reset error counter.
                        self.consecutive_errors = 0;

                        // Update last block timestamp.
                        self.last_block_timestamp = Some(Instant::now());

                        // Calculate header hash and return it.
                        let block = block.seal_slow::<DefaultCrypto>();
                        self.new_heads_sub = Some(SubscriptionStatus::Subscribed(new_heads_sub));
                        return Poll::Ready(Some(NewBlockEvent::Pending(block)));
                    },

                    // Subscription returned an error
                    Poll::Ready(Some(Err(err))) => {
                        self.consecutive_errors += 1;
                        match err {
                            RpcError::RestartNeeded(_) | RpcError::HttpNotImplemented => {
                                // Subscription was terminated... no need to unsubscribe.
                                None
                            },
                            err => {
                                if self.consecutive_errors >= self.config.stream_error_threshold {
                                    // Consecutive error threshold reached, unsubscribe and close
                                    // the stream.
                                    tracing::error!(
                                        "new heads stream returned too many consecutive errors: {}",
                                        err.truncate()
                                    );
                                    Some(SubscriptionStatus::Unsubscribing(
                                        new_heads_sub.unsubscribe().boxed(),
                                    ))
                                } else {
                                    tracing::error!("new heads stream error: {}", err.truncate());
                                    Some(SubscriptionStatus::Subscribed(new_heads_sub))
                                }
                            },
                        }
                    },
                    Poll::Ready(None) => {
                        // Stream ended
                        tracing::warn!(
                            "new heads subscription terminated, will poll new blocks every {:?}",
                            self.config.polling_interval
                        );
                        None
                    },
                    Poll::Pending => Some(SubscriptionStatus::Subscribed(new_heads_sub)),
                }
            },
            Some(SubscriptionStatus::Unsubscribing(mut fut)) => match fut.poll_unpin(cx) {
                Poll::Ready(Ok(())) => None,
                Poll::Ready(Err(err)) => {
                    tracing::error!(
                        "failed to unsubscribe from new heads stream: {}",
                        err.truncate()
                    );
                    None
                },
                Poll::Pending => Some(SubscriptionStatus::Unsubscribing(fut)),
            },
            None => None,
        };

        Poll::Pending
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
