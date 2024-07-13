use anyhow::Result;
use block_stream::BlockStream;
pub use client::EthereumClient;
pub use rosetta_config_ethereum::{
    EthereumMetadata, EthereumMetadataParams, Event, Query as EthQuery, QueryItem,
    QueryResult as EthQueryResult, SubmitResult, Subscription,
};
use rosetta_core::{
    crypto::{address::Address, PublicKey},
    types::{BlockIdentifier, PartialBlockIdentifier},
    BlockchainClient, BlockchainConfig,
};
use rosetta_ethereum_backend::jsonrpsee::Adapter;
use rosetta_server::ws::{default_client, default_http_client, DefaultClient, HttpClient};
use url::Url;

mod block_stream;
mod client;
mod event_stream;
mod finalized_block_stream;
mod log_filter;
mod multi_block;
mod new_heads;
mod proof;
mod shared_stream;
mod state;
mod utils;

pub use event_stream::EthereumEventStream;

pub mod config {
    pub use rosetta_config_ethereum::*;
}

/// Re-exports libraries to not require any additional
/// dependencies to be explicitly added on the client side.
#[doc(hidden)]
pub mod ext {
    pub use anyhow;
    pub use rosetta_config_ethereum as config;
    pub use rosetta_core as core;
    pub use rosetta_ethereum_backend as backend;
}

#[derive(Clone)]
pub enum MaybeWsEthereumClient {
    Http(EthereumClient<HttpClient>),
    Ws(EthereumClient<DefaultClient>),
}

impl MaybeWsEthereumClient {
    /// Creates a new ethereum client from `network` and `addr`.
    /// Supported blockchains are `ethereum`, `polygon` and `arbitrum`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn new<S: AsRef<str> + Send>(
        blockchain: &str,
        network: &str,
        addr: S,
        private_key: Option<[u8; 32]>,
    ) -> Result<Self> {
        let config = match blockchain {
            "ethereum" => rosetta_config_ethereum::config(network)?,
            "polygon" => rosetta_config_ethereum::polygon_config(network)?,
            "arbitrum" => rosetta_config_ethereum::arbitrum_config(network)?,
            blockchain => anyhow::bail!("unsupported blockchain: {blockchain}"),
        };
        Self::from_config(config, addr, private_key).await
    }

    /// Creates a new ethereum client from `config` and `addr`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn from_config<S: AsRef<str> + Send>(
        config: BlockchainConfig,
        addr: S,
        private_key: Option<[u8; 32]>,
    ) -> Result<Self> {
        let uri = Url::parse(addr.as_ref())?;
        if uri.scheme() == "ws" || uri.scheme() == "wss" {
            let client = default_client(uri.as_str(), None).await?;
            Self::from_jsonrpsee(config, client, private_key).await
        } else {
            let http_connection = default_http_client(uri.as_str())?;
            // let http_connection = Http::new(uri);
            let client = EthereumClient::new(config, http_connection, private_key).await?;
            Ok(Self::Http(client))
        }
    }

    /// Creates a new Ethereum Client from the provided `JsonRpsee` client,
    /// this method is useful for reusing the same rpc client for ethereum and substrate calls.
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn from_jsonrpsee(
        config: BlockchainConfig,
        client: DefaultClient,
        private_key: Option<[u8; 32]>,
    ) -> Result<Self> {
        let client = EthereumClient::new(config, client, private_key).await?;
        Ok(Self::Ws(client))
    }
}

#[async_trait::async_trait]
impl BlockchainClient for MaybeWsEthereumClient {
    type MetadataParams = EthereumMetadataParams;
    type Metadata = EthereumMetadata;
    type EventStream<'a> = shared_stream::SharedStream<BlockStream<Adapter<DefaultClient>>> where Self: 'a;
    type Call = EthQuery;
    type CallResult = EthQueryResult;

    type AtBlock = PartialBlockIdentifier;
    type BlockIdentifier = BlockIdentifier;

    type Query = EthQuery;
    type Transaction = rosetta_config_ethereum::SignedTransaction;
    type Subscription = Subscription;
    type Event = Event;
    type SubmitResult = SubmitResult;

    async fn query(
        &self,
        query: Self::Query,
    ) -> Result<<Self::Query as rosetta_core::traits::Query>::Result> {
        match self {
            Self::Http(http_client) => http_client.call(&query).await,
            Self::Ws(ws_client) => ws_client.call(&query).await,
        }
    }

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
        Ok(BlockIdentifier { index: block.header().number(), hash: block.header().hash().0 })
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

