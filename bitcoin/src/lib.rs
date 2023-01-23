use anyhow::Result;
use bitcoincore_rpc_async::{Auth, Client, RpcApi};
use rosetta::crypto::address::AddressFormat;
use rosetta::crypto::Algorithm;
use rosetta::types::BlockIdentifier;
use rosetta::{BlockchainClient, BlockchainConfig};

pub fn config(network: &str) -> Result<BlockchainConfig> {
    anyhow::ensure!(network == "regtest");
    Ok(BlockchainConfig {
        blockchain: "bitcoin",
        network: "regtest",
        algorithm: Algorithm::EcdsaSecp256k1,
        address_format: AddressFormat::Bech32("bcrt"),
        coin: 1,
        bip44: true,
        utxo: true,
        currency_unit: "satoshi",
        currency_symbol: "tBTC",
        currency_decimals: 10,
        node_port: 18443,
        node_image: "ruimarinho/bitcoin-core",
        node_command: vec![
            "-regtest=1".into(),
            "-rpcbind=0.0.0.0".into(),
            "-rpcport=18443".into(),
            "-rpcallowip=0.0.0.0/0".into(),
            "-rpcuser=rosetta".into(),
            "-rpcpassword=rosetta".into(),
        ],
        node_additional_ports: &[],
        connector_port: 8080,
    })
}

pub struct BitcoinClient {
    config: BlockchainConfig,
    client: Client,
    node_version: String,
    genesis_block: BlockIdentifier,
}

#[async_trait::async_trait]
impl BlockchainClient for BitcoinClient {
    async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = config(network)?;
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
    use rosetta::types::NetworkRequest;
    use rosetta_docker::Docker;

    #[tokio::test]
    async fn test_bitcoin_client() -> Result<()> {
        env_logger::try_init().ok();
        let docker = Docker::new()?;
        let config = config("regtest")?;
        let client = docker.node::<BitcoinClient>(&config).await?;
        println!("node_version: {}", client.node_version());
        println!("genesis_block: {:?}", client.genesis_block());
        println!("current_block: {:?}", client.current_block().await?);
        docker.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_rosetta_client() -> Result<()> {
        env_logger::try_init().ok();
        let docker = Docker::new()?;
        let config = config("regtest")?;
        let client = docker.connector(&config).await?;
        let req = NetworkRequest {
            network_identifier: config.network(),
            metadata: None,
        };
        let status = client.network_status(&req).await?;
        println!("{:?}", status);
        docker.shutdown().await?;
        Ok(())
    }
}
