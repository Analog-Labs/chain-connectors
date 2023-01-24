use anyhow::Result;
use bitcoincore_rpc_async::{Auth, Client, RpcApi};
use rosetta_server::types::BlockIdentifier;
use rosetta_server::{BlockchainClient, BlockchainConfig};

pub struct BitcoinClient {
    config: BlockchainConfig,
    client: Client,
    node_version: String,
    genesis_block: BlockIdentifier,
}

#[async_trait::async_trait]
impl BlockchainClient for BitcoinClient {
    async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_bitcoin::config(network)?;
        let client = Client::new(
            addr.to_string(),
            Auth::UserPass("rosetta".into(), "rosetta".into()),
        )
        .await?;
        let info = client.get_network_info().await?;
        let genesis = client.get_block_hash(0).await?;
        let major = info.version / 10000;
        let rest = info.version % 10000;
        let minor = rest / 100;
        let patch = rest % 100;
        let node_version = format!("{}.{}.{}", major, minor, patch);
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
        let index = self.client.get_block_count().await?;
        let hash = self.client.get_block_hash(index).await?;
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
        let config = rosetta_config_bitcoin::config("regtest")?;
        rosetta_server::tests::network_list(config).await
    }

    #[tokio::test]
    async fn test_network_options() -> Result<()> {
        let config = rosetta_config_bitcoin::config("regtest")?;
        rosetta_server::tests::network_options::<BitcoinClient>(config).await
    }

    #[tokio::test]
    async fn test_network_status() -> Result<()> {
        let config = rosetta_config_bitcoin::config("regtest")?;
        rosetta_server::tests::network_status::<BitcoinClient>(config).await
    }
}
