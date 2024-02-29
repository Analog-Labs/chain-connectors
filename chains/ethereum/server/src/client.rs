#![allow(clippy::option_if_let_else)]
use crate::{
    event_stream::EthereumEventStream,
    log_filter::LogFilter,
    proof::verify_proof,
    utils::{AtBlockExt, EthereumRpcExt, PartialBlock},
};
use anyhow::{Context, Result};
use ethers::{
    prelude::*,
    types::transaction::eip2718::TypedTransaction,
    utils::{keccak256, rlp::Encodable},
};
use rosetta_config_ethereum::{
    ext::types::{rpc::CallRequest, AccessList, AtBlock},
    CallContract, CallResult, EthereumMetadata, EthereumMetadataParams, GetBalance, GetProof,
    GetStorageAt, GetTransactionReceipt, Query as EthQuery, QueryResult as EthQueryResult,
    Subscription,
};
use rosetta_core::{
    crypto::{address::Address, PublicKey},
    types::{BlockIdentifier, PartialBlockIdentifier},
    BlockchainConfig,
};
use rosetta_ethereum_backend::{
    jsonrpsee::{
        core::client::{ClientT, SubscriptionClientT},
        Adapter,
    },
    BlockRange, EthereumPubSub, EthereumRpc, ExitReason,
};
use std::sync::{
    atomic::{self, Ordering},
    Arc,
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
    pub backend: Adapter<P>,
    genesis_block: PartialBlock,
    block_finality_strategy: BlockFinalityStrategy,
    nonce: Arc<std::sync::atomic::AtomicU64>,
    private_key: Option<[u8; 32]>,
    log_filter: Arc<std::sync::Mutex<LogFilter>>,
}

impl<P> Clone for EthereumClient<P>
where
    P: Clone,
{
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            backend: self.backend.clone(),
            genesis_block: self.genesis_block.clone(),
            block_finality_strategy: self.block_finality_strategy,
            nonce: self.nonce.clone(),
            private_key: self.private_key,
            log_filter: self.log_filter.clone(),
        }
    }
}

impl<P> EthereumClient<P>
where
    P: ClientT + Clone + Send + Sync + 'static,
{
    #[allow(clippy::missing_errors_doc)]
    pub async fn new(
        config: BlockchainConfig,
        rpc_client: P,
        private_key: Option<[u8; 32]>,
    ) -> Result<Self> {
        let backend = Adapter(rpc_client.clone());
        let at = AtBlock::At(rosetta_config_ethereum::ext::types::BlockIdentifier::Number(0));
        let genesis_block = backend
            .block(at)
            .await?
            .ok_or_else(|| anyhow::format_err!("FATAL: genesis block not found"))?
            .try_seal()
            .map_err(|_| {
                anyhow::format_err!(
                    "FATAL: api returned an invalid genesis block: block hash missing"
                )
            })?;

        let block_finality_strategy = BlockFinalityStrategy::from_config(&config);
        let (private_key, nonce) = if let Some(private) = private_key {
            let wallet = LocalWallet::from_bytes(&private)?;
            let address = wallet.address();
            let nonce = Arc::new(atomic::AtomicU64::from(
                backend.get_transaction_count(address, AtBlock::Latest).await?,
            ));
            (private_key, nonce)
        } else {
            (None, Arc::new(atomic::AtomicU64::new(0)))
        };
        Ok(Self {
            config,
            backend,
            genesis_block,
            block_finality_strategy,
            nonce,
            private_key,
            log_filter: Arc::new(std::sync::Mutex::new(LogFilter::new())),
        })
    }
}

