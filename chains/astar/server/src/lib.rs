use anyhow::Result;
use ethers::prelude::*;
use rosetta_config_ethereum::{EthereumMetadata, EthereumMetadataParams};
use rosetta_server::crypto::address::Address;
use rosetta_server::crypto::PublicKey;
use rosetta_server::types::{
    Block, BlockIdentifier, CallRequest, Coin, PartialBlockIdentifier, Transaction,
    TransactionIdentifier,
};
use rosetta_server::{BlockchainClient, BlockchainConfig};
use rosetta_server_ethereum::EthereumClient;
use serde_json::Value;

pub struct AstarClient {
    client: EthereumClient,
    addr: String,
}

#[async_trait::async_trait]
impl BlockchainClient for AstarClient {
    type MetadataParams = EthereumMetadataParams;
    type Metadata = EthereumMetadata;

    fn create_config(network: &str) -> Result<BlockchainConfig> {
        rosetta_config_astar::config(network)
    }

    async fn new(config: BlockchainConfig, addr: &str) -> Result<Self> {
        let client = EthereumClient::new(config, addr).await?;
        Ok(Self {
            client,
            addr: addr.into(),
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

    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128> {
        self.client.balance(address, block).await
    }

    async fn coins(&self, address: &Address, block: &BlockIdentifier) -> Result<Vec<Coin>> {
        self.client.coins(address, block).await
    }

    async fn faucet(&self, address: &Address, value: u128) -> Result<Vec<u8>> {
        use parity_scale_codec::{Decode, Encode};
        use sp_keyring::AccountKeyring;
        use subxt::tx::{PairSigner, StaticTxPayload};
        use subxt::utils::{AccountId32, MultiAddress};
        use subxt::{OnlineClient, PolkadotConfig};

        #[derive(Decode, Encode, Debug)]
        pub struct Transfer {
            pub dest: MultiAddress<AccountId32, u32>,
            #[codec(compact)]
            pub value: u128,
        }

        let addr = &self.addr;
        let client = OnlineClient::<PolkadotConfig>::from_url(format!("ws://{addr}")).await?;

        // convert address
        let address: H160 = address.address().parse()?;
        let mut data = [0u8; 24];
        data[0..4].copy_from_slice(b"evm:");
        data[4..24].copy_from_slice(&address[..]);
        let hash = sp_core::hashing::blake2_256(&data);
        let address = AccountId32::from(Into::<[u8; 32]>::into(hash));
        //

        let signer = PairSigner::<PolkadotConfig, _>::new(AccountKeyring::Alice.pair());
        let dest: MultiAddress<AccountId32, u32> = MultiAddress::Id(address);
        let tx = StaticTxPayload::new("Balances", "transfer", Transfer { dest, value }, [0; 32])
            .unvalidated();
        let hash = client
            .tx()
            .sign_and_submit_then_watch_default(&tx, &signer)
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
        block: &BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        self.client.block_transaction(block, tx).await
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

    #[tokio::test]
    async fn test_find_transaction() -> Result<()> {
        let config = rosetta_config_astar::config("dev")?;
        rosetta_server::tests::find_transaction(config).await
    }

    #[tokio::test]
    async fn test_list_transactions() -> Result<()> {
        let config = rosetta_config_astar::config("dev")?;
        rosetta_server::tests::list_transactions(config).await
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

        let env = Env::new("smart-contract", config.clone()).await?;

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
        Ok(())
    }

    #[tokio::test]
    async fn test_smart_contract_view() -> Result<()> {
        let config = rosetta_config_astar::config("dev")?;

        let env = Env::new("smart-contract-view", config.clone()).await?;

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
            )
            .await?;
        println!("{:?}", response);
        let result: Vec<String> = serde_json::from_value(response.result)?;
        assert_eq!(result[0], "true");

        Ok(())
    }
}
