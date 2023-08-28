use anyhow::Result;
use client::EthereumClient;
use ethers::providers::Http;
use rosetta_config_ethereum::{EthereumMetadata, EthereumMetadataParams};
use rosetta_server::crypto::address::Address;
use rosetta_server::crypto::PublicKey;
use rosetta_server::types::{
    Block, BlockIdentifier, CallRequest, Coin, PartialBlockIdentifier, Transaction,
    TransactionIdentifier,
};
use rosetta_server::{BlockchainClient, BlockchainConfig};
use serde_json::Value;
use url::Url;

mod client;
mod eth_types;
mod event_stream;
mod proof;
mod utils;
mod ws_provider;

use ws_provider::ExtendedWs;

pub use event_stream::EthereumEventStream;

pub enum MaybeWsEthereumClient {
    Http(EthereumClient<Http>),
    Ws(EthereumClient<ExtendedWs>),
}

impl MaybeWsEthereumClient {
    pub async fn new<S: AsRef<str>>(config: BlockchainConfig, addr: S) -> Result<Self> {
        let addr = addr.as_ref();
        if addr.starts_with("ws://") || addr.starts_with("wss://") {
            let ws_connection = ExtendedWs::connect(addr).await?;
            let client = EthereumClient::new(config, ws_connection).await?;
            Ok(Self::Ws(client))
        } else {
            let http_connection = Http::new(Url::parse(addr)?);
            let client = EthereumClient::new(config, http_connection).await?;
            Ok(Self::Http(client))
        }
    }
}

#[async_trait::async_trait]
impl BlockchainClient for MaybeWsEthereumClient {
    type MetadataParams = EthereumMetadataParams;
    type Metadata = EthereumMetadata;
    type EventStream<'a> = EthereumEventStream<'a, ExtendedWs>;

    fn create_config(network: &str) -> Result<BlockchainConfig> {
        rosetta_config_ethereum::config(network)
    }

    async fn new(config: BlockchainConfig, addr: &str) -> Result<Self> {
        MaybeWsEthereumClient::new(config, addr).await
    }

    fn config(&self) -> &BlockchainConfig {
        match self {
            MaybeWsEthereumClient::Http(http_client) => http_client.config(),
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.config(),
        }
    }

    fn genesis_block(&self) -> &BlockIdentifier {
        match self {
            MaybeWsEthereumClient::Http(http_client) => http_client.genesis_block(),
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.genesis_block(),
        }
    }

    async fn node_version(&self) -> Result<String> {
        match self {
            MaybeWsEthereumClient::Http(http_client) => http_client.node_version().await,
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.node_version().await,
        }
    }

    async fn current_block(&self) -> Result<BlockIdentifier> {
        match self {
            MaybeWsEthereumClient::Http(http_client) => http_client.current_block().await,
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.current_block().await,
        }
    }

    async fn finalized_block(&self) -> Result<BlockIdentifier> {
        match self {
            MaybeWsEthereumClient::Http(http_client) => http_client.finalized_block().await,
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.finalized_block().await,
        }
    }

    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128> {
        match self {
            MaybeWsEthereumClient::Http(http_client) => http_client.balance(address, block).await,
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.balance(address, block).await,
        }
    }

    async fn coins(&self, address: &Address, block: &BlockIdentifier) -> Result<Vec<Coin>> {
        match self {
            MaybeWsEthereumClient::Http(http_client) => http_client.coins(address, block).await,
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.coins(address, block).await,
        }
    }

    async fn faucet(&self, address: &Address, param: u128) -> Result<Vec<u8>> {
        match self {
            MaybeWsEthereumClient::Http(http_client) => http_client.faucet(address, param).await,
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.faucet(address, param).await,
        }
    }

    async fn metadata(
        &self,
        public_key: &PublicKey,
        options: &Self::MetadataParams,
    ) -> Result<EthereumMetadata> {
        match self {
            MaybeWsEthereumClient::Http(http_client) => {
                http_client.metadata(public_key, options).await
            }
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.metadata(public_key, options).await,
        }
    }

