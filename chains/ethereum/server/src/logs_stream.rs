use futures_util::{future::BoxFuture, Stream, StreamExt};
use rosetta_ethereum_backend::{
    ext::types::{crypto::DefaultCrypto, rpc::RpcBlock, AtBlock, SealedBlock, H256},
    jsonrpsee::core::client::{Error as RpcError, Subscription},
    EthereumPubSub, EthereumRpc,
};
use rosetta_utils::{futures::FutureFactory, jsonrpsee::{AutoSubscribe, CircuitBreaker, PollingInterval}};
use std::{
    mem,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

/// Default polling interval for checking for new finalized blocks.
const DEFAULT_POLLING_INTERVAL: Duration = Duration::from_secs(2);

/// Maximum number of errors before terminate the stream.
const MAX_ERRORS: u32 = 10;

type PartialBlock = SealedBlock<H256, H256>;

struct PollLatestBlock<RPC>(RPC);

impl<RPC> FutureFactory for PollLatestBlock<RPC>
where
    RPC: EthereumRpc<Error = RpcError> + Send + Sync + 'static,
{
    type Output = Result<Option<PartialBlock>, RpcError>;
    type Future<'a> = BoxFuture<'a, Self::Output>;
    fn new_future(&mut self) -> Self::Future<'_> {
        self.0.block(AtBlock::Latest)
    }
}

struct LogsSubscriber<RPC> {
    backend: RPC,
}

impl<RPC> FutureFactory for LogsSubscriber<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Send
        + Sync
        + 'static,
{
    type Output = Result<Subscription<RpcBlock<H256>>, RpcError>;
    type Future<'a> = BoxFuture<'a, Self::Output>;

    fn new_future(&mut self) -> Self::Future<'_> {
        EthereumPubSub::logs(&self.backend)
    }
}

impl<RPC> LogsSubscriber<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Send
        + Sync
        + 'static,
{
    #[must_use]
    pub const fn new(backend: RPC) -> Self {
        Self { backend }
    }

    pub fn into_inner(self) -> RPC {
        self.backend
    }
}

// Subscription to new block headers. Can be either a websocket subscription or a polling interval.
enum State<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Send
        + Sync
        + 'static,
{
    Subscription(AutoSubscribe<RpcBlock<H256>, NewHeadsSubscriber<RPC>>),
    Polling(CircuitBreaker<PollingInterval<PollLatestBlock<RPC>>, ()>),
    Terminated,
    Poisoned,
}

impl<RPC> State<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Send
        + Sync
        + 'static,
{
    #[must_use]
    pub const fn new(backend: RPC) -> Self {
        let subscriber = NewHeadsSubscriber::new(backend);
        Self::Subscription(AutoSubscribe::new(DEFAULT_POLLING_INTERVAL, subscriber))
    }
}

/// A stream which emits new blocks and logs matching a filter.
#[pin_project::pin_project]
pub struct LogStream<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Unpin
        + Send
        + Sync
        + 'static,
{
    /// Subscription or Polling to new block headers.
    state: State<RPC>,

    /// Timestamp when the last block was received.
    last_block_timestamp: Option<Instant>,

    /// Error count, used to determine if the stream should be terminated.
    error_count: u32,
}

impl<RPC> LogStream<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + EthereumRpc
        + Unpin
        + Send
        + Sync
        + 'static,
{
    #[must_use]
    pub const fn new(backend: RPC) -> Self {
        Self { state: State::new(backend), last_block_timestamp: None, error_count: 0 }
    }
}

