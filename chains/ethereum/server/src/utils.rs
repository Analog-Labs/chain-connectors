use ethers::{prelude::*, types::H256};
use rosetta_config_ethereum::{
    ext::types::{
        rpc::{RpcBlock, RpcTransaction},
        SealedBlock, SealedHeader, SignedTransaction, TransactionReceipt, TypedTransaction,
    },
    AtBlock,
};
use rosetta_core::types::{BlockIdentifier, PartialBlockIdentifier};
use rosetta_ethereum_backend::{jsonrpsee::core::ClientError, EthereumRpc};
use std::string::ToString;

pub type FullBlock = SealedBlock<SignedTransaction<TypedTransaction>, SealedHeader>;
pub type PartialBlock = SealedBlock<H256, H256>;
pub type PartialBlockWithUncles = SealedBlock<H256, SealedHeader>;

/// A block that is not pending, so it must have a valid hash and number.
/// This allow skipping duplicated checks in the code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonPendingBlock {
    pub hash: H256,
    pub number: u64,
    pub identifier: BlockIdentifier,
    pub block: ethers::types::Block<H256>,
}

/// Maximum length of error messages to log.
const ERROR_MSG_MAX_LENGTH: usize = 100;

/// Helper type that truncates the error message to `ERROR_MSG_MAX_LENGTH` before logging.
pub struct SafeLogError<'a, T>(&'a T);

impl<T> std::fmt::Display for SafeLogError<'_, T>
where
    T: ToString,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = <T as ToString>::to_string(self.0);
        let msg_str = msg.trim();
        if msg_str.chars().count() > ERROR_MSG_MAX_LENGTH {
            let msg = msg_str.chars().take(ERROR_MSG_MAX_LENGTH).collect::<String>();
            let msg_str = msg.trim_end();
            write!(f, "{msg_str}...")
        } else {
            write!(f, "{msg_str}")
        }
    }
}

pub trait LogErrorExt: Sized {
    fn truncate(&self) -> SafeLogError<'_, Self>;
}

impl LogErrorExt for rosetta_ethereum_backend::jsonrpsee::core::ClientError {
    fn truncate(&self) -> SafeLogError<'_, Self> {
        SafeLogError(self)
    }
}

pub trait AtBlockExt {
    fn as_block_id(&self) -> ethers::types::BlockId;
    fn from_partial_identifier(block_identifier: &PartialBlockIdentifier) -> Self;
}

impl AtBlockExt for AtBlock {
    fn as_block_id(&self) -> ethers::types::BlockId {
        use rosetta_config_ethereum::ext::types::BlockIdentifier;
        match self {
            Self::Latest => BlockId::Number(BlockNumber::Latest),
            Self::Earliest => BlockId::Number(BlockNumber::Earliest),
            Self::Finalized => BlockId::Number(BlockNumber::Finalized),
            Self::Pending => BlockId::Number(BlockNumber::Pending),
            Self::Safe => BlockId::Number(BlockNumber::Safe),
            Self::At(BlockIdentifier::Hash(hash)) => BlockId::Hash(*hash),
            Self::At(BlockIdentifier::Number(number)) => {
                BlockId::Number(BlockNumber::Number((*number).into()))
            },
        }
    }

