use anyhow::Result;
use client::EthereumClient;
use ethers::providers::Http;
pub use rosetta_config_ethereum::{
    EthereumMetadata, EthereumMetadataParams, Query as EthQuery, QueryResult as EthQueryResult,
};
use rosetta_core::{
    crypto::{address::Address, PublicKey},
    types::{Block, BlockIdentifier, PartialBlockIdentifier, Transaction, TransactionIdentifier},
    BlockchainClient, BlockchainConfig,
};
use rosetta_server::ws::{default_client, DefaultClient};
use url::Url;

mod client;
mod eth_types;
mod event_stream;
mod proof;
mod utils;

use rosetta_ethereum_rpc_client::EthPubsubAdapter;

pub use event_stream::EthereumEventStream;

pub mod config {
    pub use rosetta_config_ethereum::*;
}

#[derive(Clone)]
pub enum MaybeWsEthereumClient {
    Http(EthereumClient<Http>),
    Ws(EthereumClient<EthPubsubAdapter<DefaultClient>>),
}

impl MaybeWsEthereumClient {
    /// Creates a new ethereum client from `network` and `addr`.
    /// Supported blockchains are `ethereum` and `polygon`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn new<S: AsRef<str> + Send>(
        blockchain: &str,
        network: &str,
        addr: S,
    ) -> Result<Self> {
        let config = match blockchain {
            "ethereum" => rosetta_config_ethereum::config(network)?,
            "polygon" => rosetta_config_ethereum::polygon_config(network)?,
            "arbitrum" => rosetta_config_ethereum::arbitrum_config(network)?,
            blockchain => anyhow::bail!("unsupported blockchain: {blockchain}"),
        };
        Self::from_config(config, addr).await
    }

    /// Creates a new ethereum client from `config` and `addr`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn from_config<S: AsRef<str> + Send>(
        config: BlockchainConfig,
        addr: S,
    ) -> Result<Self> {
        let uri = Url::parse(addr.as_ref())?;
        if uri.scheme() == "ws" || uri.scheme() == "wss" {
            let client = default_client(uri.as_str(), None).await?;
            Self::from_jsonrpsee(config, client).await
        } else {
            let http_connection = Http::new(uri);
            let client = EthereumClient::new(config, http_connection).await?;
            Ok(Self::Http(client))
        }
    }

    /// Creates a new Ethereum Client from the provided `JsonRpsee` client,
    /// this method is useful for reusing the same rpc client for ethereum and substrate calls.
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn from_jsonrpsee(config: BlockchainConfig, client: DefaultClient) -> Result<Self> {
        let ws_connection = EthPubsubAdapter::new(client);
        let client = EthereumClient::new(config, ws_connection).await?;
        Ok(Self::Ws(client))
    }
}

#[async_trait::async_trait]
impl BlockchainClient for MaybeWsEthereumClient {
    type MetadataParams = EthereumMetadataParams;
    type Metadata = EthereumMetadata;
    type EventStream<'a> = EthereumEventStream<'a, EthPubsubAdapter<DefaultClient>>;
    type Call = EthQuery;
    type CallResult = EthQueryResult;

    type AtBlock = PartialBlockIdentifier;
    type BlockIdentifier = BlockIdentifier;

    fn config(&self) -> &BlockchainConfig {
        match self {
            Self::Http(http_client) => http_client.config(),
            Self::Ws(ws_client) => ws_client.config(),
        }
    }

    fn genesis_block(&self) -> Self::BlockIdentifier {
        match self {
            Self::Http(http_client) => http_client.genesis_block(),
            Self::Ws(ws_client) => ws_client.genesis_block(),
        }
    }

    async fn node_version(&self) -> Result<String> {
        match self {
            Self::Http(http_client) => http_client.node_version().await,
            Self::Ws(ws_client) => ws_client.node_version().await,
        }
    }

    async fn current_block(&self) -> Result<Self::BlockIdentifier> {
        match self {
            Self::Http(http_client) => http_client.current_block().await,
            Self::Ws(ws_client) => ws_client.current_block().await,
        }
    }

    async fn finalized_block(&self) -> Result<Self::BlockIdentifier> {
        let block = match self {
            Self::Http(http_client) => http_client.finalized_block(None).await?,
            Self::Ws(ws_client) => ws_client.finalized_block(None).await?,
        };
        Ok(BlockIdentifier { index: block.number, hash: block.hash.0 })
    }

    async fn balance(&self, address: &Address, block: &Self::AtBlock) -> Result<u128> {
        match self {
            Self::Http(http_client) => http_client.balance(address, block).await,
            Self::Ws(ws_client) => ws_client.balance(address, block).await,
        }
    }

    async fn faucet(&self, address: &Address, param: u128) -> Result<Vec<u8>> {
        match self {
            Self::Http(http_client) => http_client.faucet(address, param).await,
            Self::Ws(ws_client) => ws_client.faucet(address, param).await,
        }
    }

