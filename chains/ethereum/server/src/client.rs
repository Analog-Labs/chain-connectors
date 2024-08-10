#![allow(clippy::option_if_let_else)]
use crate::{
    block_stream::BlockStream,
    log_filter::LogFilter,
    proof::verify_proof,
    shared_stream::SharedStream,
    state::State,
    utils::{
        AtBlockExt, DefaultFeeEstimatorConfig, EthereumRpcExt, PartialBlock,
        PolygonFeeEstimatorConfig,
    },
};
use anyhow::{Context, Result};
use futures_util::StreamExt;
use rosetta_config_ethereum::{
    ext::types::{
        crypto::{Crypto, DefaultCrypto, Keypair, Signer},
        ext::rlp::Encodable,
        rlp_utils::RlpDecodableTransaction,
        rpc::CallRequest,
        transactions::LegacyTransaction,
        AccessList, AtBlock, Bytes, TransactionT, TypedTransaction, H160, U256,
    },
    query::GetBlock,
    CallContract, CallResult, EthereumMetadata, EthereumMetadataParams, GetBalance, GetProof,
    GetStorageAt, GetTransactionCount, GetTransactionReceipt, Query as EthQuery,
    QueryResult as EthQueryResult, SubmitResult, Subscription,
};

use rosetta_core::{
    crypto::{address::Address, PublicKey},
    types::{BlockIdentifier, PartialBlockIdentifier},
    BlockchainConfig, ClientEvent,
};
use rosetta_ethereum_backend::{
    jsonrpsee::{
        core::client::{ClientT, SubscriptionClientT},
        Adapter,
    },
    BlockRange, EthereumRpc, ExitReason,
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
    chain_id: u64,
    config: BlockchainConfig,
    pub backend: Adapter<P>,
    genesis_block: PartialBlock,
    block_finality_strategy: BlockFinalityStrategy,
    nonce: Arc<std::sync::atomic::AtomicU64>,
    private_key: Option<[u8; 32]>,
    log_filter: Arc<std::sync::Mutex<LogFilter>>,
    // event_stream: SharedStream<BlockStream<Adapter<P>>>
}

