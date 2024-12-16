use rosetta_config_ethereum::{
    ext::types::{
        rpc::CallRequest, SealedBlock, SealedHeader, SignedTransaction, TransactionReceipt,
        TypedTransaction, H256, I256, U256,
    },
    AtBlock, CallResult, SubmitResult,
};
use rosetta_core::types::PartialBlockIdentifier;
use rosetta_ethereum_backend::{jsonrpsee::core::ClientError, EthereumRpc, ExitReason};
use std::string::ToString;

pub type FullBlock = SealedBlock<SignedTransaction<TypedTransaction>, SealedHeader>;
pub type PartialBlock = SealedBlock<H256, H256>;
pub type PartialBlockWithUncles = SealedBlock<H256, SealedHeader>;

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

pub trait AtBlockExt {
    fn from_partial_identifier(block_identifier: &PartialBlockIdentifier) -> Self;
}

impl AtBlockExt for AtBlock {
    fn from_partial_identifier(block_identifier: &PartialBlockIdentifier) -> Self {
        match (block_identifier.index, block_identifier.hash) {
            (_, Some(hash)) => Self::from(hash),
            (Some(index), None) => Self::from(index),
            (None, None) => Self::Latest,
        }
    }
}

pub trait FeeEstimatorConfig {
    /// The number of blocks from the past for which the fee rewards are fetched for fee estimation.
    const EIP1559_FEE_ESTIMATION_PAST_BLOCKS: u64;
    /// The default percentile of gas premiums that are fetched for fee estimation.
    const EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE: f64;
    /// The default max priority fee per gas, used in case the base fee is within a threshold.
    const EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE: u64;
    /// The threshold for base fee below which we use the default priority fee, and beyond which we
    /// estimate an appropriate value for priority fee.
    const EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER: u64;
    /// The threshold max change/difference (in %) at which we will ignore the fee history values
    /// under it.
    const EIP1559_FEE_ESTIMATION_THRESHOLD_MAX_CHANGE: i64;
    /// Different evm blockchains returns base fee in different units. Like Ethereum returns in wei,
    /// Polygon returns in gwei. so this multiplier converts them to wei format in order to
    /// calculate gas fee
    const EIP1559_BASE_FEE_MULTIPLIER: u64;
}

// Default config for ethereum and astar
pub struct DefaultFeeEstimatorConfig {}

impl FeeEstimatorConfig for DefaultFeeEstimatorConfig {
    const EIP1559_FEE_ESTIMATION_PAST_BLOCKS: u64 = 10;
    const EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE: f64 = 5.0;
    const EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE: u64 = 3_000_000_000;
    const EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER: u64 = 100_000_000_000;
    const EIP1559_FEE_ESTIMATION_THRESHOLD_MAX_CHANGE: i64 = 200;
    const EIP1559_BASE_FEE_MULTIPLIER: u64 = 1;
}

// Polygon Amoy fee estimator config
pub struct PolygonFeeEstimatorConfig {}

impl FeeEstimatorConfig for PolygonFeeEstimatorConfig {
    // Computes safe low,
    // reference: https://docs.polygon.technology/tools/gas/polygon-gas-station/
    const EIP1559_FEE_ESTIMATION_PAST_BLOCKS: u64 = 15;
    const EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE: f64 = 10.0;
    const EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE: u64 = 30_000_000_000;
    const EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER: u64 = 0;
    const EIP1559_FEE_ESTIMATION_THRESHOLD_MAX_CHANGE: i64 = 200;
    // Polygon returns base fee in gwei. we need to convert it into wei
    const EIP1559_BASE_FEE_MULTIPLIER: u64 = 1_000_000_000;
}

