use anyhow::{Context, Result};
use parity_scale_codec::Decode;
use rosetta_config_ethereum::{
    ext::types::{H160, H256},
    EthereumMetadata, EthereumMetadataParams, Query as EthQuery, QueryResult as EthQueryResult,
};
use rosetta_config_humanode::metadata::{
    dev as humanode_metadata,
    dev::runtime_types::{frame_system::AccountInfo, pallet_balances::AccountData},
};
use rosetta_core::{
    crypto::{
        address::{Address, AddressFormat},
        PublicKey,
    },
    types::{BlockIdentifier, PartialBlockIdentifier},
    BlockchainClient, BlockchainConfig,
};
use rosetta_server::ws::default_client;
use rosetta_server_ethereum::MaybeWsEthereumClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use subxt::{
    backend::{
        legacy::{rpc_methods::BlockNumber, LegacyBackendBuilder, LegacyRpcMethods},
        rpc::RpcClient,
        BlockRef,
    },
    config::substrate::U256,
    dynamic::Value as SubtxValue,
    ext::sp_core::{self, crypto::Ss58AddressFormat},
    tx::PairSigner,
    utils::AccountId32,
    OnlineClient, PolkadotConfig,
};

/// Re-exports libraries to not require any additional
/// dependencies to be explicitly added on the client side.
#[doc(hidden)]
pub mod ext {
    pub use anyhow;
    pub use rosetta_config_ethereum as ethereum_config;
    pub use rosetta_config_humanode as humanode_config;
    pub use rosetta_core as core;
    pub use subxt;
}

#[derive(Deserialize, Serialize)]
pub struct HumanodeMetadataParams(pub EthereumMetadataParams);

#[derive(Deserialize, Serialize)]
pub struct HumanodeMetadata(pub EthereumMetadata);

pub struct HumanodeClient {
    client: MaybeWsEthereumClient,
    ws_client: OnlineClient<PolkadotConfig>,
    rpc_methods: LegacyRpcMethods<PolkadotConfig>,
}

