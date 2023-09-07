use crate::{event_stream::EthereumEventStream, proof::verify_proof};
use anyhow::{bail, Context, Result};
use ethabi::token::{LenientTokenizer, Tokenizer};
use ethers::abi::{Detokenize, HumanReadableParser, InvalidOutputType, Token};
use ethers::prelude::*;
use ethers::providers::{JsonRpcClient, Middleware, Provider};
use ethers::utils::keccak256;
use ethers::utils::rlp::Encodable;
use rosetta_config_ethereum::{EthereumMetadata, EthereumMetadataParams};
use rosetta_core::{
    crypto::{address::Address, PublicKey},
    types::{
        Block, BlockIdentifier, CallRequest, Coin, PartialBlockIdentifier, Transaction,
        TransactionIdentifier,
    },
    BlockchainConfig,
};
use serde_json::{json, Value};
use std::str::FromStr;
use std::sync::Arc;

pub struct EthereumClient<P> {
    config: BlockchainConfig,
    client: Arc<Provider<P>>,
    genesis_block: BlockIdentifier,
}

impl<P> Clone for EthereumClient<P> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
            genesis_block: self.genesis_block.clone(),
        }
    }
}

impl<P> EthereumClient<P>
where
    P: JsonRpcClient,
{
    pub async fn new(config: BlockchainConfig, rpc_client: P) -> Result<Self> {
        let client = Arc::new(Provider::new(rpc_client));
        let genesis = client
            .get_block(0)
            .await?
            .context("Failed to get genesis block")?;
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

    pub fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    pub fn genesis_block(&self) -> &BlockIdentifier {
        &self.genesis_block
    }

    pub async fn node_version(&self) -> Result<String> {
        Ok(self.client.client_version().await?)
    }

    pub async fn current_block(&self) -> Result<BlockIdentifier> {
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

    pub async fn finalized_block(&self) -> Result<BlockIdentifier> {
        if let Some(block) = self
            .client
            .get_block(BlockId::Number(BlockNumber::Finalized))
            .await?
        {
            Ok(BlockIdentifier {
                index: block.number.context("Block is pending")?.as_u64(),
                hash: hex::encode(block.hash.as_ref().unwrap()),
            })
        } else {
            Ok(self.genesis_block.clone())
        }
    }

    pub async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128> {
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

    pub async fn coins(&self, _address: &Address, _block: &BlockIdentifier) -> Result<Vec<Coin>> {
        anyhow::bail!("not a utxo chain");
    }

    pub async fn faucet(&self, address: &Address, param: u128) -> Result<Vec<u8>> {
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
            .confirmations(2)
            .await?
            .unwrap()
            .transaction_hash
            .0
            .to_vec())
    }

    pub async fn metadata(
        &self,
        public_key: &PublicKey,
        options: &EthereumMetadataParams,
    ) -> Result<EthereumMetadata> {
        let from: H160 = public_key
            .to_address(self.config().address_format)
            .address()
            .parse()?;
        let to: Option<NameOrAddress> = if options.destination.len() >= 20 {
            Some(H160::from_slice(&options.destination).into())
        } else {
            None
        };
        let chain_id = self.client.get_chainid().await?;
        let nonce = self.client.get_transaction_count(from, None).await?;
        let (max_fee_per_gas, max_priority_fee_per_gas) =
            self.client.estimate_eip1559_fees(None).await?;
        let tx = Eip1559TransactionRequest {
            from: Some(from),
            to,
            value: Some(U256(options.amount)),
            data: Some(options.data.clone().into()),
            ..Default::default()
        };
        let gas_limit = self.client.estimate_gas(&tx.into(), None).await?;
        Ok(EthereumMetadata {
            chain_id: chain_id.as_u64(),
            nonce: nonce.as_u64(),
            max_priority_fee_per_gas: max_priority_fee_per_gas.0,
            max_fee_per_gas: max_fee_per_gas.0,
            gas_limit: gas_limit.0,
        })
    }

    pub async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        let tx = transaction.to_vec().into();
        Ok(self
            .client
            .send_raw_transaction(Bytes(tx))
            .await?
            .confirmations(2)
            .await?
            .context("Failed to get transaction receipt")?
            .transaction_hash
            .0
            .to_vec())
    }

    pub async fn block(&self, block_identifier: &PartialBlockIdentifier) -> Result<Block> {
        let block_id = if let Some(hash) = block_identifier.hash.as_ref() {
            BlockId::Hash(H256::from_str(hash)?)
        } else {
            let index = if let Some(index) = block_identifier.index {
                BlockNumber::Number(U64::from(index))
            } else {
                BlockNumber::Latest
            };
            BlockId::Number(index)
        };
        let block = self
            .client
            .get_block_with_txs(block_id)
            .await
            .map_err(|error| {
                anyhow::anyhow!(
                    "Failed to get block with transactions: {}",
                    error.to_string()
                )
            })?
            .context("block not found")?;
        let block_number = block.number.context("Unable to fetch block number")?;
        let block_hash = block.hash.context("Unable to fetch block hash")?;
        let mut transactions = vec![];
        let block_reward_transaction =
            crate::utils::block_reward_transaction(&self.client, self.config(), &block).await?;
        transactions.push(block_reward_transaction);
        for transaction in &block.transactions {
            let transaction =
                crate::utils::get_transaction(&self.client, self.config(), &block, transaction)
                    .await?;
            transactions.push(transaction);
        }
        Ok(Block {
            block_identifier: BlockIdentifier {
                index: block_number.as_u64(),
                hash: hex::encode(block_hash),
            },
            parent_block_identifier: BlockIdentifier {
                index: block_number.as_u64().saturating_sub(1),
                hash: hex::encode(block.parent_hash),
            },
            timestamp: block.timestamp.as_u64() as i64,
            transactions,
            metadata: None,
        })
    }

    pub async fn block_transaction(
        &self,
        block: &BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        let tx_id = H256::from_str(&tx.hash)?;
        let block = self
            .client
            .get_block(BlockId::Hash(H256::from_str(&block.hash)?))
            .await?
            .context("block not found")?;
        let transaction = self
            .client
            .get_transaction(tx_id)
            .await?
            .context("transaction not found")?;
        let transaction =
            crate::utils::get_transaction(&self.client, self.config(), &block, &transaction)
                .await?;
        Ok(transaction)
    }

    pub async fn call(&self, req: &CallRequest) -> Result<Value> {
        let call_details = req.method.split('-').collect::<Vec<&str>>();
        if call_details.len() != 3 {
            anyhow::bail!("Invalid length of call request params");
        }
        let contract_address = call_details[0];
        let method_or_position = call_details[1];
        let call_type = call_details[2];

        let block_id = req
            .block_identifier
            .as_ref()
            .map(|block_identifier| -> Result<BlockId> {
                if let Some(block_hash) = block_identifier.hash.as_ref() {
                    return BlockId::from_str(block_hash).map_err(|e| anyhow::anyhow!("{e}"));
                } else if let Some(block_number) = block_identifier.index {
                    return Ok(BlockId::Number(BlockNumber::Number(U64::from(
                        block_number,
                    ))));
                };
                bail!("invalid block identifier")
            })
            .transpose()?;

        let params = &req.parameters;
        match call_type.to_lowercase().as_str() {
            "call" => {
                //process constant call
                let contract_address = H160::from_str(contract_address)?;

                let function = HumanReadableParser::parse_function(method_or_position)?;
                let params: Vec<String> = serde_json::from_value(params.clone())?;
                let mut tokens = Vec::with_capacity(params.len());
                for (ty, arg) in function.inputs.iter().zip(params) {
                    tokens.push(LenientTokenizer::tokenize(&ty.kind, &arg)?);
                }
                let data = function.encode_input(&tokens)?;

                let tx = Eip1559TransactionRequest {
                    to: Some(contract_address.into()),
                    data: Some(data.into()),
                    ..Default::default()
                };

                let tx = &tx.into();
                let received_data = self.client.call(tx, block_id).await?;

                struct Detokenizer {
                    tokens: Vec<Token>,
                }
                impl Detokenize for Detokenizer {
                    fn from_tokens(tokens: Vec<Token>) -> Result<Self, InvalidOutputType> {
                        Ok(Self { tokens })
                    }
                }
                let detokenizer: Detokenizer =
                    decode_function_data(&function, received_data, false)?;
                let mut result = Vec::with_capacity(tokens.len());
                for token in detokenizer.tokens {
                    result.push(token.to_string());
                }
                Ok(serde_json::to_value(result)?)
            }
            "storage" => {
                //process storage call
                let from = H160::from_str(contract_address)?;

                let location = H256::from_str(method_or_position)?;

                // TODO: remove the params["block_number"], use block_identifier instead, leaving it here for compatibility
                let block_num = params["block_number"]
                    .as_u64()
                    .map(|block_num| BlockId::Number(block_num.into()))
                    .or(block_id);

                let storage_check = self
                    .client
                    .get_storage_at(from, location, block_num)
                    .await?;
                Ok(Value::String(format!("{storage_check:#?}",)))
            }
            "storage_proof" => {
                let from = H160::from_str(contract_address)?;

                let location = H256::from_str(method_or_position)?;

                // TODO: remove the params["block_number"], use block_identifier instead, leaving it here for compatibility
                let block_num = params["block_number"]
                    .as_u64()
                    .map(|block_num| BlockId::Number(block_num.into()))
                    .or(block_id);

                let proof_data = self
                    .client
                    .get_proof(from, vec![location], block_num)
                    .await?;

                //process verfiicatin of proof
                let storage_hash = proof_data.storage_hash;
                let storage_proof = proof_data.storage_proof.first().context("No proof found")?;

                let key = &storage_proof.key;
                let key_hash = keccak256(key);
                let encoded_val = storage_proof.value.rlp_bytes().to_vec();

                let is_valid = verify_proof(
                    &storage_proof.proof,
                    storage_hash.as_bytes(),
                    &key_hash.to_vec(),
                    &encoded_val,
                );

                let result = serde_json::to_value(&proof_data)?;

                Ok(json!({
                    "proof": result,
                    "isValid": is_valid
                }))
            }
            "transaction_receipt" => {
                let tx_hash = H256::from_str(contract_address)?;
                let receipt = self.client.get_transaction_receipt(tx_hash).await?;
                let result = serde_json::to_value(&receipt)?;
                if block_id.is_some() {
                    bail!("block identifier is ignored for transaction receipt");
                }
                Ok(result)
            }
            _ => {
                bail!("request type not supported")
            }
        }
    }
}

impl<P> EthereumClient<P>
where
    P: PubsubClient,
{
    pub async fn listen(&self) -> Result<EthereumEventStream<'_, P>> {
        let new_head_subscription = self.client.subscribe_blocks().await?;
        Ok(EthereumEventStream::new(new_head_subscription))
    }
}
