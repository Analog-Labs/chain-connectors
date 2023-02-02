use std::str::FromStr;

use anyhow::{Context, Result};
use ethers::prelude::*;
use rosetta_config_ethereum::{EthereumMetadata, EthereumMetadataParams};
use rosetta_server::crypto::address::Address;
use rosetta_server::crypto::PublicKey;
use rosetta_server::types::{BlockIdentifier, Coin};
use rosetta_server::types::{
    self as rosetta_types, AccountIdentifier, Amount, Currency, Operation, OperationIdentifier,
    TransactionIdentifier,
};
use rosetta_server::types::{BlockIdentifier, CallRequest};
use rosetta_server::{BlockchainClient, BlockchainConfig};
use serde_json::{json, Value};

use crate::utils::{
    get_block_traces, GethLoggerConfig, LoadedTransaction, ResultGethExecTrace,
    ResultGethExecTraces, get_fee_operations, get_traces_operations,
};

mod utils;

pub struct EthereumClient {
    config: BlockchainConfig,
    client: Provider<Http>,
    genesis_block: BlockIdentifier,
}

#[async_trait::async_trait]
impl BlockchainClient for EthereumClient {
    type MetadataParams = EthereumMetadataParams;
    type Metadata = EthereumMetadata;

    async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_ethereum::config(network)?;
        let client = Provider::<Http>::try_from(format!("http://{}", addr))?;
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
    async fn block(&self, block_req: &rosetta_types::BlockRequest, config: &BlockchainConfig) {
        let mut transaction_vec = vec![];
        let bl_identifier = block_req.block_identifier.clone();

        let bl_id = if let Some(hash) = bl_identifier.hash {
            //convert it to H256 hash
            let h256 = H256::from_str(&hash).unwrap();
            BlockId::Hash(h256)
        } else if let Some(index) = bl_identifier.index {
            let ehters_u64 = U64::from(index);
            let bl_number = BlockNumber::Number(ehters_u64);
            BlockId::Number(bl_number)
        } else {
            return;
        };

        let block_eth = self
            .client
            .get_block_with_txs(bl_id)
            .await
            .unwrap()
            .unwrap();

        println!("block_eth: {:#?}", block_eth);
        println!("============================");

        let block_transactions = block_eth.transactions;

        let traces_block_number = match bl_id {
            BlockId::Number(n) => n,
            _ => {
                let block_number = block_eth.number.unwrap();
                let number = BlockNumber::Number(block_number);
                number
            }
        };

        let traces = get_block_traces(&block_eth.hash.unwrap(), &self.client).await;

        println!("traces {:#?}", traces);

        for (transaction, trace) in block_transactions.iter().zip(traces.0) {

            let tx_data = transaction;
            println!("tx_data: {:#?}", tx_data);

            let tx_receipt = self
                .client
                .get_transaction_receipt(tx_data.hash)
                .await
                .unwrap()
                .unwrap();
            println!("tx_receipt: {:#?}", tx_receipt);

            let (fee_amount, fee_burned) =
                estimatate_gas(&tx_data, &tx_receipt, block_eth.base_fee_per_gas);

            let loaded_tx = LoadedTransaction {
                transaction: tx_data.clone(),
                from: tx_data.from,
                block_number: block_eth.number.unwrap(),
                block_hash: block_eth.hash.unwrap(),
                fee_amount,
                fee_burned: fee_burned,
                miner: block_eth.author.unwrap(),
                receipt: tx_receipt,
            };

            let fee_operations = get_fee_operations(&loaded_tx, config);
            let traces_ops = get_traces_operations(&loaded_tx, &trace, config);
        }

        let block_index = block_eth.number.unwrap().as_u64();
        let block_hash = block_eth.hash.unwrap().to_string();
        let parent_hash = block_eth.parent_hash.to_string();
        let timestamp = block_eth.timestamp.to_string().parse::<i64>().unwrap();
        let block = rosetta_types::Block {
            block_identifier: BlockIdentifier {
                index: block_index,
                hash: block_hash,
            },
            parent_block_identifier: BlockIdentifier {
                index: block_index.saturating_sub(1),
                hash: parent_hash,
            },
            timestamp: timestamp,
            transactions: transaction_vec,
            metadata: None,
        };

        let response = rosetta_types::BlockResponse {
            block: Some(block),
            other_transactions: None,
        };
    }

    async fn block_transaction(&self, req: &rosetta_types::BlockTransactionRequest) {}

    async fn call(&self, req: &CallRequest) {}
}

fn estimatate_gas(
    tx: &Transaction,
    receipt: &TransactionReceipt,
    base_fee: Option<U256>,
) -> (U256, Option<U256>) {
    let gas_used = receipt.gas_used.unwrap();
    let gas_price = effective_gas_price(tx, base_fee);

    let fee_amount = gas_used * gas_price;

    let fee_burned = if let Some(fee) = base_fee {
        Some(gas_used * fee)
    } else {
        None
    };

    (fee_amount, fee_burned)
}

fn effective_gas_price(tx: &Transaction, base_fee: Option<U256>) -> U256 {
    if tx.transaction_type.unwrap().as_u64() != 2 {
        return tx.gas_price.unwrap();
    }

    let total_fee = base_fee.unwrap() + tx.max_priority_fee_per_gas.unwrap();
    total_fee
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
