use anyhow::Result;
use rosetta_config_ethereum::{
    EthereumMetadata, EthereumMetadataParams, Query as EthQuery, QueryResult as EthQueryResult,
};
use rosetta_core::{
    crypto::{address::Address, PublicKey},
    types::{
        Block, BlockIdentifier, CallRequest, Coin, PartialBlockIdentifier, Transaction,
        TransactionIdentifier,
    },
    BlockchainClient, BlockchainConfig,
};
use rosetta_server_ethereum::MaybeWsEthereumClient;

use rosetta_server::ws::default_client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct ArbitrumMetadataParams(pub EthereumMetadataParams);

#[derive(Debug, Deserialize, Serialize)]
pub struct ArbitrumMetadata(pub EthereumMetadata);

pub struct ArbitrumClient {
    client: MaybeWsEthereumClient,
}

impl ArbitrumClient {
    /// Creates a new Arbitrum client, loading the config from `network` and connects to `addr`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn new(network: &str, url: &str) -> Result<Self> {
        let config = rosetta_config_arbitrum::config(network)?;
        Self::from_config(config, url).await
    }

    /// Creates a new Arbitrum client using the provided `config` and connects to `addr`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn from_config(config: BlockchainConfig, url: &str) -> Result<Self> {
        let ws_client = default_client(url, None).await?;
        let ethereum_client = MaybeWsEthereumClient::from_jsonrpsee(config, ws_client).await?;
        Ok(Self { client: ethereum_client })
    }
}

#[async_trait::async_trait]
impl BlockchainClient for ArbitrumClient {
    type MetadataParams = ArbitrumMetadataParams;
    type Metadata = ArbitrumMetadata;
    type EventStream<'a> = <MaybeWsEthereumClient as BlockchainClient>::EventStream<'a>;
    type Call = EthQuery;
    type CallResult = EthQueryResult;

    fn config(&self) -> &BlockchainConfig {
        self.client.config()
    }

    fn genesis_block(&self) -> &BlockIdentifier {
        self.client.genesis_block()
    }

    async fn node_version(&self) -> Result<String> {
        self.client.node_version().await
    }

    async fn current_block(&self) -> Result<BlockIdentifier> {
        self.client.current_block().await
    }

    async fn finalized_block(&self) -> Result<BlockIdentifier> {
        self.client.finalized_block().await
    }

    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128> {
        self.client.balance(address, block).await
    }

    async fn coins(&self, address: &Address, block: &BlockIdentifier) -> Result<Vec<Coin>> {
        self.client.coins(address, block).await
    }

    async fn faucet(
        &self,
        address: &Address,
        value: u128,
        private_key: Option<&str>,
    ) -> Result<Vec<u8>> {
        self.client.faucet(address, value, private_key).await
    }

    async fn metadata(
        &self,
        public_key: &PublicKey,
        options: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        Ok(ArbitrumMetadata(self.client.metadata(public_key, &options.0).await?))
    }

    async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        self.client.submit(transaction).await
    }

    async fn block(&self, block_identifier: &PartialBlockIdentifier) -> Result<Block> {
        self.client.block(block_identifier).await
    }

    async fn block_transaction(
        &self,
        block_identifier: &BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        self.client.block_transaction(block_identifier, tx).await
    }

    async fn call(&self, req: &EthQuery) -> Result<EthQueryResult> {
        self.client.call(req).await
    }

    async fn listen<'a>(&'a self) -> Result<Option<Self::EventStream<'a>>> {
        self.client.listen().await
    }
}