impl HumanodeClient {
    /// Creates a new polkadot client, loading the config from `network` and connects to `addr`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn new(network: &str, url: &str) -> Result<Self> {
        let config = rosetta_config_humanode::config(network)?;
        Self::from_config(config, url).await
    }

    /// Creates a new polkadot client using the provided `config` and connects to `addr`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn from_config(config: BlockchainConfig, url: &str) -> Result<Self> {
        let ws_client = default_client(url, None).await?;
        let rpc_client = RpcClient::new(ws_client.clone());
        let rpc_methods = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client.clone());
        let backend = LegacyBackendBuilder::new().build(rpc_client);
        let substrate_client =
            OnlineClient::<PolkadotConfig>::from_backend(Arc::new(backend)).await?;
        let ethereum_client =
            MaybeWsEthereumClient::from_jsonrpsee(config, ws_client, None).await?;
        Ok(Self { client: ethereum_client, ws_client: substrate_client, rpc_methods })
    }

    async fn account_info(
        &self,
        address: &Address,
        maybe_block: Option<&PartialBlockIdentifier>,
    ) -> Result<AccountInfo<u32, AccountData<u128>>> {
        let account: AccountId32 = address
            .address()
            .parse()
            .map_err(|err| anyhow::anyhow!("{}", err))
            .context("invalid address")?;

        // Build a dynamic storage query to iterate account information.
        let storage_query =
            subxt::dynamic::storage("System", "Account", vec![SubtxValue::from_bytes(account)]);

        // TODO: Change the `PartialBlockIdentifier` for distinguish between ethereum blocks and
        // substrate blocks.
        let block_hash = match maybe_block {
            Some(PartialBlockIdentifier { hash: Some(block_hash), .. }) => {
                // If a hash if provided, we don't know if it's a ethereum block hash or substrate
                // block hash. We try to fetch the block using ethereum first, and
                // if it fails, we try to fetch it using substrate.
                let ethereum_block = self
                    .client
                    .call(&EthQuery::GetBlockByHash(H256(*block_hash).into()))
                    .await
                    .map(|result| match result {
                        EthQueryResult::GetBlockByHash(block) => block,
                        _ => unreachable!(),
                    });

                if let Ok(Some(ethereum_block)) = ethereum_block {
                    // Convert ethereum block to substrate block by fetching the block by number.
                    let substrate_block_number =
                        BlockNumber::Number(ethereum_block.header().number());
                    let substrate_block_hash = self
                        .rpc_methods
                        .chain_get_block_hash(Some(substrate_block_number))
                        .await?
                        .map(BlockRef::from_hash)
                        .ok_or_else(|| anyhow::anyhow!("no block hash found"))?;

                    // Verify if the ethereum block belongs to this substrate block.
                    let query_current_eth_block =
                        humanode_metadata::storage().ethereum().current_block();

                    // Fetch ethereum block from `ethereum.current_block` state.
                    let Some(actual_eth_block) = self
                        .ws_client
                        .storage()
                        .at(substrate_block_hash.clone())
                        .fetch(&query_current_eth_block)
                        .await?
                    else {
                        // This error should not happen, once all humanode blocks must have one
                        // ethereum block
                        anyhow::bail!("[report this bug!] no ethereum block found for humanode at block {substrate_block_hash:?}");
                    };

                    // Verify if the ethereum block hash matches the provided ethereum block hash.
                    // TODO: compute the block hash
                    if U256(actual_eth_block.header.number.0) !=
                        U256::from(ethereum_block.header().number())
                    {
                        anyhow::bail!("ethereum block hash mismatch");
                    }
                    if actual_eth_block.header.parent_hash.as_fixed_bytes() !=
                        &ethereum_block.header().header().parent_hash.0
                    {
                        anyhow::bail!("ethereum block hash mismatch");
                    }
                    substrate_block_hash
                } else {
                    self.rpc_methods
                        .chain_get_block_hash(Some(BlockNumber::Hex(U256::from_big_endian(
                            block_hash,
                        ))))
                        .await?
                        .map(BlockRef::from_hash)
                        .ok_or_else(|| anyhow::anyhow!("no block hash found"))?
                }
            },
            Some(PartialBlockIdentifier { index: Some(block_number), .. }) => {
                // If a block number is provided, the value is the same for ethereum blocks and
                // substrate blocks.
                self.rpc_methods
                    .chain_get_block_hash(Some(BlockNumber::Number(*block_number)))
                    .await?
                    .map(BlockRef::from_hash)
                    .ok_or_else(|| anyhow::anyhow!("no block hash found"))?
            },
            Some(PartialBlockIdentifier { .. }) | None => self
                .rpc_methods
                .chain_get_block_hash(None)
                .await?
                .map(BlockRef::from_hash)
                .ok_or_else(|| anyhow::anyhow!("no block hash found"))?,
        };

        let account_info = self.ws_client.storage().at(block_hash).fetch(&storage_query).await?;
        account_info.map_or_else(
            || {
                Ok(AccountInfo::<u32, AccountData<u128>> {
                    nonce: 0,
                    consumers: 0,
                    providers: 0,
                    sufficients: 0,
                    data: AccountData { free: 0, reserved: 0, misc_frozen: 0, fee_frozen: 0 },
                })
            },
            |account_info| {
                <AccountInfo<u32, AccountData<u128>>>::decode(&mut account_info.encoded())
                    .map_err(|_| anyhow::anyhow!("invalid format"))
            },
        )
    }
}

#[async_trait::async_trait]
impl BlockchainClient for HumanodeClient {
    type MetadataParams = HumanodeMetadataParams;
    type Metadata = HumanodeMetadata;
    type EventStream<'a> = <MaybeWsEthereumClient as BlockchainClient>::EventStream<'a>;
    type Call = EthQuery;
    type CallResult = EthQueryResult;

    type AtBlock = PartialBlockIdentifier;
    type BlockIdentifier = BlockIdentifier;

    type Query = rosetta_config_ethereum::Query;
    type Transaction = rosetta_config_ethereum::SignedTransaction;
    type Subscription = <MaybeWsEthereumClient as BlockchainClient>::Subscription;
    type Event = <MaybeWsEthereumClient as BlockchainClient>::Event;
    type SubmitResult = <MaybeWsEthereumClient as BlockchainClient>::SubmitResult;

