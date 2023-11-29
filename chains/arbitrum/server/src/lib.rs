use anyhow::Result;
use rosetta_config_ethereum::{EthereumMetadata, EthereumMetadataParams};
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
        self.balance(address, block).await
    }

    async fn coins(&self, address: &Address, block: &BlockIdentifier) -> Result<Vec<Coin>> {
        self.client.coins(address, block).await
    }

    async fn faucet(&self, address: &Address, value: u128) -> Result<Vec<u8>> {
        // // convert address
        // let dest = {
        //     let address: H160 = address.address().parse()?;
        //     let mut data = [0u8; 24];
        //     data[0..4].copy_from_slice(b"evm:");
        //     data[4..24].copy_from_slice(&address[..]);
        //     let hash = sp_core::hashing::blake2_256(&data);
        //     AccountId32::from(Into::<[u8; 32]>::into(hash))
        // };

        // // Build the transfer transaction
        // let balance_transfer_tx = astar_metadata::tx().balances().transfer(dest.into(), value);
        // let alice = sp_keyring::AccountKeyring::Alice.pair();
        // let signer = PairSigner::<PolkadotConfig, _>::new(alice);

        // let hash = self
        //     .ws_client
        //     .tx()
        //     .sign_and_submit_then_watch_default(&balance_transfer_tx, &signer)
        //     .await?
        //     .wait_for_finalized_success()
        //     .await?
        //     .extrinsic_hash();
        // Ok(hash.0.to_vec())
        self.client.faucet(address, value).await
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

    async fn call(&self, req: &CallRequest) -> Result<Value> {
        self.client.call(req).await
    }

    async fn listen<'a>(&'a self) -> Result<Option<Self::EventStream<'a>>> {
        self.client.listen().await
    }
}