    fn from_partial_identifier(block_identifier: &PartialBlockIdentifier) -> Self {
        match (block_identifier.index, block_identifier.hash) {
            (_, Some(hash)) => Self::from(hash),
            (Some(index), None) => Self::from(index),
            (None, None) => Self::Latest,
        }
    }
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

/// The number of blocks from the past for which the fee rewards are fetched for fee estimation.
const EIP1559_FEE_ESTIMATION_PAST_BLOCKS: u64 = 10;
/// The default percentile of gas premiums that are fetched for fee estimation.
const EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE: f64 = 5.0;
/// The default max priority fee per gas, used in case the base fee is within a threshold.
const EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE: u64 = 3_000_000_000;
/// The threshold for base fee below which we use the default priority fee, and beyond which we
/// estimate an appropriate value for priority fee.
const EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER: u64 = 100_000_000_000;
/// The threshold max change/difference (in %) at which we will ignore the fee history values
/// under it.
const EIP1559_FEE_ESTIMATION_THRESHOLD_MAX_CHANGE: i64 = 200;

fn estimate_priority_fee(rewards: &[Vec<U256>]) -> U256 {
    let mut rewards: Vec<U256> =
        rewards.iter().map(|r| r[0]).filter(|r| *r > U256::zero()).collect();
    if rewards.is_empty() {
        return U256::zero();
    }
    if rewards.len() == 1 {
        return rewards[0];
    }
    // Sort the rewards as we will eventually take the median.
    rewards.sort();

    // A copy of the same vector is created for convenience to calculate percentage change
    // between subsequent fee values.
    let mut rewards_copy = rewards.clone();
    rewards_copy.rotate_left(1);

    let mut percentage_change: Vec<I256> = rewards
        .iter()
        .zip(rewards_copy.iter())
        .map(|(a, b)| {
            let a = I256::try_from(*a).unwrap_or(I256::MAX);
            let b = I256::try_from(*b).unwrap_or(I256::MAX);
            ((b - a) * 100) / a
        })
        .collect();
    percentage_change.pop();

    // Fetch the max of the percentage change, and that element's index.
    let max_change = percentage_change.iter().max().copied().unwrap_or(I256::zero());
    let max_change_index = percentage_change.iter().position(|&c| c == max_change).unwrap_or(0);

    // If we encountered a big change in fees at a certain position, then consider only
    // the values >= it.
    let values = if max_change >= EIP1559_FEE_ESTIMATION_THRESHOLD_MAX_CHANGE.into() &&
        (max_change_index >= (rewards.len() / 2))
    {
        rewards[max_change_index..].to_vec()
    } else {
        rewards
    };

    // Return the median.
    values[values.len() / 2]
}

fn base_fee_surged(base_fee_per_gas: U256) -> U256 {
    if base_fee_per_gas <= U256::from(40_000_000_000u64) {
        base_fee_per_gas * 2
    } else if base_fee_per_gas <= U256::from(100_000_000_000u64) {
        base_fee_per_gas * 16 / 10
    } else if base_fee_per_gas <= U256::from(200_000_000_000u64) {
        base_fee_per_gas * 14 / 10
    } else {
        base_fee_per_gas * 12 / 10
    }
}

pub fn eip1559_default_estimator(base_fee_per_gas: U256, rewards: &[Vec<U256>]) -> (U256, U256) {
    let max_priority_fee_per_gas =
        if base_fee_per_gas < U256::from(EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER) {
            U256::from(EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE)
        } else {
            std::cmp::max(
                estimate_priority_fee(rewards),
                U256::from(EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE),
            )
        };
    let potential_max_fee = base_fee_surged(base_fee_per_gas);
    let max_fee_per_gas = if max_priority_fee_per_gas > potential_max_fee {
        max_priority_fee_per_gas + potential_max_fee
    } else {
        potential_max_fee
    };
    (max_fee_per_gas, max_priority_fee_per_gas)
}

#[async_trait::async_trait]
pub trait EthereumRpcExt {
    async fn wait_for_transaction_receipt(
        &self,
        tx_hash: H256,
    ) -> anyhow::Result<TransactionReceipt>;

    async fn estimate_eip1559_fees(&self) -> anyhow::Result<(U256, U256)>;

