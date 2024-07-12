#![allow(dead_code)]
use crate::{
    finalized_block_stream::FinalizedBlockStream, new_heads::NewHeadsStream, state::State,
};
use rosetta_ethereum_backend::{
    ext::types::{rpc::RpcBlock, H256},
    jsonrpsee::core::{client::Subscription, ClientError as RpcError},
    EthereumPubSub,
};

pub struct BlockStream<RPC>
where
    RPC: for<'s> EthereumPubSub<Error = RpcError, NewHeadsStream<'s> = Subscription<RpcBlock<H256>>>
        + Unpin
        + Send
        + Sync
        + 'static,
    RPC::SubscriptionError: Send + Sync,
{
    finalized: FinalizedBlockStream<RPC>,
    new_heads: NewHeadsStream<RPC>,
    state: State,
}