    async fn query(
        &self,
        query: Self::Query,
    ) -> Result<<Self::Query as rosetta_core::traits::Query>::Result> {
        self.client.query(query).await
    }

    fn config(&self) -> &BlockchainConfig {
        self.client.config()
    }

    fn genesis_block(&self) -> Self::BlockIdentifier {
        self.client.genesis_block()
    }

    async fn current_block(&self) -> Result<Self::BlockIdentifier> {
        self.client.current_block().await
    }

    async fn finalized_block(&self) -> Result<Self::BlockIdentifier> {
        self.client.finalized_block().await
    }

    async fn balance(&self, address: &Address, block: &Self::AtBlock) -> Result<u128> {
        let balance = match address.format() {
            AddressFormat::Ss58(_) => {
                let account_info = self.account_info(address, Some(block)).await?;
                account_info.data.free
            },
            AddressFormat::Eip55 => {
                // Frontier `eth_getBalance` returns the reducible_balance instead the free balance:
                // https://github.com/paritytech/frontier/blob/polkadot-v0.9.43/frame/evm/src/lib.rs#L853-L855
                // using substrate to get the free balance
                let address = address
                    .evm_to_ss58(Ss58AddressFormat::custom(42))
                    .map_err(|err| anyhow::anyhow!("{}", err))?;
                let account_info = self.account_info(&address, Some(block)).await?;
                account_info.data.free
            },
            AddressFormat::Bech32(_) => return Err(anyhow::anyhow!("invalid address format")),
        };
        Ok(balance)
    }

    async fn faucet(
        &self,
        address: &Address,
        value: u128,
        _high_gas_price: Option<u128>,
    ) -> Result<Vec<u8>> {
        // convert address
        let dest = {
            let address: H160 = address.address().parse()?;
            let mut data = [0u8; 24];
            data[0..4].copy_from_slice(b"evm:");
            data[4..24].copy_from_slice(&address[..]);
            let hash = sp_core::hashing::blake2_256(&data);
            AccountId32::from(Into::<[u8; 32]>::into(hash))
        };

        // Build the transfer transaction
        let balance_transfer_tx = humanode_metadata::tx().balances().transfer(dest.into(), value);
        let alice = sp_keyring::AccountKeyring::Alice.pair();
        let signer = PairSigner::<PolkadotConfig, _>::new(alice);

        let hash = self
            .ws_client
            .tx()
            .sign_and_submit_then_watch_default(&balance_transfer_tx, &signer)
            .await?
            .wait_for_finalized_success()
            .await?
            .extrinsic_hash();
        Ok(hash.0.to_vec())
    }

    async fn metadata(
        &self,
        public_key: &PublicKey,
        options: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        Ok(HumanodeMetadata(self.client.metadata(public_key, &options.0).await?))
    }

    async fn submit(&self, transaction: &[u8]) -> Result<Self::SubmitResult> {
        self.client.submit(transaction).await
    }

    async fn call(&self, req: &EthQuery) -> Result<EthQueryResult> {
        self.client.call(req).await
    }

    async fn listen<'a>(&'a self) -> Result<Option<Self::EventStream<'a>>> {
        self.client.listen().await
    }
    async fn subscribe(&self, _sub: &Self::Subscription) -> Result<u32> {
        anyhow::bail!("not implemented");
    }
}