fn estimate_priority_fee<F: FeeEstimatorConfig>(rewards: &[Vec<U256>]) -> U256 {
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
    let values = if max_change >= F::EIP1559_FEE_ESTIMATION_THRESHOLD_MAX_CHANGE.into()
        && (max_change_index >= (rewards.len() / 2))
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

pub fn eip1559_default_estimator<F: FeeEstimatorConfig>(
    base_fee_per_gas: U256,
    rewards: &[Vec<U256>],
) -> (U256, U256) {
    let max_priority_fee_per_gas =
        if base_fee_per_gas < U256::from(F::EIP1559_FEE_ESTIMATION_PRIORITY_FEE_TRIGGER) {
            U256::from(F::EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE)
        } else {
            std::cmp::max(
                estimate_priority_fee::<F>(rewards),
                U256::from(F::EIP1559_FEE_ESTIMATION_DEFAULT_PRIORITY_FEE),
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

    async fn get_call_result(
        &self,
        receipt: TransactionReceipt,
        call_request: CallRequest,
    ) -> SubmitResult;

    async fn estimate_eip1559_fees<F: FeeEstimatorConfig>(&self) -> anyhow::Result<(U256, U256)>;

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
    // Wait for the transaction to be included in a block by polling the transaction receipt every 2
    // seconds
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
                    anyhow::bail!(
                        "Transaction not included in a block after {} seconds",
                        timeout.as_secs()
                    );
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            };
            break receipt;
        };
        Ok(receipt)
    }

    async fn get_call_result(
        &self,
        receipt: TransactionReceipt,
        call_request: CallRequest,
    ) -> SubmitResult {
        // Helper function used when we can't fetch the revert reason or call result
        fn result_from_receipt(tx_hash: H256, receipt: TransactionReceipt) -> SubmitResult {
            if receipt.status_code == Some(1) {
                SubmitResult::Executed { tx_hash, result: CallResult::Success(Vec::new()), receipt }
            } else {
                SubmitResult::Executed { tx_hash, result: CallResult::Error, receipt }
            }
        }

        let tx_hash = receipt.transaction_hash;

        // Fetch the block to get the parent_hash
        let block_number = match receipt.block_number {
            Some(block_number) => block_number,
            None => match self.block(receipt.block_hash.into()).await {
                Ok(Some(block)) => block.header.number,
                Ok(None) => {
                    tracing::warn!("Block {:?} not found", receipt.block_hash);
                    return result_from_receipt(tx_hash, receipt);
                },
                Err(error) => {
                    tracing::warn!(
                        "Failed to retrieve block by hash {:?}: {error:?}",
                        receipt.block_hash
                    );
                    return result_from_receipt(tx_hash, receipt);
                },
            },
        };

        // Execute the call in the parent block_hash to get the transaction result
        let exit_reason = match self
            .call(&call_request, AtBlock::At(block_number.saturating_sub(1).into()))
            .await
        {
            Ok(exit_reason) => exit_reason,
            Err(error) => {
                if matches!(receipt.status_code, Some(0) | None) {
                    tracing::warn!(
                        "Failed to retrieve transaction revert reason: {tx_hash:?}: {error:?}"
                    );
                } else {
                    // Using debug level, once retrieve the transaction result is not critical
                    tracing::debug!(
                        "Failed to retrieve transaction result: {tx_hash:?}: {error:?}"
                    );
                }
                return result_from_receipt(tx_hash, receipt);
            },
        };
        SubmitResult::Executed {
            tx_hash,
            receipt,
            result: match exit_reason {
                ExitReason::Succeed(bytes) => CallResult::Success(bytes.to_vec()),
                ExitReason::Revert(bytes) => CallResult::Revert(bytes.to_vec()),
                ExitReason::Error(_) => CallResult::Error,
            },
        }
    }

    async fn estimate_eip1559_fees<F: FeeEstimatorConfig>(&self) -> anyhow::Result<(U256, U256)> {
        let Some(block) = self.block(AtBlock::Latest).await? else {
            anyhow::bail!("latest block not found");
        };
        let Some(mut base_fee_per_gas) = block.header.base_fee_per_gas else {
            anyhow::bail!("EIP-1559 not activated");
        };

        base_fee_per_gas = base_fee_per_gas.saturating_mul(F::EIP1559_BASE_FEE_MULTIPLIER);

        let fee_history = self
            .fee_history(
                F::EIP1559_FEE_ESTIMATION_PAST_BLOCKS,
                AtBlock::Latest,
                &[F::EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE],
            )
            .await?;

        // Estimate fees
        let (max_fee_per_gas, max_priority_fee_per_gas) =
            eip1559_default_estimator::<F>(base_fee_per_gas.into(), fee_history.reward.as_ref());
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
