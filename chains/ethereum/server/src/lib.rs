use anyhow::{Context, Result};
use ethers::prelude::*;
use rosetta_server::types::BlockIdentifier;
use rosetta_server::{BlockchainClient, BlockchainConfig};

pub struct EthereumClient {
    config: BlockchainConfig,
    client: Provider<Http>,
    node_version: String,
    genesis_block: BlockIdentifier,
}

#[async_trait::async_trait]
impl BlockchainClient for EthereumClient {
    async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_ethereum::config(network)?;
        let client = Provider::<Http>::try_from(format!("http://{}", addr))?;
        let node_version = client.client_version().await?;
        let genesis = client.get_block(0).await?.unwrap();
        let genesis_block = BlockIdentifier {
            index: 0,
            hash: hex::encode(genesis.hash.as_ref().unwrap()),
        };
        Ok(Self {
            config,
            client,
            node_version,
            genesis_block,
        })
    }

    fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    fn node_version(&self) -> &str {
        &self.node_version
    }

    fn genesis_block(&self) -> &BlockIdentifier {
        &self.genesis_block
    }

    async fn current_block(&self) -> Result<BlockIdentifier> {
        let index = self.client.get_block_number().await?.as_u64();
        let block = self
            .client
            .get_block(index)
            .await?
            .context("missing block")?;
        Ok(BlockIdentifier {
            index,
            hash: hex::encode(block.hash.as_ref().unwrap()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_list() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::network_list(config).await
    }

    #[tokio::test]
    async fn test_network_options() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::network_options::<EthereumClient>(config).await
    }

    #[tokio::test]
    async fn test_network_status() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::network_status::<EthereumClient>(config).await
    }
}