impl<P> EthereumClient<P>
where
    P: ClientT + Send + Sync + 'static,
{
    pub const fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    pub const fn genesis_block(&self) -> BlockIdentifier {
        BlockIdentifier {
            index: self.genesis_block.header().header().number,
            hash: self.genesis_block.header().hash().0,
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn current_block(&self) -> Result<BlockIdentifier> {
        let Some(block) = self.backend.block(AtBlock::Latest).await? else {
            anyhow::bail!("[report this bug] latest block not found");
        };
        let Some(hash) = block.hash else {
            anyhow::bail!("[report this bug] api returned latest block without hash");
        };
        Ok(BlockIdentifier { index: block.header.number, hash: hash.0 })
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn finalized_block(&self, latest_block: Option<u64>) -> Result<PartialBlock> {
        let number: AtBlock = match self.block_finality_strategy {
            BlockFinalityStrategy::Confirmations(confirmations) => {
                let latest_block = match latest_block {
                    Some(number) => number,
                    None => self
                        .backend
                        .block_number()
                        .await
                        .context("Failed to retrieve latest block number")?,
                };
                let block_number = latest_block.saturating_sub(confirmations);
                // If the number is zero, the latest finalized is the genesis block
                if block_number == 0 {
                    return Ok(self.genesis_block.clone());
                }
                AtBlock::At(block_number.into())
            },
            BlockFinalityStrategy::Finalized => AtBlock::Finalized,
        };

        let Some(finalized_block) = self.backend.block(number).await? else {
            anyhow::bail!("Cannot find finalized block at {number}");
        };
        let finalized_block = finalized_block.try_seal().map_err(|_| {
            anyhow::format_err!("api returned an invalid finalized block: block hash missing")
        })?;
        Ok(finalized_block)
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn balance(
        &self,
        address: &Address,
        block_identifier: &PartialBlockIdentifier,
    ) -> Result<u128> {
        // Convert `PartialBlockIdentifier` to `AtBlock`
        let at_block = AtBlock::from_partial_identifier(block_identifier);
        let address: H160 = address.address().parse()?;
        let balance = self.backend.get_balance(address, at_block).await?;
        let balance = u128::try_from(balance)
            .map_err(|err| anyhow::format_err!("balance overflow: {err}"))?;
        Ok(balance)
    }

    #[allow(clippy::single_match_else, clippy::missing_errors_doc)]
    pub async fn faucet(&self, address: &Address, param: u128) -> Result<Vec<u8>> {
        match self.private_key {
            Some(private_key) => {
                let chain_id = self.backend.chain_id().await?;
                let address: H160 = address.address().parse()?;
                let wallet = LocalWallet::from_bytes(&private_key)?;
                let nonce_u32 = U256::from(self.nonce.load(Ordering::Relaxed));
                // Create a transaction request
                let transaction_request = TransactionRequest {
                    from: None,
                    to: Some(ethers::types::NameOrAddress::Address(address)),
                    value: Some(U256::from(param)),
                    gas: Some(U256::from(210_000)),
                    gas_price: Some(U256::from(500_000_000)),
                    nonce: Some(nonce_u32),
                    data: None,
                    chain_id: Some(chain_id.into()),
                };

                let tx: TypedTransaction = transaction_request.into();
                let signature = wallet.sign_transaction(&tx).await?;
                let tx = tx.rlp_signed(&signature);
                let tx_hash = self.backend.send_raw_transaction(tx.0.into()).await?;

                // Wait for the transaction to be mined
                let receipt = self.backend.wait_for_transaction_receipt(tx_hash).await?;

                // Check if the transaction was successful
                if !matches!(receipt.status_code, Some(1)) {
                    anyhow::bail!("Transaction reverted: {tx_hash}");
                }
                Ok(tx_hash.0.to_vec())
            },
            None => {
                // first account will be the coinbase account on a dev net
                let coinbase = self
                    .backend
                    .get_accounts()
                    .await?
                    .into_iter()
                    .next()
                    .context("no accounts found")?;
                let address: H160 = address.address().parse()?;

                let (max_fee_per_gas, max_priority_fee_per_gas) =
                    self.backend.estimate_eip1559_fees().await?;
                let tx = CallRequest {
                    from: Some(coinbase),
                    to: Some(address),
                    gas_limit: None,
                    gas_price: None,
                    value: Some(U256::from(param)),
                    data: None,
                    nonce: None,
                    chain_id: None,
                    max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
                    access_list: AccessList::default(),
                    max_fee_per_gas: Some(max_fee_per_gas),
                    transaction_type: None,
                };

                let tx_hash = self.backend.send_transaction(&tx).await?;
                let receipt = self.backend.wait_for_transaction_receipt(tx_hash).await?;
                if !matches!(receipt.status_code, Some(1)) {
                    anyhow::bail!("Transaction reverted: {tx_hash}");
                }
                Ok(tx_hash.0.to_vec())
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
        let to: Option<H160> = if options.destination.len() >= 20 {
            Some(H160::from_slice(&options.destination))
        } else {
            None
        };
        let (max_fee_per_gas, max_priority_fee_per_gas) =
            self.backend.estimate_eip1559_fees().await?;
        let chain_id = self.backend.chain_id().await?;
        let nonce = self.backend.get_transaction_count(from, AtBlock::Latest).await?;
        let tx = CallRequest {
            from: Some(from),
            to,
            gas_limit: None,
            gas_price: None,
            value: Some(U256(options.amount)),
            data: Some(options.data.clone().into()),
            nonce: Some(nonce),
            chain_id: None, // Astar doesn't support this field
            max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
            access_list: AccessList::default(),
            max_fee_per_gas: Some(max_fee_per_gas),
            transaction_type: Some(2),
        };
        let gas_limit = self.backend.estimate_gas(&tx, AtBlock::Latest).await?;

        Ok(EthereumMetadata {
            chain_id,
            nonce,
            max_priority_fee_per_gas: max_priority_fee_per_gas.0,
            max_fee_per_gas: max_fee_per_gas.0,
            gas_limit: gas_limit.0,
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        let tx = rosetta_ethereum_backend::ext::types::Bytes::from_iter(transaction);
        let tx_hash = self.backend.send_raw_transaction(tx).await?;

        // Wait for the transaction to be mined
        let receipt = self.backend.wait_for_transaction_receipt(tx_hash).await?;

        if !matches!(receipt.status_code, Some(1)) {
            anyhow::bail!("Transaction reverted: {tx_hash}");
        }
        Ok(tx_hash.0.to_vec())
    }

    #[allow(clippy::too_many_lines, clippy::missing_errors_doc)]
    pub async fn call(&self, req: &EthQuery) -> Result<EthQueryResult> {
        let result = match req {
            EthQuery::GetBalance(GetBalance { address, block }) => {
                let balance = self.backend.get_balance(*address, *block).await?;
                EthQueryResult::GetBalance(balance)
            },
            EthQuery::GetStorageAt(GetStorageAt { address, at, block }) => {
                let value = self.backend.storage(*address, *at, *block).await?;
                EthQueryResult::GetStorageAt(value)
            },
            EthQuery::GetTransactionReceipt(GetTransactionReceipt { tx_hash }) => {
                let receipt = self.backend.transaction_receipt(*tx_hash).await?;
                EthQueryResult::GetTransactionReceipt(receipt)
            },
            EthQuery::CallContract(CallContract { from, to, data, value, block }) => {
                use rosetta_config_ethereum::ext::types::Bytes;
                let call = CallRequest {
                    from: *from,
                    to: Some(*to),
                    data: Some(Bytes::from_iter(data)),
                    value: Some(*value),
                    gas_limit: None, // TODO: the default gas limit changes from client to client
                    gas_price: None,
                    nonce: None,
                    chain_id: None,
                    max_priority_fee_per_gas: None,
                    access_list: AccessList::default(),
                    max_fee_per_gas: None,
                    transaction_type: None,
                };
                let result = match self.backend.call(&call, *block).await? {
                    ExitReason::Succeed(data) => CallResult::Success(data.to_vec()),
                    ExitReason::Revert(data) => CallResult::Revert(data.to_vec()),
                    ExitReason::Error(_) => CallResult::Error,
                };
                EthQueryResult::CallContract(result)
            },
            EthQuery::GetProof(GetProof { account, storage_keys, block }) => {
                let proof_data = self.backend.get_proof(*account, storage_keys, *block).await?;

                //process verfiicatin of proof
                let storage_hash = proof_data.storage_hash;
                let storage_proof = proof_data.storage_proof.first().context("No proof found")?;

                let key = &storage_proof.key;
                let key_hash = keccak256(key);
                let encoded_val = storage_proof.value.rlp_bytes().freeze();

                let _is_valid = verify_proof(
                    storage_proof.proof.as_ref(),
                    storage_hash.as_bytes(),
                    key_hash.as_ref(),
                    encoded_val.as_ref(),
                );
                EthQueryResult::GetProof(proof_data)
            },
            EthQuery::GetBlockByHash(block_hash) => {
                let Some(block) =
                    self.backend.block_with_uncles(AtBlock::from(block_hash.0)).await?
                else {
                    return Ok(EthQueryResult::GetBlockByHash(None));
                };
                EthQueryResult::GetBlockByHash(Some(block))
            },
            EthQuery::ChainId => {
                let chain_id = self.backend.chain_id().await?;
                EthQueryResult::ChainId(chain_id)
            },
            EthQuery::GetLogs(logs) => {
                let block_range = BlockRange {
                    address: logs.contracts.clone(),
                    topics: logs.topics.clone(),
                    from: None,
                    to: None,
                    blockhash: Some(logs.block),
                };
                let logs = self.backend.get_logs(block_range).await?;
                EthQueryResult::GetLogs(logs)
            },
        };
        Ok(result)
    }

    /// # Errors
    /// Will return an error if the subscription lock is poisoned
    pub fn subscribe(&self, sub: &Subscription) -> Result<u32> {
        match sub {
            Subscription::Logs { address, topics } => {
                let Ok(mut log_filter) = self.log_filter.lock() else {
                    anyhow::bail!("Fatal error: subscription lock is poisoned");
                };
                log_filter.add(*address, topics.iter().copied());

                // TODO: Implement a better subscription id manager
                let mut id = [0u8; 4];
                id.copy_from_slice(&address.0[0..4]);
                Ok(u32::from_be_bytes(id))
            },
        }
    }
}

impl<P> EthereumClient<P>
where
    P: SubscriptionClientT + Send + Sync + 'static,
{
    #[allow(clippy::missing_errors_doc)]
    pub async fn listen(&self) -> Result<EthereumEventStream<'_, P>> {
        let new_heads = EthereumPubSub::new_heads(&self.backend).await?;
        Ok(EthereumEventStream::new(self, new_heads))
    }
}
