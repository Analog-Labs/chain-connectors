use crate::{
    event_stream::EthereumEventStream,
    proof::verify_proof,
    utils::{get_non_pending_block, NonPendingBlock},
};
use anyhow::{Context, Result};
use ethers::{
    prelude::*,
    providers::{JsonRpcClient, Middleware, Provider},
    types::{transaction::eip2718::TypedTransaction, Bytes},
    utils::{keccak256, rlp::Encodable},
};
use rosetta_config_ethereum::{
    AtBlock, CallContract, CallResult, EIP1186ProofResponse, EthereumMetadata,
    EthereumMetadataParams, GetBalance, GetProof, GetStorageAt, GetTransactionReceipt, Log,
    Query as EthQuery, QueryResult as EthQueryResult, StorageProof, TransactionReceipt,
};
use rosetta_core::{
    crypto::{address::Address, PublicKey},
    types::{
        Block, BlockIdentifier, Coin, PartialBlockIdentifier, Transaction, TransactionIdentifier,
    },
    BlockchainConfig,
};
use std::{
    str::FromStr,
    sync::{
        atomic::{self, Ordering},
        Arc,
    },
};

/// Strategy used to determine the finalized block
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockFinalityStrategy {
    /// Query the finalized block using `eth_getBlockByNumber("finalized")` json-rpc method
    #[default]
    Finalized,
    /// Use the number of confirmations to determine the finalized block
    Confirmations(u64),
}

impl BlockFinalityStrategy {
    pub fn from_config(config: &BlockchainConfig) -> Self {
        match (config.blockchain, config.testnet) {
            // TODO: ISSUE-176 Replace this hack by querying polygon checkpoints
            // Polygon finalized blocks are stored on ethereum mainnet roughly every 30 minutes
            // and polygon block interval is ~2 seconds, 30 minutes / 2 seconds == 900 blocks.
            ("polygon", false) => Self::Confirmations(900),
            ("polygon", true) => Self::Confirmations(6), // For local testnet use 6 confirmations
            _ => Self::Finalized,
        }
    }
}

pub struct EthereumClient<P> {
    config: BlockchainConfig,
    client: Arc<Provider<P>>,
    genesis_block: NonPendingBlock,
    block_finality_strategy: BlockFinalityStrategy,
    nonce: Arc<std::sync::atomic::AtomicU32>,
    private_key: Option<[u8; 32]>,
}

impl<P> Clone for EthereumClient<P> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
            genesis_block: self.genesis_block.clone(),
            block_finality_strategy: self.block_finality_strategy,
            private_key: self.private_key,
            nonce: self.nonce.clone(),
        }
    }
}