    async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        match self {
            MaybeWsEthereumClient::Http(http_client) => http_client.submit(transaction).await,
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.submit(transaction).await,
        }
    }

    async fn block(&self, block_identifier: &PartialBlockIdentifier) -> Result<Block> {
        match self {
            MaybeWsEthereumClient::Http(http_client) => http_client.block(block_identifier).await,
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.block(block_identifier).await,
        }
    }

    async fn block_transaction(
        &self,
        block: &BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        match self {
            MaybeWsEthereumClient::Http(http_client) => {
                http_client.block_transaction(block, tx).await
            }
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.block_transaction(block, tx).await,
        }
    }

    async fn call(&self, req: &CallRequest) -> Result<Value> {
        match self {
            MaybeWsEthereumClient::Http(http_client) => http_client.call(req).await,
            MaybeWsEthereumClient::Ws(ws_client) => ws_client.call(req).await,
        }
    }

    async fn listen<'a>(&'a self) -> Result<Option<Self::EventStream<'a>>> {
        match self {
            MaybeWsEthereumClient::Http(_) => Ok(None),
            MaybeWsEthereumClient::Ws(ws_client) => {
                let subscription = ws_client.listen().await?;
                Ok(Some(subscription))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers_solc::artifacts::Source;
    use ethers_solc::{CompilerInput, EvmVersion, Solc};
    use rosetta_client::EthereumExt;
    use rosetta_docker::Env;
    use sha3::Digest;
    use std::collections::BTreeMap;
    use std::path::Path;

    #[tokio::test]
    async fn test_network_list() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::network_list(config).await
    }

    #[tokio::test]
    async fn test_network_options() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::network_options::<MaybeWsEthereumClient>(config).await
    }

    #[tokio::test]
    async fn test_network_status() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::network_status::<MaybeWsEthereumClient>(config).await
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

    fn compile_snippet(source: &str) -> Result<Vec<u8>> {
        let solc = Solc::default();
        let source = format!("contract Contract {{ {source} }}");
        let mut sources = BTreeMap::new();
        sources.insert(Path::new("contract.sol").into(), Source::new(source));
        let input = CompilerInput::with_sources(sources)[0]
            .clone()
            .evm_version(EvmVersion::Homestead);
        let output = solc.compile_exact(&input)?;
        let file = output.contracts.get("contract.sol").unwrap();
        let contract = file.get("Contract").unwrap();
        let bytecode = contract
            .evm
            .as_ref()
            .unwrap()
            .bytecode
            .as_ref()
            .unwrap()
            .object
            .as_bytes()
            .unwrap()
            .to_vec();
        Ok(bytecode)
    }

    #[tokio::test]
    async fn test_smart_contract() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;

        let env = Env::new("ethereum-smart-contract", config.clone()).await?;

        let faucet = 100 * u128::pow(10, config.currency_decimals);
        let wallet = env.ephemeral_wallet()?;
        wallet.faucet(faucet).await?;

        let bytes = compile_snippet(
            r#"
            event AnEvent();
            function emitEvent() public {
                emit AnEvent();
            }
        "#,
        )?;
        let response = wallet.eth_deploy_contract(bytes).await?;

        let receipt = wallet.eth_transaction_receipt(&response.hash).await?;
        let contract_address = receipt.result["contractAddress"].as_str().unwrap();
        let response = wallet
            .eth_send_call(contract_address, "function emitEvent()", &[], 0)
            .await?;
        let receipt = wallet.eth_transaction_receipt(&response.hash).await?;
        let logs = receipt.result["logs"].as_array().unwrap();
        assert_eq!(logs.len(), 1);
        let topic = logs[0]["topics"][0].as_str().unwrap();
        let expected = format!("0x{}", hex::encode(sha3::Keccak256::digest("AnEvent()")));
        assert_eq!(topic, expected);
        env.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_smart_contract_view() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;

        let env = Env::new("ethereum-smart-contract-view", config.clone()).await?;

        let faucet = 100 * u128::pow(10, config.currency_decimals);
        let wallet = env.ephemeral_wallet()?;
        wallet.faucet(faucet).await?;

        let bytes = compile_snippet(
            r#"
            function identity(bool a) public view returns (bool) {
                return a;
            }
        "#,
        )?;
        let response = wallet.eth_deploy_contract(bytes).await?;
        let receipt = wallet.eth_transaction_receipt(&response.hash).await?;
        let contract_address = receipt.result["contractAddress"].as_str().unwrap();

        let response = wallet
            .eth_view_call(
                contract_address,
                "function identity(bool a) returns (bool)",
                &["true".into()],
                None,
            )
            .await?;
        let result: Vec<String> = serde_json::from_value(response.result)?;
        assert_eq!(result[0], "true");
        env.shutdown().await?;
        Ok(())
    }
}