    async fn metadata(
        &self,
        public_key: &PublicKey,
        options: &Self::MetadataParams,
    ) -> Result<EthereumMetadata> {
        match self {
            Self::Http(http_client) => http_client.metadata(public_key, options).await,
            Self::Ws(ws_client) => ws_client.metadata(public_key, options).await,
        }
    }

    async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        match self {
            Self::Http(http_client) => http_client.submit(transaction).await,
            Self::Ws(ws_client) => ws_client.submit(transaction).await,
        }
    }

    async fn block(&self, block_identifier: &Self::AtBlock) -> Result<Block> {
        match self {
            Self::Http(http_client) => http_client.block(block_identifier).await,
            Self::Ws(ws_client) => ws_client.block(block_identifier).await,
        }
    }

    async fn block_transaction(
        &self,
        block: &Self::BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        match self {
            Self::Http(http_client) => http_client.block_transaction(block, tx).await,
            Self::Ws(ws_client) => ws_client.block_transaction(block, tx).await,
        }
    }

    async fn call(&self, req: &EthQuery) -> Result<EthQueryResult> {
        match self {
            Self::Http(http_client) => http_client.call(req).await,
            Self::Ws(ws_client) => ws_client.call(req).await,
        }
    }

    async fn listen<'a>(&'a self) -> Result<Option<Self::EventStream<'a>>> {
        match self {
            Self::Http(_) => Ok(None),
            Self::Ws(ws_client) => {
                let subscription = ws_client.listen().await?;
                Ok(Some(subscription))
            },
        }
    }
}

#[allow(clippy::ignored_unit_patterns)]
#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_types::{sol, SolCall};
    use ethabi::ethereum_types::H256;
    use ethers_solc::{artifacts::Source, CompilerInput, EvmVersion, Solc};
    use rosetta_config_ethereum::{AtBlock, CallResult};
    use rosetta_docker::Env;
    use sha3::Digest;
    use std::{collections::BTreeMap, path::Path};

    sol! {
        interface TestContract {
            event AnEvent();
            function emitEvent() external;

            function identity(bool a) external view returns (bool);
        }
    }

    pub async fn client_from_config(config: BlockchainConfig) -> Result<MaybeWsEthereumClient> {
        let url = config.node_uri.to_string();
        MaybeWsEthereumClient::from_config(config, url.as_str()).await
    }

    #[tokio::test]
    async fn test_network_status() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_docker::tests::network_status::<MaybeWsEthereumClient, _, _>(
            client_from_config,
            config,
        )
        .await
    }

    #[tokio::test]
    async fn test_account() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_docker::tests::account::<MaybeWsEthereumClient, _, _>(client_from_config, config)
            .await
    }

    #[tokio::test]
    async fn test_construction() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_docker::tests::construction::<MaybeWsEthereumClient, _, _>(
            client_from_config,
            config,
        )
        .await
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

        let env = Env::new("ethereum-smart-contract", config.clone(), client_from_config).await?;

        let faucet = 100 * u128::pow(10, config.currency_decimals);
        let wallet = env.ephemeral_wallet().await?;
        wallet.faucet(faucet).await?;

        let bytes = compile_snippet(
            r"
            event AnEvent();
            function emitEvent() public {
                emit AnEvent();
            }
        ",
        )?;
        let tx_hash = wallet.eth_deploy_contract(bytes).await?;
        let receipt = wallet.eth_transaction_receipt(tx_hash).await?.unwrap();
        let contract_address = receipt.contract_address.unwrap();
        let tx_hash = {
            let call = TestContract::emitEventCall {};
            wallet.eth_send_call(contract_address.0, call.abi_encode(), 0).await?
        };
        let receipt = wallet.eth_transaction_receipt(tx_hash).await?.unwrap();
        assert_eq!(receipt.logs.len(), 1);
        let topic = receipt.logs[0].topics[0];
        let expected = H256(sha3::Keccak256::digest("AnEvent()").into());
        assert_eq!(topic, expected);
        env.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_smart_contract_view() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;

        let env =
            Env::new("ethereum-smart-contract-view", config.clone(), client_from_config).await?;

        let faucet = 100 * u128::pow(10, config.currency_decimals);
        let wallet = env.ephemeral_wallet().await?;
        wallet.faucet(faucet).await?;

        let bytes = compile_snippet(
            r"
            function identity(bool a) public view returns (bool) {
                return a;
            }
        ",
        )?;
        let tx_hash = wallet.eth_deploy_contract(bytes).await?;
        let receipt = wallet.eth_transaction_receipt(tx_hash).await?.unwrap();
        let contract_address = receipt.contract_address.unwrap();

        let response = {
            let call = TestContract::identityCall { a: true };
            wallet
                .eth_view_call(contract_address.0, call.abi_encode(), AtBlock::Latest)
                .await?
        };
        assert_eq!(
            response,
            CallResult::Success(
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 1
                ]
                .to_vec()
            )
        );
        env.shutdown().await?;
        Ok(())
    }
}