impl<P> EthereumClient<P>
where
    P: JsonRpcClient + 'static,
{
    #[allow(clippy::missing_errors_doc)]
    pub async fn new(
        config: BlockchainConfig,
        rpc_client: P,
        private_key: Option<[u8; 32]>,
    ) -> Result<Self> {
        let block_finality_strategy = BlockFinalityStrategy::from_config(&config);
        let client = Arc::new(Provider::new(rpc_client));
        //store
        // private key
        // nonce
        // every transaction will incress the nonce
        let wallet = LocalWallet::from_bytes(&private_key.unwrap());
        let nonce = Arc::new(atomic::AtomicU32::from(
            client.get_transaction_count(wallet.unwrap().address(), None).await?.as_u32(),
        ));

        let Some(genesis_block) =
            get_non_pending_block(Arc::clone(&client), BlockNumber::Number(0.into())).await?
        else {
            anyhow::bail!("FATAL: genesis block not found");
        };
        Ok(Self { config, client, genesis_block, block_finality_strategy, private_key, nonce })
    }

    pub const fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    pub const fn genesis_block(&self) -> &BlockIdentifier {
        &self.genesis_block.identifier
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn node_version(&self) -> Result<String> {
        Ok(self.client.client_version().await?)
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn current_block(&self) -> Result<BlockIdentifier> {
        let index = self.client.get_block_number().await?.as_u64();
        let Some(block_hash) = self.client.get_block(index).await?.context("missing block")?.hash
        else {
            anyhow::bail!("FATAL: block hash is missing");
        };
        Ok(BlockIdentifier { index, hash: hex::encode(block_hash) })
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn finalized_block(&self, latest_block: Option<u64>) -> Result<NonPendingBlock> {
        let number = match self.block_finality_strategy {
            BlockFinalityStrategy::Confirmations(confirmations) => {
                let latest_block = match latest_block {
                    Some(number) => number,
                    None => self
                        .client
                        .get_block_number()
                        .await
                        .context("Failed to retrieve latest block number")?
                        .as_u64(),
                };
                let block_number = latest_block.saturating_sub(confirmations);
                // If the number is zero, the latest finalized is the genesis block
                if block_number == 0 {
                    return Ok(self.genesis_block.clone());
                }
                BlockNumber::Number(U64::from(block_number))
            },
            BlockFinalityStrategy::Finalized => BlockNumber::Finalized,
        };

        let Some(finalized_block) = get_non_pending_block(Arc::clone(&self.client), number).await?
        else {
            anyhow::bail!("Cannot find finalized block at {number}");
        };
        Ok(finalized_block)
    }

    #[allow(clippy::missing_errors_doc)]
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

    #[allow(clippy::unused_async, clippy::missing_errors_doc)]
    pub async fn coins(&self, _address: &Address, _block: &BlockIdentifier) -> Result<Vec<Coin>> {
        anyhow::bail!("not a utxo chain");
    }

    #[allow(clippy::single_match_else, clippy::missing_errors_doc)]
    pub async fn faucet(&self, address: &Address, param: u128) -> Result<Vec<u8>> {
        match self.private_key {
            Some(private_key) => {
                let chain_id = self.client.get_chainid().await?.as_u64();
                let address: H160 = address.address().parse()?;
                // let nonce = self
                //     .client
                //     .get_transaction_count(
                //         ethers::types::NameOrAddress::Address(H160::from_str(
                //             "0x3f1eae7d46d88f08fc2f8ed27fcb2ab183eb2d0e",
                //         )?),
                //         None,
                //     )
                //     .await?; //public key of faucet account
                let wallet = LocalWallet::from_bytes(&private_key)?;
                let nonce_u32 = U256::from(self.nonce.load(Ordering::Relaxed));
                // Create a transaction request
                let transaction_request = TransactionRequest {
                    from: None,
                    to: Some(ethers::types::NameOrAddress::Address(address)),
                    value: Some(U256::from(param)), // Specify the amount you want to send
                    gas: Some(U256::from(210_000)), // Adjust gas values accordingly
                    gas_price: Some(U256::from(500_000_000)), // Adjust gas price accordingly
                    nonce: Some(nonce_u32),         // Nonce will be automatically determined
                    data: None,
                    chain_id: Some(chain_id.into()),
                };

                let tx: TypedTransaction = transaction_request.into();
                let signature = wallet.sign_transaction(&tx).await?;
                let tx = tx.rlp_signed(&signature);
                let response = self
                    .client
                    .send_raw_transaction(tx)
                    .await?
                    .confirmations(2)
                    .await?
                    .context("failed to retrieve tx receipt")?
                    .transaction_hash
                    .0
                    .to_vec();
                Ok(response)
            },
            None => {
                // first account will be the coinbase account on a dev net
                let coinbase = self.client.get_accounts().await?[0];
                let address: H160 = address.address().parse()?;
                let tx = TransactionRequest::new().to(address).value(param).from(coinbase);
                Ok(self
                    .client
                    .send_transaction(tx, None)
                    .await?
                    .confirmations(2)
                    .await?
                    .context("failed to retrieve tx receipt")?
                    .transaction_hash
                    .0
                    .to_vec())
            },
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn metadata(
        &self,
        public_key: &PublicKey,
        options: &EthereumMetadataParams,
    ) -> Result<EthereumMetadata> {
        let from: H160 = public_key.to_address(self.config().address_format).address().parse()?;
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

    #[allow(clippy::missing_errors_doc)]
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

    #[allow(clippy::missing_errors_doc)]
    pub async fn block(&self, block_identifier: &PartialBlockIdentifier) -> Result<Block> {
        let block_id = if let Some(hash) = block_identifier.hash.as_ref() {
            BlockId::Hash(H256::from_str(hash)?)
        } else {
            let index = block_identifier
                .index
                .map_or(BlockNumber::Latest, |index| BlockNumber::Number(U64::from(index)));
            BlockId::Number(index)
        };
        let block = self
            .client
            .get_block_with_txs(block_id)
            .await
            .map_err(|error| {
                anyhow::anyhow!("Failed to get block with transactions: {}", error.to_string())
            })?
            .context("block not found")?;
        let block_number = block.number.context("Unable to fetch block number")?;
        let block_hash = block.hash.context("Unable to fetch block hash")?;
        let mut transactions = vec![];
        let block_reward_transaction =
            crate::utils::block_reward_transaction(&self.client, self.config(), &block).await?;
        transactions.push(block_reward_transaction);
        Ok(Block {
            block_identifier: BlockIdentifier {
                index: block_number.as_u64(),
                hash: hex::encode(block_hash),
            },
            parent_block_identifier: BlockIdentifier {
                index: block_number.as_u64().saturating_sub(1),
                hash: hex::encode(block.parent_hash),
            },
            timestamp: i64::try_from(block.timestamp.as_u64()).context("timestamp overflow")?,
            transactions,
            metadata: None,
        })
    }

    #[allow(clippy::missing_errors_doc)]
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
        let transaction =
            self.client.get_transaction(tx_id).await?.context("transaction not found")?;
        let transaction =
            crate::utils::get_transaction(&self.client, self.config(), block, &transaction).await?;
        Ok(transaction)
    }

    #[allow(clippy::too_many_lines, clippy::missing_errors_doc)]
    pub async fn call(&self, req: &EthQuery) -> Result<EthQueryResult> {
        let result = match req {
            EthQuery::GetBalance(GetBalance { address, block }) => {
                let block_id = match *block {
                    AtBlock::Latest => BlockId::Number(BlockNumber::Latest),
                    AtBlock::Number(number) => BlockId::Number(number.into()),
                    AtBlock::Hash(hash) => BlockId::Hash(hash),
                };
                let balance = self.client.get_balance(*address, Some(block_id)).await?;
                EthQueryResult::GetBalance(balance)
            },
            EthQuery::GetStorageAt(GetStorageAt { address, at, block }) => {
                let block_id = match *block {
                    AtBlock::Latest => BlockId::Number(BlockNumber::Latest),
                    AtBlock::Number(number) => BlockId::Number(BlockNumber::Number(number.into())),
                    AtBlock::Hash(hash) => BlockId::Hash(hash),
                };
                let value = self.client.get_storage_at(*address, *at, Some(block_id)).await?;
                EthQueryResult::GetStorageAt(value)
            },
            EthQuery::GetTransactionReceipt(GetTransactionReceipt { tx_hash }) => {
                let receipt = self.client.get_transaction_receipt(*tx_hash).await?.map(|receipt| {
                    TransactionReceipt {
                        transaction_hash: receipt.transaction_hash,
                        transaction_index: receipt.transaction_index.as_u64(),
                        block_hash: receipt.block_hash,
                        block_number: receipt.block_number.map(|number| number.as_u64()),
                        from: receipt.from,
                        to: receipt.to,
                        cumulative_gas_used: receipt.cumulative_gas_used,
                        gas_used: receipt.gas_used,
                        contract_address: receipt.contract_address,
                        status_code: receipt.status.map(|number| number.as_u64()),
                        state_root: receipt.root,
                        logs: receipt
                            .logs
                            .into_iter()
                            .map(|log| Log {
                                address: log.address,
                                topics: log.topics,
                                data: log.data.to_vec(),
                                block_hash: log.block_hash,
                                block_number: log.block_number.map(|n| n.as_u64()),
                                transaction_hash: log.transaction_hash,
                                transaction_index: log.transaction_index.map(|n| n.as_u64()),
                                log_index: log.log_index,
                                transaction_log_index: log.transaction_log_index,
                                log_type: log.log_type,
                                removed: log.removed,
                            })
                            .collect(),
                        logs_bloom: receipt.logs_bloom,
                        effective_gas_price: receipt.effective_gas_price,
                        transaction_type: receipt.transaction_type.map(|number| number.as_u64()),
                    }
                });
                EthQueryResult::GetTransactionReceipt(receipt)
            },
            EthQuery::CallContract(CallContract { from, to, data, value, block }) => {
                let block_id = match *block {
                    AtBlock::Latest => BlockId::Number(BlockNumber::Latest),
                    AtBlock::Number(number) => BlockId::Number(BlockNumber::Number(number.into())),
                    AtBlock::Hash(hash) => BlockId::Hash(hash),
                };
                let tx = Eip1559TransactionRequest {
                    from: *from,
                    to: Some((*to).into()),
                    data: Some(data.clone().into()),
                    value: Some(*value),
                    ..Default::default()
                };
                let tx = &tx.into();
                let received_data = self.client.call(tx, Some(block_id)).await?;
                EthQueryResult::CallContract(CallResult::Success(received_data.to_vec()))
            },
            EthQuery::GetProof(GetProof { account, storage_keys, block }) => {
                let block_id = match *block {
                    AtBlock::Latest => BlockId::Number(BlockNumber::Latest),
                    AtBlock::Number(number) => BlockId::Number(BlockNumber::Number(number.into())),
                    AtBlock::Hash(hash) => BlockId::Hash(hash),
                };
                let proof_data =
                    self.client.get_proof(*account, storage_keys.clone(), Some(block_id)).await?;

                //process verfiicatin of proof
                let storage_hash = proof_data.storage_hash;
                let storage_proof = proof_data.storage_proof.first().context("No proof found")?;

                let key = &storage_proof.key;
                let key_hash = keccak256(key);
                let encoded_val = storage_proof.value.rlp_bytes().to_vec();

                let _is_valid = verify_proof(
                    &storage_proof.proof,
                    storage_hash.as_bytes(),
                    &key_hash.to_vec(),
                    &encoded_val,
                );
                EthQueryResult::GetProof(EIP1186ProofResponse {
                    address: proof_data.address,
                    balance: proof_data.balance,
                    code_hash: proof_data.code_hash,
                    nonce: proof_data.nonce.as_u64(),
                    storage_hash: proof_data.storage_hash,
                    account_proof: proof_data
                        .account_proof
                        .into_iter()
                        .map(|bytes| bytes.0.to_vec())
                        .collect(),
                    storage_proof: proof_data
                        .storage_proof
                        .into_iter()
                        .map(|storage_proof| StorageProof {
                            key: storage_proof.key,
                            proof: storage_proof
                                .proof
                                .into_iter()
                                .map(|proof| proof.0.to_vec())
                                .collect(),
                            value: storage_proof.value,
                        })
                        .collect(),
                })
            },
        };
        Ok(result)
    }
}

impl<P> EthereumClient<P>
where
    P: PubsubClient + 'static,
{
    #[allow(clippy::missing_errors_doc)]
    pub async fn listen(&self) -> Result<EthereumEventStream<'_, P>> {
        let new_head_subscription = self.client.subscribe_blocks().await?;
        Ok(EthereumEventStream::new(self, new_head_subscription))
    }
}
