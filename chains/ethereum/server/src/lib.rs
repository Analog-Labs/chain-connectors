use anyhow::{Context, Result};
use ethers::prelude::*;
use rosetta_server::crypto::address::Address;
use rosetta_server::crypto::PublicKey;
use rosetta_server::types::{BlockIdentifier, Coin};
use rosetta_server::{BlockchainClient, BlockchainConfig};

pub struct EthereumClient {
    config: BlockchainConfig,
    client: Provider<Http>,
    genesis_block: BlockIdentifier,
}

#[async_trait::async_trait]
impl BlockchainClient for EthereumClient {
    type MetadataParams = ();
    type Metadata = ();

    async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_ethereum::config(network)?;
        let client = Provider::<Http>::try_from(format!("http://{}", addr))?;
        let genesis = client.get_block(0).await?.unwrap();
        let genesis_block = BlockIdentifier {
            index: 0,
            hash: hex::encode(genesis.hash.as_ref().unwrap()),
        };
        Ok(Self {
            config,
            client,
            genesis_block,
        })
    }

    fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    fn genesis_block(&self) -> &BlockIdentifier {
        &self.genesis_block
    }

    async fn node_version(&self) -> Result<String> {
        Ok(self.client.client_version().await?)
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

    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128> {
        let block = hex::decode(&block.hash)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid block hash"))?;
        Ok(self
            .client
            .get_balance(address.address(), Some(BlockId::Hash(H256(block))))
            .await?
            .as_u128())
    }

    async fn coins(&self, _address: &Address, _block: &BlockIdentifier) -> Result<Vec<Coin>> {
        anyhow::bail!("not a utxo chain");
    }

    async fn faucet(&self, _address: &Address, _param: u128) -> Result<Vec<u8>> {
        // from: eth.coinbase to: address value: param
        /*let tx = todo!();
        Ok(self
            .client
            .send_transaction(tx, None)
            .await?
            .await?
            .unwrap()
            .block_hash
            .unwrap()
            .0
            .to_vec())*/
        todo!()
    }

    async fn metadata(
        &self,
        _public_key: &PublicKey,
        _options: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        todo!()
    }

    async fn submit(&self, _transaction: &[u8]) -> Result<Vec<u8>> {
        todo!()
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

    #[tokio::test]
    async fn test_account() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::account(config).await
    }

    #[tokio::test]
    async fn test_construction() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::construction(config).await
    }
}
