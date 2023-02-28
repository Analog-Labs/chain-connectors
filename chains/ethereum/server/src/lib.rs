use crate::eth_types::GENESIS_BLOCK_INDEX;
use crate::utils::{get_block, get_transaction, populate_transactions, EthDetokenizer};
use anyhow::{anyhow, bail, Context, Result};
use ethers::abi::Abi;
use ethers::contract as ethers_contract;
use ethers::prelude::*;
use rosetta_config_ethereum::{EthereumMetadata, EthereumMetadataParams};
use rosetta_server::crypto::address::Address;
use rosetta_server::crypto::PublicKey;
use rosetta_server::types::{
    Block, BlockIdentifier, CallRequest, Coin, PartialBlockIdentifier, Transaction,
    TransactionIdentifier,
};
use rosetta_server::{BlockchainClient, BlockchainConfig};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;

mod eth_types;
mod utils;

pub struct EthereumClient {
    config: BlockchainConfig,
    client: Arc<Provider<Http>>,
    genesis_block: BlockIdentifier,
}

#[async_trait::async_trait]
impl BlockchainClient for EthereumClient {
    type MetadataParams = EthereumMetadataParams;
    type Metadata = EthereumMetadata;

    async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_ethereum::config(network)?;
        let client = Arc::new(Provider::<Http>::try_from(format!("http://{addr}"))?);
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
        let address: H160 = address.address().parse()?;
        Ok(self
            .client
            .get_balance(address, Some(BlockId::Hash(H256(block))))
            .await?
            .as_u128())
    }

    async fn coins(&self, _address: &Address, _block: &BlockIdentifier) -> Result<Vec<Coin>> {
        anyhow::bail!("not a utxo chain");
    }

    async fn faucet(&self, address: &Address, param: u128) -> Result<Vec<u8>> {
        // first account will be the coinbase account on a dev net
        let coinbase = self.client.get_accounts().await?[0];
        let address: H160 = address.address().parse()?;
        let tx = TransactionRequest::new()
            .to(address)
            .value(param)
            .from(coinbase);
        Ok(self
            .client
            .send_transaction(tx, None)
            .await?
            .await?
            .unwrap()
            .transaction_hash
            .0
            .to_vec())
    }

    async fn metadata(
        &self,
        public_key: &PublicKey,
        options: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        let from: H160 = public_key
            .to_address(self.config().address_format)
            .address()
            .parse()
            .unwrap();
        let to = H160::from_slice(&options.destination);
        let chain_id = self.client.get_chainid().await?;
        let nonce = self.client.get_transaction_count(from, None).await?;
        let (max_fee_per_gas, max_priority_fee_per_gas) =
            self.client.estimate_eip1559_fees(None).await?;
        let tx = Eip1559TransactionRequest::new()
            .from(from)
            .to(to)
            .value(U256(options.amount))
            .data(options.data.clone());
        let gas_limit = self.client.estimate_gas(&tx.into(), None).await?;
        Ok(EthereumMetadata {
            chain_id: chain_id.as_u64(),
            nonce: nonce.as_u64(),
            max_priority_fee_per_gas: max_priority_fee_per_gas.0,
            max_fee_per_gas: max_fee_per_gas.0,
            gas_limit: gas_limit.0,
        })
    }

    async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        let tx = transaction.to_vec().into();
        Ok(self
            .client
            .send_raw_transaction(Bytes(tx))
            .await?
            .await?
            .unwrap()
            .transaction_hash
            .0
            .to_vec())
    }

    async fn block(&self, block: &PartialBlockIdentifier) -> Result<Block> {
        let (block, loaded_tx, uncles) = get_block(block, &self.client).await?;

        let block_number = block.number.context("Unable to fetch block number")?;
        let block_hash = block.hash.context("Unable to fetch block hash")?;
        let block_identifier = BlockIdentifier {
            index: block_number.as_u64(),
            hash: hex::encode(block_hash),
        };

        let mut parent_identifier = block_identifier.clone();
        if block_identifier.index != GENESIS_BLOCK_INDEX {
            parent_identifier.index -= 1;
            parent_identifier.hash = hex::encode(block.parent_hash);
        }

        let transactions = populate_transactions(
            &block_identifier,
            &block,
            uncles,
            loaded_tx,
            &self.config.currency(),
        )
        .await?;

        Ok(Block {
            block_identifier,
            parent_block_identifier: parent_identifier,
            timestamp: block.timestamp.as_u64() as i64,
            transactions,
            metadata: None,
        })
    }

    async fn block_transaction(
        &self,
        block: &BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        let transaction =
            get_transaction(block, &tx.hash, &self.client, &self.config.currency()).await?;

        Ok(transaction)
    }

    async fn call(&self, req: &CallRequest) -> Result<Value> {
        let method = req.method.clone();
        let params = req.parameters.clone();

        let call_type = params["type"].as_str().context("type not found")?;

        match call_type.to_lowercase().as_str() {
            "call" => {
                //process constant call
                let abi_str = params["abi"].as_str().context("ABI not found")?;

                let abi: Abi = serde_json::from_str(abi_str).map_err(|err| anyhow!(err))?;

                let contract_address = H160::from_str(
                    params["contract_address"]
                        .as_str()
                        .context("contact address not found")?,
                )
                .map_err(|err| anyhow!(err))?;

                let contract =
                    ethers_contract::Contract::new(contract_address, abi, self.client.clone());

                let value: EthDetokenizer = contract
                    .method(&method, ())
                    .map_err(|err| anyhow!(err))?
                    .call()
                    .await
                    .map_err(|err| anyhow!(err))?;

                let result: Value = serde_json::from_str(&value.json)?;
                return Ok(result);
            }
            "storage" => {
                //process storage call
                let from = H160::from_str(
                    params["address"]
                        .as_str()
                        .context("address field not found")?,
                )
                .map_err(|err| anyhow!(err))?;

                let location =
                    H256::from_str(params["position"].as_str().context("position not found")?)
                        .map_err(|err| anyhow!(err))?;

                let block_num = params["block_number"]
                    .as_u64()
                    .map(|block_num| BlockId::Number(block_num.into()));

                let storage_check = self
                    .client
                    .get_storage_at(from, location, block_num)
                    .await?;
                return Ok(Value::String(format!("{storage_check:#?}",)));
            }
            _ => {
                bail!("request type not supported")
            }
        }
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