impl<P> Clone for EthereumClient<P>
where
    P: Clone,
{
    fn clone(&self) -> Self {
        Self {
            chain_id: self.chain_id,
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

        // Get the chain id
        let chain_id = backend.chain_id().await?;

        // Get the genesis block
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

        // Get the block finality strategy
        let block_finality_strategy = BlockFinalityStrategy::from_config(&config);

        // Load the funding wallet, if any
        let (private_key, nonce) = if let Some(private) = private_key {
            let wallet = Keypair::from_slice(&private)?;
            let address = wallet.address();
            let nonce = Arc::new(atomic::AtomicU64::from(
                backend.get_transaction_count(address, AtBlock::Latest).await?,
            ));
            (private_key, nonce)
        } else {
            (None, Arc::new(atomic::AtomicU64::new(0)))
        };
        Ok(Self {
            chain_id,
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
    pub async fn faucet(&self, address: &Address, param: u128, high_gas_price: Option<u128>) -> Result<Vec<u8>> {
        match self.private_key {
            Some(private_key) => {
                let chain_id = self.chain_id;
                let wallet = Keypair::from_bytes(private_key)?;
                let address: H160 = address.address().parse()?;
                let nonce = self.nonce.load(Ordering::Relaxed);
                let gas_price = if let Some(high_gas_price) = high_gas_price {
                    U256::from(high_gas_price)
                } else {
                    U256::from(500_000_000) // Default gas price
                };
                // Create a transaction request
                let transaction_request = LegacyTransaction {
                    to: Some(address),
                    value: U256::from(param),
                    gas_limit: 210_000,
                    gas_price,
                    nonce,
                    data: Bytes::default(),
                    chain_id: Some(chain_id),
                };
                let tx: TypedTransaction = transaction_request.into();
                let signature = wallet.sign_prehash(tx.sighash(), Some(chain_id))?;
                let raw_tx = tx.encode(Some(&signature));
                let tx_hash = self.backend.send_raw_transaction(raw_tx).await?;

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
                    if self.config().blockchain == "polygon" {
                        self.backend.estimate_eip1559_fees::<PolygonFeeEstimatorConfig>().await?
                    } else {
                        self.backend.estimate_eip1559_fees::<DefaultFeeEstimatorConfig>().await?
                    };
                let tx = CallRequest {
                    from: Some(coinbase),
                    to: Some(address),
                    gas_limit: None,
                    gas_price: None,
                    value: Some(U256::from(param)),
                    data: None,
                    nonce: None,
                    chain_id: None, // Astar doesn't support this field for eth_call
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
        let to = options.destination.map(H160);
        let (max_fee_per_gas, max_priority_fee_per_gas) = if self.config().blockchain == "polygon" {
            self.backend.estimate_eip1559_fees::<PolygonFeeEstimatorConfig>().await?
        } else {
            self.backend.estimate_eip1559_fees::<DefaultFeeEstimatorConfig>().await?
        };
        let chain_id = self.backend.chain_id().await?;

        let nonce = if let Some(nonce) = options.nonce {
            nonce
        } else {
            self.backend.get_transaction_count(from, AtBlock::Latest).await?
        };
        let mut tx = CallRequest {
            from: Some(from),
            to,
            gas_limit: None,
            gas_price: None,
            value: Some(U256(options.amount)),
            data: Some(options.data.clone().into()),
            nonce: None,
            chain_id: None, // Astar doesn't support this field for eth_call
            max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
            access_list: AccessList::default(),
            max_fee_per_gas: Some(max_fee_per_gas),
            transaction_type: Some(2),
        };
        let gas_limit = if let Some(gas_limit) = options.gas_limit {
            gas_limit
        } else {
            let gas_limit = self.backend.estimate_gas(&tx, AtBlock::Latest).await?;
            u64::try_from(gas_limit).unwrap_or(u64::MAX)
        };

        tx.nonce = Some(nonce);

        Ok(EthereumMetadata {
            chain_id,
            nonce,
            max_priority_fee_per_gas: max_priority_fee_per_gas.0,
            max_fee_per_gas: max_fee_per_gas.0,
            gas_limit,
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn submit(&self, transaction: &[u8]) -> Result<SubmitResult> {
        // Check if the transaction is valid and signed
        let rlp = rosetta_config_ethereum::ext::types::ext::rlp::Rlp::new(transaction);
        let (tx_hash, call_request) = match TypedTransaction::rlp_decode(&rlp, true) {
            Ok((tx, Some(signature))) => {
                let tx_hash = tx.compute_tx_hash(&signature);
                let sender = DefaultCrypto::secp256k1_ecdsa_recover(&signature, tx.sighash())?;
                // Obs: this call is used only to retrieve the revert reason
                let call_request = CallRequest {
                    from: Some(sender),
                    to: tx.to(),
                    gas_limit: Some(tx.gas_limit()),
                    gas_price: None,
                    value: Some(tx.value()),
                    data: Some(Bytes::from_iter(tx.data())),
                    nonce: None, // Omit the nonce, once it was causing issues in astar
                    chain_id: None,
                    max_priority_fee_per_gas: None,
                    access_list: tx.access_list().cloned().unwrap_or_default(),
                    max_fee_per_gas: None,
                    transaction_type: None,
                };
                (tx_hash, call_request)
            },
            Ok((_, None)) => {
                anyhow::bail!("Invalid Transaction: not signed");
            },
            Err(_) => anyhow::bail!("Invalid Transaction: failed to parse, must be a valid EIP1159, EIP-Eip2930 or Legacy"),
        };

        // Check if the transaction is already included in a block
        if let Some(receipt) = self.backend.transaction_receipt(tx_hash).await? {
            return Ok(self.backend.get_call_result(receipt, call_request).await);
        }

        // Check if the message is not peding
        if self.backend.transaction_by_hash(tx_hash).await?.is_none() {
            // Send the transaction
            let actual_hash =
                self.backend.send_raw_transaction(Bytes::from_iter(transaction)).await?;
            if tx_hash != actual_hash {
                anyhow::bail!("Transaction hash mismatch, expect {tx_hash}, got {actual_hash}");
            }
        }

        // Wait for the transaction receipt
        let Ok(receipt) = self.backend.wait_for_transaction_receipt(tx_hash).await else {
            tracing::warn!("Transaction receipt timeout: {tx_hash:?}");
            return Ok(SubmitResult::Timeout { tx_hash });
        };
        tracing::debug!(
            "Transaction included in a block: {tx_hash:?}, status: {:?}",
            receipt.status_code
        );
        Ok(self.backend.get_call_result(receipt, call_request).await)
    }

    #[allow(clippy::too_many_lines, clippy::missing_errors_doc)]
    pub async fn call(&self, req: &EthQuery) -> Result<EthQueryResult> {
        let result = match req {
            EthQuery::GetBalance(GetBalance { address, block }) => {
                let balance = self.backend.get_balance(*address, *block).await?;
                EthQueryResult::GetBalance(balance)
            },
            EthQuery::GetTransactionCount(GetTransactionCount { address, block }) => {
                let nonce = self.backend.get_transaction_count(*address, *block).await?;
                EthQueryResult::GetTransactionCount(nonce)
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
                let key_hash = DefaultCrypto::keccak256(key);
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
            EthQuery::GetBlock(GetBlock(at)) => {
                let Some(block) = self.backend.block_with_uncles(*at).await? else {
                    return Ok(EthQueryResult::GetBlock(None));
                };
                EthQueryResult::GetBlock(Some(block))
            },
            EthQuery::ChainId => {
                let chain_id = self.backend.chain_id().await?;
                EthQueryResult::ChainId(chain_id)
            },
            EthQuery::GetLogs(logs) => {
                let block_range = BlockRange {
                    address: logs.contracts.clone(),
                    topics: logs.topics.clone(),
                    filter: logs.block,
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
    P: SubscriptionClientT + Unpin + Clone + Send + Sync + 'static,
{
    #[allow(clippy::missing_errors_doc)]
    pub async fn listen(&self) -> Result<SharedStream<BlockStream<Adapter<P>>>> {
        let best_finalized_block = self.finalized_block(None).await?;
        let mut stream = BlockStream::new(self.backend.clone(), State::new(best_finalized_block));
        match stream.next().await {
            Some(ClientEvent::Close(msg)) => anyhow::bail!(msg),
            None => anyhow::bail!("Failed to open the event stream"),
            Some(_) => {},
        }
        Ok(SharedStream::new(stream, 100))
    }
}