    async fn block_with_uncles(
        &self,
        at: AtBlock,
    ) -> Result<Option<PartialBlockWithUncles>, ClientError>;
}

#[async_trait::async_trait]
impl<T> EthereumRpcExt for T
where
    T: EthereumRpc<Error = ClientError> + Send + Sync + 'static,
{
    // Wait for the transaction to be mined by polling the transaction receipt every 2 seconds
    async fn wait_for_transaction_receipt(
        &self,
        tx_hash: H256,
    ) -> anyhow::Result<TransactionReceipt> {
        let now = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);
        let receipt = loop {
            let Some(receipt) = <T as EthereumRpc>::transaction_receipt(self, tx_hash).await?
            else {
                if now.elapsed() > timeout {
                    anyhow::bail!("Transaction not mined after {} seconds", timeout.as_secs());
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            };
            break receipt;
        };
        Ok(receipt)
    }

    async fn estimate_eip1559_fees(&self) -> anyhow::Result<(U256, U256)> {
        let Some(block) = self.block(AtBlock::Latest).await? else {
            anyhow::bail!("latest block not found");
        };
        let Some(base_fee_per_gas) = block.header.base_fee_per_gas else {
            anyhow::bail!("EIP-1559 not activated");
        };

        let fee_history = self
            .fee_history(
                EIP1559_FEE_ESTIMATION_PAST_BLOCKS,
                AtBlock::Latest,
                &[EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE],
            )
            .await?;

        // Estimate fees
        let (max_fee_per_gas, max_priority_fee_per_gas) =
            eip1559_default_estimator(base_fee_per_gas.into(), fee_history.reward.as_ref());
        Ok((max_fee_per_gas, max_priority_fee_per_gas))
    }

    async fn block_with_uncles(
        &self,
        at: AtBlock,
    ) -> Result<Option<PartialBlockWithUncles>, ClientError> {
        let Some(block) = self.block(at).await? else {
            return Ok(None);
        };

        // Convert the `RpcBlock` to `SealedBlock`
        let block = SealedBlock::try_from(block)
            .map_err(|err| ClientError::Custom(format!("invalid block: {err}")))?;

        // Fetch block uncles
        let block_hash = block.header().hash();
        let mut uncles = Vec::with_capacity(block.body().uncles.len());
        for index in 0..block.body().uncles.len() {
            let index = u32::try_from(index).unwrap_or(u32::MAX);
            let Some(uncle) = self.uncle_by_blockhash(block_hash, index).await? else {
                return Err(ClientError::Custom(format!(
                    "uncle not found for block {block_hash:?} at index {index}"
                )));
            };
            uncles.push(uncle);
        }
        let block = block.with_ommers(uncles);
        Ok(Some(block))
    }
}

pub trait RpcBlockExt {
    fn try_into_sealed(
        self,
    ) -> anyhow::Result<SealedBlock<SignedTransaction<TypedTransaction>, H256>>;
}

impl RpcBlockExt for RpcBlock<RpcTransaction, H256> {
    fn try_into_sealed(
        self,
    ) -> anyhow::Result<SealedBlock<SignedTransaction<TypedTransaction>, TxHash>> {
        // Convert the `RpcBlock` to `SealedBlock`
        let block = SealedBlock::try_from(self)
            .map_err(|err| anyhow::format_err!("invalid block: {err}"))?;

        // Convert the `RpcTransaction` to `SignedTransaction`
        let block_hash = block.header().hash();
        let block = {
            let transactions = block
                .body()
                .transactions
                .iter()
                .enumerate()
                .map(|(index, tx)| {
                    SignedTransaction::try_from(tx.clone()).map_err(|err| {
                        anyhow::format_err!(
                            "Invalid tx in block {block_hash:?} at index {index}: {err}"
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            block.with_transactions(transactions)
        };
        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    #[test]
    fn it_works() {
        use rosetta_config_ethereum::ext::types::{Address, Bloom, BloomInput, H256};
        use std::str::FromStr;

        let expect = Bloom::from_str("40000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000400000000000000000040000000000").unwrap();
        let address = Address::from(hex!("3c9eaef1ee4c91682a070b0acbcd7ab55abad44c"));
        let topic = H256(hex!("93fe6d397c74fdf1402a8b72e47b68512f0510d7b98a4bc4cbdf6ac7108b3c59"));

        assert!(expect.contains_input(BloomInput::Raw(address.as_bytes())));
        assert!(expect.contains_input(BloomInput::Raw(topic.as_bytes())));

        let mut actual = Bloom::default();
        actual.accrue(BloomInput::Raw(address.as_bytes()));
        actual.accrue(BloomInput::Raw(topic.as_bytes()));

        assert_eq!(actual, expect);
    }
}
