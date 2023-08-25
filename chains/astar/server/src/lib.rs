use anyhow::{Context, Result};
use ethers::prelude::*;
use parity_scale_codec::Decode;
use rosetta_config_astar::metadata::{
    dev as astar_metadata,
    dev::runtime_types::{frame_system::AccountInfo, pallet_balances::types::AccountData},
};
use rosetta_config_ethereum::{EthereumMetadata, EthereumMetadataParams};
use rosetta_server::crypto::address::{Address, AddressFormat};
use rosetta_server::crypto::PublicKey;
use rosetta_server::types::{
    Block, BlockIdentifier, CallRequest, Coin, PartialBlockIdentifier, Transaction,
    TransactionIdentifier,
};
use rosetta_server::{ws::default_client, BlockchainClient, BlockchainConfig};
use rosetta_server_ethereum::MaybeWsEthereumClient;
use serde_json::Value;
use sp_core::crypto::Ss58AddressFormat;
use std::sync::Arc;
use subxt::{
    dynamic::Value as SubtxValue, rpc::types::BlockNumber, tx::PairSigner, utils::AccountId32,
    OnlineClient, PolkadotConfig,
};

pub struct AstarClient {
    client: MaybeWsEthereumClient,
    ws_client: OnlineClient<PolkadotConfig>,
}

impl AstarClient {
    async fn account_info(
        &self,
        address: &Address,
        maybe_block: Option<&BlockIdentifier>,
    ) -> Result<AccountInfo<u32, AccountData<u128>>> {
        let account: AccountId32 = address
            .address()
            .parse()
            .map_err(|err| anyhow::anyhow!("{}", err))
            .context("invalid address")?;

        // Build a dynamic storage query to iterate account information.
        let storage_query =
            subxt::dynamic::storage("System", "Account", vec![SubtxValue::from_bytes(account)]);

        let block_hash = {
            let block_number = maybe_block.map(|block| BlockNumber::from(block.index));
            self.ws_client
                .rpc()
                .block_hash(block_number)
                .await?
                .ok_or_else(|| anyhow::anyhow!("no block hash found"))?
        };

        let account_info = self
            .ws_client
            .storage()
            .at(block_hash)
            .fetch(&storage_query)
            .await?
            .ok_or_else(|| anyhow::anyhow!("account not found"))?;

        <AccountInfo<u32, AccountData<u128>>>::decode(&mut account_info.encoded())
            .map_err(|_| anyhow::anyhow!("invalid format"))
    }
}

#[async_trait::async_trait]
impl BlockchainClient for AstarClient {
    type MetadataParams = EthereumMetadataParams;
    type Metadata = EthereumMetadata;

    fn create_config(network: &str) -> Result<BlockchainConfig> {
        rosetta_config_astar::config(network)
    }

    async fn new(config: BlockchainConfig, addr: &str) -> Result<Self> {
        // TODO: Fix this hack, need to support multiple addresses per node
        let (http_uri, ws_uri) = if let Some(addr_without_scheme) = addr.strip_prefix("ws://") {
            (format!("http://{addr_without_scheme}"), addr.to_string())
        } else if let Some(addr_without_scheme) = addr.strip_prefix("wss://") {
            (format!("https://{addr_without_scheme}"), addr.to_string())
        } else if let Some(addr_without_scheme) = addr.strip_prefix("http://") {
            (addr.to_string(), format!("ws://{addr_without_scheme}"))
        } else if let Some(addr_without_scheme) = addr.strip_prefix("https://") {
            (addr.to_string(), format!("wss://{addr_without_scheme}"))
        } else {
            (format!("http://{addr}"), format!("ws://{addr}"))
        };
        let substrate_client = {
            let client = default_client(ws_uri.as_str(), None).await?;
            log::info!("Connected to {}", ws_uri.as_str());
            OnlineClient::<PolkadotConfig>::from_rpc_client(Arc::new(client)).await?
        };
        let ethereum_client = MaybeWsEthereumClient::new(config, http_uri.as_str()).await?;
        Ok(Self {
            client: ethereum_client,
            ws_client: substrate_client,
        })
    }

    fn config(&self) -> &BlockchainConfig {
        self.client.config()
    }

    fn genesis_block(&self) -> &BlockIdentifier {
        self.client.genesis_block()
    }

    async fn node_version(&self) -> Result<String> {
        self.client.node_version().await
    }

    async fn current_block(&self) -> Result<BlockIdentifier> {
        self.client.current_block().await
    }

    async fn finalized_block(&self) -> Result<BlockIdentifier> {
        self.client.finalized_block().await
    }

    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128> {
        let balance = match address.format() {
            AddressFormat::Ss58(_) => {
                let account_info = self.account_info(address, Some(block)).await?;
                account_info.data.free
            }
            AddressFormat::Eip55 => {
                // Frontier `eth_getBalance` returns the reducible_balance instead the free balance:
                // https://github.com/paritytech/frontier/blob/polkadot-v0.9.43/frame/evm/src/lib.rs#L853-L855
                // using substrate to get the free balance
                let address = address
                    .evm_to_ss58(Ss58AddressFormat::custom(42))
                    .map_err(|err| anyhow::anyhow!("{}", err))?;
                let account_info = self.account_info(&address, Some(block)).await?;
                account_info.data.free
            }
            _ => {
                return Err(anyhow::anyhow!("invalid address format"));
            }
        };
        Ok(balance)
    }

    async fn coins(&self, address: &Address, block: &BlockIdentifier) -> Result<Vec<Coin>> {
        self.client.coins(address, block).await
    }

    async fn faucet(&self, address: &Address, value: u128) -> Result<Vec<u8>> {
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
        let balance_transfer_tx = astar_metadata::tx().balances().transfer(dest.into(), value);
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
        self.client.metadata(public_key, options).await
    }

    async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        self.client.submit(transaction).await
    }

    async fn block(&self, block_identifier: &PartialBlockIdentifier) -> Result<Block> {
        self.client.block(block_identifier).await
    }

    async fn block_transaction(
        &self,
        block_identifier: &BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        self.client.block_transaction(block_identifier, tx).await
    }

    async fn call(&self, req: &CallRequest) -> Result<Value> {
        self.client.call(req).await
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
        let config = rosetta_config_astar::config("dev")?;
        rosetta_server::tests::network_list(config).await
    }

    #[tokio::test]
    async fn test_network_options() -> Result<()> {
        let config = rosetta_config_astar::config("dev")?;
        rosetta_server::tests::network_options::<AstarClient>(config).await
    }

    #[tokio::test]
    async fn test_network_status() -> Result<()> {
        let config = rosetta_config_astar::config("dev")?;
        rosetta_server::tests::network_status::<AstarClient>(config).await
    }

    #[tokio::test]
    async fn test_account() -> Result<()> {
        let config = rosetta_config_astar::config("dev")?;
        rosetta_server::tests::account(config).await
    }

    #[tokio::test]
    async fn test_construction() -> Result<()> {
        let config = rosetta_config_astar::config("dev")?;
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
        let config = rosetta_config_astar::config("dev")?;

        let env = Env::new("astar-smart-contract", config.clone()).await?;

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
        let config = rosetta_config_astar::config("dev")?;

        let env = Env::new("astar-smart-contract-view", config.clone()).await?;

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
        println!("{:?}", response);
        let result: Vec<String> = serde_json::from_value(response.result)?;
        assert_eq!(result[0], "true");
        env.shutdown().await?;
        Ok(())
    }
}
