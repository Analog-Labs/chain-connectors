use anyhow::Result;
use bitcoincore_rpc_async::{Auth, Client, RpcApi};
use rosetta_server::crypto::address::Address;
use rosetta_server::crypto::{PublicKey, Signature};
use rosetta_server::types::{BlockIdentifier, Coin};
use rosetta_server::{BlockchainClient, BlockchainConfig};

pub struct BitcoinClient {
    config: BlockchainConfig,
    client: Client,
    genesis_block: BlockIdentifier,
}

#[async_trait::async_trait]
impl BlockchainClient for BitcoinClient {
    type Metadata = ();
    type Payload = ();

    async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_bitcoin::config(network)?;
        let client = Client::new(
            addr.to_string(),
            Auth::UserPass("rosetta".into(), "rosetta".into()),
        )
        .await?;
        let genesis = client.get_block_hash(0).await?;
        let genesis_block = BlockIdentifier {
            index: 0,
            hash: hex::encode(genesis.as_ref()),
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
        let info = self.client.get_network_info().await?;
        let major = info.version / 10000;
        let rest = info.version % 10000;
        let minor = rest / 100;
        let patch = rest % 100;
        Ok(format!("{}.{}.{}", major, minor, patch))
    }

    async fn current_block(&self) -> Result<BlockIdentifier> {
        let index = self.client.get_block_count().await?;
        let hash = self.client.get_block_hash(index).await?;
        Ok(BlockIdentifier {
            index,
            hash: hex::encode(hash.as_ref()),
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

    async fn metadata(&self, _public_key: &PublicKey) -> Result<Self::Metadata> {
        todo!()
    }

    async fn combine(&self, _payload: &Self::Payload, _signature: &Signature) -> Result<Vec<u8>> {
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