#[allow(clippy::ignored_unit_patterns, clippy::pub_underscore_fields)]
#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_types::{sol, SolCall};
    use ethers_solc::{artifacts::Source, CompilerInput, EvmVersion, Solc};
    use hex_literal::hex;
    use rosetta_chain_testing::run_test;
    use rosetta_client::Wallet;
    use rosetta_config_ethereum::{AtBlock, CallResult};
    use sha3::Digest;
    use std::{collections::BTreeMap, path::Path};

    /// Humanode rpc url
    const HUMANODE_RPC_WS_URL: &str = "ws://127.0.0.1:9944";

    sol! {
        interface TestContract {
            event AnEvent();
            function emitEvent() external;

            function identity(bool a) external view returns (bool);
        }
    }

    #[tokio::test]
    async fn test_network_status() {
        // let config = rosetta_config_humanode::config("dev")?;
        // rosetta_docker::tests::network_status::<HumanodeClient, _, _>(client_from_config,
        // config).await;

        run_test(async move {
            let client = HumanodeClient::new("dev", HUMANODE_RPC_WS_URL)
                .await
                .expect("Error creating client");
            // Check if the genesis is consistent
            let genesis_block = client.genesis_block();
            assert_eq!(genesis_block.index, 0);

            // Check if the current block is consistent
            let current_block = client.current_block().await.unwrap();
            if current_block.index > 0 {
                assert_ne!(current_block.hash, genesis_block.hash);
            } else {
                assert_eq!(current_block.hash, genesis_block.hash);
            }

            // Check if the finalized block is consistent
            let finalized_block = client.finalized_block().await.unwrap();
            assert!(finalized_block.index >= genesis_block.index);
        })
        .await;
    }

    #[tokio::test]
    async fn test_account() {
        run_test(async move {
            let client = HumanodeClient::new("dev", HUMANODE_RPC_WS_URL)
                .await
                .expect("Error creating BinanceClient");
            let wallet =
                Wallet::from_config(client.config().clone(), HUMANODE_RPC_WS_URL, None, None)
                    .await
                    .unwrap();
            let value = 10 * u128::pow(10, client.config().currency_decimals);
            let _ = wallet.faucet(value, None).await;
            let amount = wallet.balance().await.unwrap();
            assert_eq!(amount, value);
        })
        .await;
    }

    #[tokio::test]
    async fn test_construction() {
        run_test(async move {
            let client = HumanodeClient::new("dev", HUMANODE_RPC_WS_URL)
                .await
                .expect("Error creating BinanceClient");
            let faucet = 100 * u128::pow(10, client.config().currency_decimals);
            let value = u128::pow(10, client.config().currency_decimals);
            let alice =
                Wallet::from_config(client.config().clone(), HUMANODE_RPC_WS_URL, None, None)
                    .await
                    .unwrap();
            let bob = Wallet::from_config(client.config().clone(), HUMANODE_RPC_WS_URL, None, None)
                .await
                .unwrap();
            assert_ne!(alice.public_key(), bob.public_key());

            // Alice and bob have no balance
            let balance = alice.balance().await.unwrap();
            assert_eq!(balance, 0);
            let balance = bob.balance().await.unwrap();
            assert_eq!(balance, 0);

            // Transfer faucets to alice
            alice.faucet(faucet, None).await.unwrap();
            let balance = alice.balance().await.unwrap();
            assert_eq!(balance, faucet);

            // Alice transfers to bob
            alice.transfer(bob.account(), value, None, None).await.unwrap();
            let amount = bob.balance().await.unwrap();
            assert_eq!(amount, value);
        })
        .await;
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
    async fn test_smart_contract() {
        run_test(async move {
            let client = HumanodeClient::new("dev", HUMANODE_RPC_WS_URL)
                .await
                .expect("Error creating BinanceClient");
            let faucet = 10 * u128::pow(10, client.config().currency_decimals);
            let wallet =
                Wallet::from_config(client.config().clone(), HUMANODE_RPC_WS_URL, None, None)
                    .await
                    .unwrap();
            wallet.faucet(faucet, None).await.unwrap();

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
        })
        .await;
    }

    #[tokio::test]
    async fn test_smart_contract_view() {
        run_test(async move {
            let client = HumanodeClient::new("dev", HUMANODE_RPC_WS_URL)
                .await
                .expect("Error creating BinanceClient");
            let faucet = 10 * u128::pow(10, client.config().currency_decimals);
            let wallet =
                Wallet::from_config(client.config().clone(), HUMANODE_RPC_WS_URL, None, None)
                    .await
                    .unwrap();
            wallet.faucet(faucet, None).await.unwrap();
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
                    hex!("0000000000000000000000000000000000000000000000000000000000000001")
                        .to_vec()
                )
            );
        })
        .await;
    }
}
