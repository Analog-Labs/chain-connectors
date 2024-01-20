use crate::eth_types::{
    BYZANTIUM_BLOCK_REWARD, CONSTANTINOPLE_BLOCK_REWARD, FRONTIER_BLOCK_REWARD, MAX_UNCLE_DEPTH,
    MINING_REWARD_OP_TYPE, SUCCESS_STATUS, TESTNET_CHAIN_CONFIG, UNCLE_REWARD_MULTIPLIER,
    UNCLE_REWARD_OP_TYPE,
};
use anyhow::{Context, Result};
use ethers::{
    prelude::*,
    providers::Middleware,
    types::{Block, Transaction, H256, U64},
    utils::to_checksum,
};
use rosetta_core::{
    types::{
        self as rosetta_types, AccountIdentifier, Amount, BlockIdentifier, Operation,
        OperationIdentifier, TransactionIdentifier,
    },
    BlockchainConfig,
};
use std::sync::Arc;

/// A block that is not pending, so it must have a valid hash and number.
/// This allow skipping duplicated checks in the code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonPendingBlock {
    pub hash: H256,
    pub number: u64,
    pub identifier: BlockIdentifier,
    pub block: ethers::types::Block<H256>,
}

impl TryFrom<ethers::types::Block<H256>> for NonPendingBlock {
    type Error = anyhow::Error;

    fn try_from(block: ethers::types::Block<H256>) -> std::result::Result<Self, Self::Error> {
        let Some(number) = block.number else { anyhow::bail!("block number is missing") };
        let Some(hash) = block.hash else { anyhow::bail!("block hash is missing") };
        Ok(Self {
            hash,
            number: number.as_u64(),
            identifier: BlockIdentifier::new(number.as_u64(), hash.0),
            block,
        })
    }
}

// Retrieve a non-pending block
pub async fn get_non_pending_block<C, ID>(
    client: Arc<Provider<C>>,
    block_id: ID,
) -> Result<Option<NonPendingBlock>>
where
    C: JsonRpcClient + 'static,
    ID: Into<BlockId> + Send + Sync,
{
    let block_id = block_id.into();
    if matches!(block_id, BlockId::Number(BlockNumber::Pending)) {
        anyhow::bail!("request a pending block is not allowed");
    }
    let Some(block) = client.get_block(block_id).await? else {
        return Ok(None);
    };
    // The block is not pending, it MUST have a valid hash and number
    let Ok(block) = NonPendingBlock::try_from(block) else {
        anyhow::bail!(
            "[RPC CLIENT BUG] the rpc client returned an invalid non-pending block at {block_id:?}"
        );
    };
    Ok(Some(block))
}

pub async fn block_reward_transaction<P: JsonRpcClient>(
    client: &Provider<P>,
    config: &BlockchainConfig,
    block: &Block<Transaction>,
) -> Result<rosetta_types::Transaction> {
    let block_number = block.number.context("missing block number")?.as_u64();
    let block_hash = block.hash.context("missing block hash")?;
    let block_id = BlockId::Hash(block_hash);
    let miner = block.author.context("missing block author")?;

    let mut uncles = vec![];
    for (i, _) in block.uncles.iter().enumerate() {
        let uncle = client
            .get_uncle(block_id, U64::from(i))
            .await?
            .context("Uncle block now found")?;
        uncles.push(uncle);
    }

    let chain_config = TESTNET_CHAIN_CONFIG;
    let mut mining_reward = if chain_config.constantinople_block <= block_number {
        CONSTANTINOPLE_BLOCK_REWARD
    } else if chain_config.byzantium_block <= block_number {
        BYZANTIUM_BLOCK_REWARD
    } else {
        FRONTIER_BLOCK_REWARD
    };
    if !uncles.is_empty() {
        mining_reward += (mining_reward / UNCLE_REWARD_MULTIPLIER) * mining_reward;
    }

    let mut operations = vec![];
    let mining_reward_operation = Operation {
        operation_identifier: OperationIdentifier { index: 0, network_index: None },
        related_operations: None,
        r#type: MINING_REWARD_OP_TYPE.into(),
        status: Some(SUCCESS_STATUS.into()),
        account: Some(AccountIdentifier {
            address: to_checksum(&miner, None),
            sub_account: None,
            metadata: None,
        }),
        amount: Some(Amount {
            value: mining_reward.to_string(),
            currency: config.currency(),
            metadata: None,
        }),
        coin_change: None,
        metadata: None,
    };
    operations.push(mining_reward_operation);

    for block in uncles {
        let uncle_miner = block.author.context("Uncle block has no author")?;
        let uncle_number = block.number.context("Uncle block has no number")?;
        let uncle_block_reward =
            (uncle_number + MAX_UNCLE_DEPTH - block_number) * (mining_reward / MAX_UNCLE_DEPTH);

        let operation = Operation {
            operation_identifier: OperationIdentifier {
                index: i64::try_from(operations.len()).context("operation.index overflow")?,
                network_index: None,
            },
            related_operations: None,
            r#type: UNCLE_REWARD_OP_TYPE.into(),
            status: Some(SUCCESS_STATUS.into()),
            account: Some(AccountIdentifier {
                address: to_checksum(&uncle_miner, None),
                sub_account: None,
                metadata: None,
            }),
            amount: Some(Amount {
                value: uncle_block_reward.to_string(),
                currency: config.currency(),
                metadata: None,
            }),
            coin_change: None,
            metadata: None,
        };
        operations.push(operation);
    }

    Ok(rosetta_types::Transaction {
        transaction_identifier: TransactionIdentifier { hash: hex::encode(block_hash) },
        related_transactions: None,
        operations,
        metadata: None,
    })
}
