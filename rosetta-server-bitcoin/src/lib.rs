use anyhow::Result;
use bitcoincore_rpc_async::{Auth, Client, RpcApi};
use rosetta_docker::BlockchainNode;
use rosetta_server::types::{BlockIdentifier, NetworkIdentifier};
use rosetta_server::BlockchainClient;
use std::net::SocketAddr;

pub struct BitcoinClient {
    client: Client,
    network: NetworkIdentifier,
    node_version: String,
    genesis_block: BlockIdentifier,
}

#[async_trait::async_trait]
impl BlockchainClient for BitcoinClient {
    async fn new(network: &str, addr: &str) -> Result<Self> {
        let client = Client::new(
            addr.into(),
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
        let network = NetworkIdentifier {
            blockchain: "bitcoin".into(),
            network: network.into(),
            sub_network_identifier: None,
        };
        Ok(Self {
            client,
            network,
            node_version,
            genesis_block,
        })
    }

    fn network(&self) -> &NetworkIdentifier {
        &self.network
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

pub struct BitcoinNode {
    name: String,
    command: Vec<String>,
    expose: Vec<u16>,
}

impl BlockchainNode for BitcoinNode {
    fn new(network: &str, addr: SocketAddr) -> Self {
        let name = format!("bitcoin-{}", network);
        let command = vec![
            format!("-{}=1", network),
            format!("-rpcbind={}", addr.ip()),
            format!("-rpcport={}", addr.port()),
            "-rpcallowip=0.0.0.0/0".into(),
            "-rpcuser=rosetta".into(),
            "-rpcpassword=rosetta".into(),
        ];
        let expose = vec![addr.port()];
        Self {
            name,
            command,
            expose,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn image(&self) -> &str {
        "ruimarinho/bitcoin-core"
    }

    fn command(&self) -> &[String] {
        &self.command
    }

    fn expose(&self) -> &[u16] {
        &self.expose
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rosetta_docker::Docker;

    #[tokio::test]
    async fn run_bitcoin() -> Result<()> {
        let docker = Docker::new()?;
        let addr = "0.0.0.0:18443".parse().unwrap();
        let node = BitcoinNode::new("regtest", addr);
        let handle = docker.run_node(&node).await?;
        let addr = "127.0.0.1:18443".parse().unwrap();
        rosetta_docker::wait_for_http("http://127.0.0.1:18443").await?;
        let client = BitcoinClient::new("regtest", addr).await?;
        println!("network: {:?}", client.network());
        println!("node_version: {}", client.node_version());
        println!("genesis_block: {:?}", client.genesis_block());
        println!("current_block: {:?}", client.current_block().await?);
        handle.stop().await?;
        Ok(())
    }
}
