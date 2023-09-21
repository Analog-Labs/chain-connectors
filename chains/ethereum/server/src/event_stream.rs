use ethers::prelude::*;
use ethers::providers::PubsubClient;
use rosetta_server::stream::Stream;
use rosetta_server::types::BlockIdentifier;
use rosetta_server::{BlockOrIdentifier, ClientEvent};
use std::pin::Pin;
use std::task::Poll;

// Maximum number of failures in sequence before closing the stream
const FAILURE_THRESHOLD: u32 = 10;

pub struct EthereumEventStream<'a, P: PubsubClient> {
    pub new_head: SubscriptionStream<'a, P, Block<H256>>,
    pub failures: u32,
    span: tracing::Span,
}

impl<'a, P> EthereumEventStream<'a, P>
where
    P: PubsubClient,
{
    pub fn new(subscription: SubscriptionStream<'a, P, Block<H256>>) -> Self {
        let id = format!("{:?}", subscription.id);
        Self {
            new_head: subscription,
            failures: 0,
            span: tracing::warn_span!("eth_subscription", id = id),
        }
    }
}

impl<P> Stream for EthereumEventStream<'_, P>
where
    P: PubsubClient,
{
    type Item = ClientEvent;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            if self.failures >= FAILURE_THRESHOLD {
                let _enter = self.span.enter();
                tracing::error!("More than 10 failures in sequence");
                return Poll::Ready(Some(ClientEvent::Close(
                    "More than 10 failures in sequence".into(),
                )));
            }

            match self.new_head.poll_next_unpin(cx) {
                Poll::Ready(Some(block)) => {
                    let Some(number) = block.number else {
                        self.failures += 1;
                        let _enter = &self.span.enter();
                        tracing::error!("block number is missing");
                        continue;
                    };

                    let Some(hash) = block.hash else {
                        self.failures += 1;
                        let _enter = &self.span.enter();
                        tracing::error!("block hash is missing");
                        continue;
                    };

                    self.failures = 0;
                    let ident = BlockIdentifier::new(number.as_u64(), hex::encode(hash));
                    return Poll::Ready(Some(ClientEvent::NewHead(BlockOrIdentifier::Identifier(
                        ident,
                    ))));
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };
        }
    }
}
