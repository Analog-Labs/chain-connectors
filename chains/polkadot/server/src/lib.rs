use anyhow::{Context, Result};
use rosetta_server::types::BlockIdentifier;
use rosetta_server::{BlockchainClient, BlockchainConfig};
use subxt::{OnlineClient, PolkadotConfig};

pub struct PolkadotClient {
    config: BlockchainConfig,
    client: OnlineClient<PolkadotConfig>,
    node_version: String,
    genesis_block: BlockIdentifier,
}

#[async_trait::async_trait]
impl BlockchainClient for PolkadotClient {
    async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_polkadot::config(network)?;
        let client = OnlineClient::<PolkadotConfig>::from_url(format!("ws://{}", addr)).await?;
        let genesis = client.rpc().genesis_hash().await?;
        let node_version = client.rpc().system_version().await?;
        let genesis_block = BlockIdentifier {
            index: 0,
            hash: hex::encode(genesis.as_ref()),
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
        let block = self.client.rpc().block(None).await?.context("no current block")?;
        let index = block.block.header.number as _;
        let hash = block.block.header.hash();
        Ok(BlockIdentifier {
            index,
            hash: hex::encode(hash.as_ref()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_list() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_server::tests::network_list(config).await
    }

    #[tokio::test]
    async fn test_network_options() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_server::tests::network_options::<PolkadotClient>(config).await
    }

    #[tokio::test]
    async fn test_network_status() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_server::tests::network_status::<PolkadotClient>(config).await
    }
}
