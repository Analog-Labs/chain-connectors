use crate::BlockchainClient;
use anyhow::Result;
use std::ops::Deref;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionRef {
    pub block_index: u64,
    pub transaction_index: u32,
}

#[derive(Clone)]
pub struct Indexer<C: BlockchainClient> {
    client: C,
}

impl<C: BlockchainClient> Deref for Indexer<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<C: BlockchainClient> Indexer<C> {
    pub fn new(client: C) -> Result<Self> {
        Ok(Self { client })
    }
}