    async fn submit(&self, transaction: &[u8]) -> Result<Self::SubmitResult> {
        match self {
            Self::Http(http_client) => http_client.submit(transaction).await,
            Self::Ws(ws_client) => ws_client.submit(transaction).await,
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

    async fn subscribe(&self, sub: &Self::Subscription) -> Result<u32> {
        match self {
            Self::Http(http_client) => http_client.subscribe(sub),
            Self::Ws(ws_client) => ws_client.subscribe(sub),
        }
    }
}

#[allow(clippy::ignored_unit_patterns, clippy::pub_underscore_fields)]
#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_types::{sol, SolCall};
    use ethers_solc::{artifacts::Source, CompilerInput, EvmVersion, Solc};
    use rosetta_config_ethereum::{ext::types::H256, query::GetLogs, AtBlock, CallResult};
    use rosetta_docker::{run_test, Env};
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
        MaybeWsEthereumClient::from_config(config, url.as_str(), None).await
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
    #[allow(clippy::needless_raw_string_hashes)]
    async fn test_smart_contract() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev").unwrap();

        let env = Env::new("ethereum-smart-contract", config.clone(), client_from_config).await?;

        run_test(env, |env| async move {
            let wallet = env.ephemeral_wallet().await.unwrap();

            let faucet = 100 * u128::pow(10, config.currency_decimals);
            wallet.faucet(faucet).await.unwrap();

            let bytes = compile_snippet(
                r"
                    event AnEvent();
                    function emitEvent() public {
                        emit AnEvent();
                    }
                ",
            )
            .unwrap();
            let tx_hash = wallet.eth_deploy_contract(bytes).await.unwrap().tx_hash().0;
            let receipt = wallet.eth_transaction_receipt(tx_hash).await.unwrap().unwrap();
            let contract_address = receipt.contract_address.unwrap();
            let tx_hash = {
                let call = TestContract::emitEventCall {};
                wallet
                    .eth_send_call(contract_address.0, call.abi_encode(), 0, None, None)
                    .await
                    .unwrap()
                    .tx_hash()
                    .0
            };
            let receipt = wallet.eth_transaction_receipt(tx_hash).await.unwrap().unwrap();
            assert_eq!(receipt.logs.len(), 1);
            let topic = receipt.logs[0].topics[0];
            let expected = H256(sha3::Keccak256::digest("AnEvent()").into());
            assert_eq!(topic, expected);

            let block_hash = receipt.block_hash;
            let block_number = receipt.block_number.unwrap();
            assert_eq!(topic, expected);

            let logs = wallet
                .query(GetLogs {
                    contracts: vec![contract_address],
                    topics: vec![topic],
                    block: AtBlock::At(block_hash.into()).into(),
                })
                .await
                .unwrap();
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].topics[0], topic);

            let logs = wallet
                .query(GetLogs {
                    contracts: vec![contract_address],
                    topics: vec![topic],
                    block: AtBlock::At(block_number.into()).into(),
                })
                .await
                .unwrap();
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].topics[0], topic);
        })
        .await;
        Ok(())
    }

    #[tokio::test]
    #[allow(clippy::needless_raw_string_hashes)]
    async fn test_smart_contract_view() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev").unwrap();
        let env = Env::new("ethereum-smart-contract-logs-view", config.clone(), client_from_config)
            .await
            .unwrap();

        run_test(env, |env| async move {
            let wallet = env.ephemeral_wallet().await.unwrap();
            let faucet = 100 * u128::pow(10, config.currency_decimals);
            wallet.faucet(faucet).await.unwrap();

            let bytes = compile_snippet(
                r"
                function identity(bool a) public view returns (bool) {
                    return a;
                }
            ",
            )
            .unwrap();
            let tx_hash = wallet.eth_deploy_contract(bytes).await.unwrap().tx_hash().0;
            let receipt = wallet.eth_transaction_receipt(tx_hash).await.unwrap().unwrap();
            let contract_address = receipt.contract_address.unwrap();

            let response = {
                let call = TestContract::identityCall { a: true };
                wallet
                    .eth_view_call(contract_address.0, call.abi_encode(), AtBlock::Latest)
                    .await
                    .unwrap()
            };
            assert_eq!(
                response,
                CallResult::Success(
                    [
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 1
                    ]
                    .to_vec()
                )
            );
        })
        .await;
        Ok(())
    }

    #[tokio::test]
    async fn test_subscription() -> Result<()> {
        use futures_util::StreamExt;
        use rosetta_core::{BlockOrIdentifier, ClientEvent};
        let config = rosetta_config_ethereum::config("dev").unwrap();
        let env = Env::new("ethereum-subscription", config.clone(), client_from_config)
            .await
            .unwrap();

        run_test(env, |env| async move {
            let wallet = env.ephemeral_wallet().await.unwrap();
            let mut stream = wallet.listen().await.unwrap().unwrap();

            let mut last_head: Option<u64> = None;
            let mut last_finalized: Option<u64> = None;
            for _ in 0..10 {
                let event = stream.next().await.unwrap();
                match event {
                    ClientEvent::NewHead(BlockOrIdentifier::Identifier(head)) => {
                        if let Some(block_number) = last_head {
                            assert!(head.index > block_number);
                        }
                        last_head = Some(head.index);
                    },
                    ClientEvent::NewFinalized(BlockOrIdentifier::Identifier(finalized)) => {
                        if let Some(block_number) = last_finalized {
                            assert!(finalized.index > block_number);
                        }
                        last_finalized = Some(finalized.index);
                    },
                    event => panic!("unexpected event: {event:?}"),
                }
            }
            assert!(last_head.is_some());
            assert!(last_finalized.is_some());
        })
        .await;
        Ok(())
    }
}
