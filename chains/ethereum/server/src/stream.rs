use futures_util::{Stream, future::BoxFuture};
use std::{task::{Context, Poll}, pin::Pin};
use rosetta_ethereum_backend::{
    ext::types::{rpc::{RpcBlock, RpcTransaction}, Header},
    EthereumRpc,
    jsonrpsee::core::client::Subscription,
};

type BlockFull = RpcBlock<RpcTransaction>;

pub struct BlockStream<B> {
    backend: B,
    new_heads: Option<Subscription<BlockFull>>,
}

impl <B: EthereumRpc + Unpin> Stream for BlockStream<B> {
    type Item = ();
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let future = self.backend.block_full::<RpcTransaction>(at);

        Poll::Pending
    }
}

