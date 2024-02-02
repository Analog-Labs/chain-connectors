use crate::{
    event_stream::EthereumEventStream,
    proof::verify_proof,
    utils::{get_non_pending_block, AtBlockExt, EthereumRpcExt, NonPendingBlock},
};
use anyhow::{Context, Result};
use ethers::{
    prelude::*,
    providers::{JsonRpcClient, Middleware, Provider},
    types::{transaction::eip2718::TypedTransaction, U64},
    utils::{keccak256, rlp::Encodable},
};
use rosetta_config_ethereum::{
    ext::types::{rpc::CallRequest, AccessList},
    BlockFull, CallContract, CallResult, EthereumMetadata, EthereumMetadataParams, GetBalance,
    GetProof, GetStorageAt, GetTransactionReceipt, Query as EthQuery,
    QueryResult as EthQueryResult,
};
use rosetta_core::{
    crypto::{address::Address, PublicKey},
    traits::{Block, Header},
    types::{BlockIdentifier, PartialBlockIdentifier},
    BlockchainConfig,
};
use rosetta_ethereum_backend::{
    ext::types::AtBlock,
    jsonrpsee::{
        core::client::{ClientT, SubscriptionClientT},
        Adapter,
    },
    EthereumRpc, ExitReason,
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
    backend: Adapter<P>,
    client: Arc<Provider<P>>,
    genesis_block: BlockFull,
    block_finality_strategy: BlockFinalityStrategy,
    nonce: Arc<std::sync::atomic::AtomicU32>,
    private_key: Option<[u8; 32]>,
}

impl<P> Clone for EthereumClient<P>
where
    P: Clone,
{
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            backend: self.backend.clone(),
            client: self.client.clone(),
            genesis_block: self.genesis_block.clone(),
            block_finality_strategy: self.block_finality_strategy,
            nonce: self.nonce.clone(),
            private_key: self.private_key,
        }
    }
}

impl<P> EthereumClient<P>
where
    P: ClientT + JsonRpcClient + Clone + 'static,
{
    #[allow(clippy::missing_errors_doc)]
    pub async fn new(
        config: BlockchainConfig,
        rpc_client: P,
        private_key: Option<[u8; 32]>,
    ) -> Result<Self> {
        let backend = Adapter(rpc_client.clone());
        let at = AtBlock::At(rosetta_config_ethereum::ext::types::BlockIdentifier::Number(0));
        let Some(genesis_block) = backend
            .block_full::<rosetta_config_ethereum::ext::types::SignedTransaction<
                rosetta_config_ethereum::ext::types::TypedTransaction,
            >, rosetta_config_ethereum::ext::types::SealedHeader>(at)
            .await?
        else {
            anyhow::bail!("FATAL: genesis block not found");
        };

        let block_finality_strategy = BlockFinalityStrategy::from_config(&config);
        let client = Arc::new(Provider::new(rpc_client));
        let (private_key, nonce) = if let Some(private) = private_key {
            let wallet = LocalWallet::from_bytes(&private)?;
            let address = wallet.address();
            let nonce = Arc::new(atomic::AtomicU32::from(
                client.get_transaction_count(address, None).await?.as_u32(),
            ));
            (private_key, nonce)
        } else {
            (None, Arc::new(atomic::AtomicU32::new(0)))
        };
        Ok(Self {
            config,
            backend,
            client,
            genesis_block: genesis_block.into(),
            block_finality_strategy,
            nonce,
            private_key,
        })
    }
}