impl<RPC> Stream for LogStream<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + EthereumRpc
        + Unpin
        + Send
        + Sync
        + 'static,
{
    type Item = PartialBlock;

    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        loop {
            match mem::replace(this.state, State::Poisoned) {
                State::Subscription(mut subscription) => {
                    match subscription.poll_next_unpin(cx) {
                        Poll::Ready(Some(result)) => match result {
                            Ok(block) => {
                                *this.last_block_timestamp = Some(Instant::now());
                                *this.error_count = 0;
                                let block = if let Some(block_hash) = block.hash {
                                    block.seal(block_hash)
                                } else {
                                    block.seal_slow::<DefaultCrypto>()
                                };
                                *this.state = State::Subscription(subscription);
                                return Poll::Ready(Some(block));
                            },
                            Err(err) => {
                                tracing::warn!(
                                    "new heads subscription returned an error: {:?}",
                                    err
                                );
                                *this.error_count += 1;
                                if *this.error_count >= MAX_ERRORS {
                                    // Unsubscribe if the error count exceeds the maximum.
                                    subscription.unsubscribe();
                                }
                                *this.state = State::Subscription(subscription);
                            },
                        },
                        Poll::Ready(None) => {
                            // Subscription terminated, switch to polling.
                            if let Some(backend) =
                                subscription.into_subscriber().map(NewHeadsSubscriber::into_inner)
                            {
                                *this.error_count = this.error_count.saturating_sub(2);
                                *this.state = State::Polling(CircuitBreaker::new(
                                    PollingInterval::new(
                                        PollLatestBlock(backend),
                                        DEFAULT_POLLING_INTERVAL,
                                    ),
                                    MAX_ERRORS,
                                    (),
                                ));
                            } else {
                                // This should never happen, once if the `AutoSubscribe` returns
                                // None, we must be able to retrieve
                                // the backend.
                                tracing::error!("[report this bug] the subscription returned None and the backend is not available");
                                *this.state = State::Terminated;
                                return Poll::Ready(None);
                            }
                        },
                        Poll::Pending => {
                            *this.state = State::Subscription(subscription);
                            return Poll::Pending;
                        },
                    }
                },

                State::Polling(mut polling) => match polling.poll_next_unpin(cx) {
                    Poll::Ready(Some(Ok(Some(block)))) => {
                        *this.state = State::Polling(polling);
                        *this.error_count = 0;
                        *this.last_block_timestamp = Some(Instant::now());
                        return Poll::Ready(Some(block));
                    },
                    Poll::Ready(Some(Ok(None))) => {
                        tracing::error!(
                            "[report this bug] the client returned null for the latest block"
                        );
                        *this.state = State::Terminated;
                        return Poll::Ready(None);
                    },
                    Poll::Ready(Some(Err(err))) => {
                        *this.state = State::Polling(polling);
                        *this.error_count += 1;
                        tracing::error!(
                            "polling interval returned an error ({}): {err:?}",
                            *this.error_count,
                        );
                    },
                    Poll::Ready(None) => {
                        *this.state = State::Terminated;
                        return Poll::Ready(None);
                    },
                    Poll::Pending => {
                        *this.state = State::Polling(polling);
                        return Poll::Pending;
                    },
                },
                State::Terminated => {
                    panic!("stream polled after completion");
                },
                State::Poisoned => {
                    panic!("stream poisoned");
                },
            }

            // Terminate the stream if the error count exceeds the maximum.
            if *this.error_count >= MAX_ERRORS {
                *this.state = State::Terminated;
                return Poll::Ready(None);
            }
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
    use rosetta_ethereum_backend::jsonrpsee::Adapter;

    pub async fn client_from_config(
        config: BlockchainConfig,
    ) -> anyhow::Result<MaybeWsEthereumClient> {
        let url = config.node_uri.to_string();
        MaybeWsEthereumClient::from_config(config, url.as_str(), None).await
    }

    struct TestSubscriber<RPC>(RPC);

    impl<RPC> FutureFactory for TestSubscriber<RPC>
    where
        RPC: for<'s> EthereumPubSub<
                Error = RpcError,
                NewHeadsStream<'s> = Subscription<RpcBlock<H256>>,
            > + Send
            + Sync
            + 'static,
    {
        type Output = Result<Subscription<RpcBlock<H256>>, RpcError>;
        type Future<'a> = BoxFuture<'a, Self::Output>;

        fn new_future(&mut self) -> Self::Future<'_> {
            EthereumPubSub::new_heads(&self.0)
        }
    }

    #[tokio::test]
    async fn new_heads_stream_works() -> anyhow::Result<()> {
        let config = rosetta_config_ethereum::config("dev").unwrap();
        let env = Env::new("new-heads-stream", config.clone(), client_from_config).await.unwrap();

        run_test(env, |env| async move {
            let client = match env.node().as_ref() {
                MaybeWsEthereumClient::Http(_) => panic!("the connections must be ws"),
                MaybeWsEthereumClient::Ws(client) => client.backend.clone(),
            };
            let client = Adapter(client.into_inner());
            let mut sub = BlockSubscription::new(client);
            let mut latest_block: Option<SealedBlock<H256>> = None;
            for i in 0..10 {
                let Some(new_block) = sub.next().await else {
                    panic!("stream ended");
                };
                if i == 5 {
                    if let State::Subscription(sub) = &mut sub.state {
                        sub.unsubscribe();
                    }
                }
                if let Some(latest_block) = latest_block.as_ref() {
                    let last_number = latest_block.header().number();
                    let new_number = new_block.header().number();
                    assert!(new_number >= last_number);
                    if new_number == (last_number + 1) {
                        assert_eq!(
                            latest_block.header().hash(),
                            new_block.header().header().parent_hash
                        );
                    }
                }
                latest_block = Some(new_block);
            }
        })
        .await;
        Ok(())
    }
}
