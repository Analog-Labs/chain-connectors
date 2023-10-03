use anyhow::{Context, Result};
use bitcoincore_rpc_async::bitcoin::BlockHash;
use bitcoincore_rpc_async::{Auth, Client, RpcApi};
use rosetta_core::{
    crypto::{address::Address, PublicKey},
    types::{
        Block, BlockIdentifier, CallRequest, Coin, PartialBlockIdentifier, Transaction,
        TransactionIdentifier,
    },
    BlockchainClient, BlockchainConfig,
};
use serde_json::Value;
use std::str::FromStr;

pub type BitcoinMetadataParams = ();
pub type BitcoinMetadata = ();

pub struct BitcoinClient {
    config: BlockchainConfig,
    client: Client,
    genesis_block: BlockIdentifier,
}

impl BitcoinClient {
    pub async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_bitcoin::config(network)?;
        Self::from_config(config, addr).await
    }

    pub async fn from_config(config: BlockchainConfig, addr: &str) -> Result<Self> {
        let client = Client::new(
            addr.to_string(),
            Auth::UserPass("rosetta".into(), "rosetta".into()),
        )
        .await?;
        let genesis = client.get_block_hash(0).await?;
        let genesis_block = BlockIdentifier {
            index: 0,
            hash: genesis.to_string(),
        };

        Ok(Self {
            config,
            client,
            genesis_block,
        })
    }
}

/// Bitcoin community has adopted 6 blocks as a standard confirmation period.
/// That is, once a transaction is included in a block in the blockchain which is followed up by at least 6 additional blocks
/// the transaction is called “confirmed.” While this was chosen somewhat arbitrarily, it is a reasonably safe value in practice
/// as the only time this would have left users vulnerable to double-spending was the atypical March 2013 fork.
const CONFIRMATION_PERIOD: u64 = 6;

#[async_trait::async_trait]
impl BlockchainClient for BitcoinClient {
    type MetadataParams = BitcoinMetadataParams;
    type Metadata = BitcoinMetadata;
    type EventStream<'a> = rosetta_core::EmptyEventStream;

    fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    fn genesis_block(&self) -> &BlockIdentifier {
        &self.genesis_block
    }

    async fn node_version(&self) -> Result<String> {
        let info = self.client.get_network_info().await?;
        let major = info.version / 10000;
        let rest = info.version % 10000;
        let minor = rest / 100;
        let patch = rest % 100;
        Ok(format!("{major}.{minor}.{patch}"))
    }

    async fn current_block(&self) -> Result<BlockIdentifier> {
        let hash = self.client.get_best_block_hash().await?;
        let info = self.client.get_block_info(&hash).await?;
        Ok(BlockIdentifier {
            index: info.height as u64,
            hash: hash.to_string(),
        })
    }

    async fn finalized_block(&self) -> Result<BlockIdentifier> {
        let index = self
            .client
            .get_block_count()
            .await?
            .saturating_sub(CONFIRMATION_PERIOD);
        let hash = self.client.get_block_hash(index).await?;
        Ok(BlockIdentifier {
            index,
            hash: hash.to_string(),
        })
    }

    async fn balance(&self, _address: &Address, _block: &BlockIdentifier) -> Result<u128> {
        todo!()
    }

    async fn coins(&self, _address: &Address, _block: &BlockIdentifier) -> Result<Vec<Coin>> {
        todo!()
    }

    async fn faucet(&self, _address: &Address, _value: u128) -> Result<Vec<u8>> {
        todo!()
    }

    async fn metadata(
        &self,
        _public_key: &PublicKey,
        _options: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        Ok(())
    }

    async fn submit(&self, _transaction: &[u8]) -> Result<Vec<u8>> {
        todo!()
    }

    async fn block(&self, block: &PartialBlockIdentifier) -> Result<Block> {
        let block = match (block.hash.as_ref(), block.index) {
            (Some(block_hash), _) => {
                let hash = BlockHash::from_str(block_hash).context("Invalid block hash")?;
                self.client.get_block(&hash).await?
            }
            (None, Some(height)) => {
                let block_bash = self
                    .client
                    .get_block_hash(height)
                    .await
                    .context("cannot find by index")?;
                self.client.get_block(&block_bash).await?
            }
            (None, None) => anyhow::bail!("the block hash or index must be specified"),
        };

        let block_height = match block.bip34_block_height().ok() {
            Some(height) => height,
            None => {
                let info = self
                    .client
                    .get_block_info(&block.block_hash())
                    .await
                    .context("Cannot find block height")?;
                info.height as u64
            }
        };

        let transactions = block
            .txdata
            .iter()
            .map(|tx| Transaction {
                transaction_identifier: TransactionIdentifier::new(tx.txid().as_hash().to_string()),
                operations: vec![],
                related_transactions: None,
                metadata: serde_json::to_value(tx.clone()).ok(),
            })
            .collect::<Vec<_>>();

        Ok(Block {
            block_identifier: BlockIdentifier {
                index: block_height,
                hash: block.block_hash().to_string(),
            },
            parent_block_identifier: BlockIdentifier {
                index: block_height.saturating_sub(1),
                hash: block.header.prev_blockhash.to_string(),
            },
            timestamp: i64::from(block.header.time) * 1000,
            transactions,
            metadata: None,
        })
    }

    async fn block_transaction(
        &self,
        _block: &BlockIdentifier,
        _tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        anyhow::bail!("not implemented")
    }

    async fn call(&self, _req: &CallRequest) -> Result<Value> {
        anyhow::bail!("not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub async fn client_from_config(config: BlockchainConfig) -> Result<BitcoinClient> {
        let network = config.network.to_string();
        let url = config.node_uri.to_string();
        BitcoinClient::new(network.as_str(), url.as_str()).await
    }

    #[tokio::test]
    async fn test_network_status() -> Result<()> {
        let config = rosetta_config_bitcoin::config("regtest")?;
        rosetta_docker::tests::network_status::<BitcoinClient, _, _>(client_from_config, config)
            .await
    }

    #[tokio::test]
    #[ignore]
    async fn test_account() -> Result<()> {
        let config = rosetta_config_bitcoin::config("regtest")?;
        rosetta_docker::tests::account::<BitcoinClient, _, _>(client_from_config, config).await
    }

    #[tokio::test]
    #[ignore]
    async fn test_construction() -> Result<()> {
        let config = rosetta_config_bitcoin::config("regtest")?;
        rosetta_docker::tests::construction::<BitcoinClient, _, _>(client_from_config, config).await
    }
}