impl<P> EthereumClient<P>
where
    P: ClientT + JsonRpcClient + 'static,
{
    pub const fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    pub fn genesis_block(&self) -> BlockIdentifier {
        BlockIdentifier {
            index: self.genesis_block.header().0.header().number,
            hash: self.genesis_block.0.header().hash().0,
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn current_block(&self) -> Result<BlockIdentifier> {
        let Some(header) = self.backend.block(AtBlock::Latest).await?.map(|block| block.unseal().0)
        else {
            anyhow::bail!("[report this bug] latest block not found");
        };
        Ok(BlockIdentifier { index: header.number(), hash: header.hash().0 })
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn finalized_block(&self, latest_block: Option<u64>) -> Result<NonPendingBlock> {
        let number: BlockNumber = match self.block_finality_strategy {
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
                    let genesis = &self.genesis_block;
                    let header = genesis.header().0.header();
                    let body = genesis.0.body();
                    let block = NonPendingBlock {
                        hash: genesis.hash().0,
                        number: genesis.header().number(),
                        identifier: BlockIdentifier {
                            hash: genesis.hash().0 .0,
                            index: genesis.header().number(),
                        },
                        block: ethers::types::Block {
                            hash: Some(genesis.header().hash().0),
                            parent_hash: header.parent_hash,
                            uncles_hash: header.ommers_hash,
                            author: Some(header.beneficiary),
                            state_root: header.state_root,
                            transactions_root: header.transactions_root,
                            receipts_root: header.receipts_root,
                            number: Some(header.number.into()),
                            gas_used: header.gas_used.into(),
                            gas_limit: header.gas_limit.into(),
                            extra_data: header.extra_data.0.clone().into(),
                            logs_bloom: Some(header.logs_bloom),
                            timestamp: header.timestamp.into(),
                            difficulty: header.difficulty,
                            total_difficulty: body.total_difficulty,
                            seal_fields: body
                                .seal_fields
                                .iter()
                                .map(|b| b.0.clone().into())
                                .collect(),
                            uncles: body
                                .uncles
                                .iter()
                                .map(rosetta_config_ethereum::ext::types::SealedHeader::hash)
                                .collect(),
                            transactions: Vec::new(), // Genesis doesn't contain transactions
                            size: body.size.map(U256::from),
                            mix_hash: Some(header.mix_hash),
                            nonce: Some(H64::from_low_u64_ne(header.nonce)),
                            base_fee_per_gas: header.base_fee_per_gas.map(U256::from),
                            blob_gas_used: header.blob_gas_used.map(U256::from),
                            excess_blob_gas: header.excess_blob_gas.map(U256::from),
                            withdrawals_root: header.withdrawals_root,
                            withdrawals: None,
                            parent_beacon_block_root: header.parent_beacon_block_root,
                            other: OtherFields::default(),
                        },
                    };
                    return Ok(block);
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
                // let coinbase = self.client.get_accounts().await?[0];
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
                use rosetta_config_ethereum::ext::types::{
                    rpc::RpcTransaction, SealedBlock, SealedHeader, SignedTransaction,
                    TypedTransaction,
                };
                let Some(block) = self
                    .backend
                    .block_full::<RpcTransaction, SealedHeader>(AtBlock::from(*block_hash))
                    .await?
                else {
                    return Ok(EthQueryResult::GetBlockByHash(None));
                };

                let (header, body) = block.unseal();
                let transactions = body
                    .transactions
                    .iter()
                    .map(|tx| SignedTransaction::<TypedTransaction>::try_from(tx.clone()))
                    .collect::<Result<Vec<SignedTransaction<TypedTransaction>>, _>>()
                    .map_err(|err| anyhow::format_err!(err))?;
                let body = body.with_transactions(transactions);
                let block = SealedBlock::new(header, body);
                EthQueryResult::GetBlockByHash(Some(block.into()))
            },
            EthQuery::ChainId => {
                let chain_id = self.backend.chain_id().await?;
                EthQueryResult::ChainId(chain_id)
            },
        };
        Ok(result)
    }
}

impl<P> EthereumClient<P>
where
    P: SubscriptionClientT + PubsubClient + 'static,
{
    #[allow(clippy::missing_errors_doc)]
    pub async fn listen(&self) -> Result<EthereumEventStream<'_, P>> {
        let new_head_subscription = self.client.subscribe_blocks().await?;
        Ok(EthereumEventStream::new(self, new_head_subscription))
    }
}
