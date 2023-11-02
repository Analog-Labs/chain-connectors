use alloc::{collections::BTreeMap, vec::Vec};

use crate::{
    state::{PrefetchError, StateDB},
    types::{
        ExecutionError, ExecutionResult, ExecutionReverted, ExecutionSucceed, ExitError,
        ExitSucceed,
    },
};
use rosetta_ethereum_backend::{AtBlock, EthereumRpc, ExitReason, TransactionCall};
use rosetta_ethereum_primitives::{Address, Block, BlockIdentifier, H256, U256, U64};

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum Error<ERR> {
    #[cfg_attr(feature = "std", error("RPC error: {0:?}"))]
    Rpc(ERR),
    #[cfg_attr(feature = "std", error("Result mismatch"))]
    ResultMismatch { vm: ExitReason, rpc: ERR },
    #[cfg_attr(feature = "std", error("Prefetch failed: {0}"))]
    PrefetchFailed(PrefetchError<ERR>),
}

pub trait SputnikConfig {
    type PrecompileSet: sputnik_evm::executor::stack::PrecompileSet;

    fn config(&self) -> &sputnik_evm::Config;
    fn precompile_set(&self) -> &Self::PrecompileSet;
}

struct TestEnv {
    config: sputnik_evm::Config,
    precompile_set: (),
}

impl TestEnv {
    pub const fn new() -> Self {
        Self { config: sputnik_evm::Config::istanbul(), precompile_set: () }
    }
}

impl SputnikConfig for TestEnv {
    type PrecompileSet = ();

    fn config(&self) -> &sputnik_evm::Config {
        &self.config
    }

    fn precompile_set(&self) -> &Self::PrecompileSet {
        &self.precompile_set
    }
}

pub struct SputnikExecutor<RPC: EthereumRpc + Send + Sync> {
    db: StateDB<RPC>,
}

impl<RPC> SputnikExecutor<RPC>
where
    RPC: EthereumRpc + Send + Sync,
{
    pub fn new(client: RPC) -> Self {
        let db = StateDB::new(client);
        Self { db }
    }

    // Execute transaction using VM
    pub fn execute<C: SputnikConfig>(
        &mut self,
        env: &C,
        tx: &TransactionCall,
        block: &Block<H256>,
    ) -> ExecutionResult {
        use sputnik_evm::{
            backend::{MemoryAccount, MemoryBackend},
            executor::stack::{
                IsPrecompileResult, MemoryStackState, PrecompileSet, StackExecutor,
                StackSubstateMetadata,
            },
            ExitFatal,
        };

        let source = tx.from.unwrap_or_default();
        let gas_limit = tx.gas_limit.map_or(u64::MAX, |v| v.as_u64());

        let precompiles = env.precompile_set();
        let config = env.config();

        // The precompile check is only used for transactional invocations. However, here we always
        // execute the check, because the check has side effects.
        let gas_limit = match precompiles.is_precompile(source, gas_limit) {
            IsPrecompileResult::Answer { extra_cost, .. } => gas_limit.saturating_sub(extra_cost),
            IsPrecompileResult::OutOfGas => {
                return ExecutionError { reason: ExitError::OutOfGas, gas_used: gas_limit }.into();
            },
        };

        // Only check the restrictions of EIP-3607 if the source of the EVM operation is from an
        // external transaction. If the source of this EVM operation is from an internal
        // call, like from `eth_call` or `eth_estimateGas` RPC, we will skip the checks for
        // the EIP-3607.
        //
        // EIP-3607: https://eips.ethereum.org/EIPS/eip-3607
        // Do not allow transactions for which `tx.sender` has any code deployed.
        // if is_transactional && self.db.accounts.get(&tx.from.unwrap_or_default()).map(|bla|
        // bla.code_hash) { 	return Err(Error::TransactionMustComeFromEOA);
        // }

        // Execute the EVM call.
        let vicinity = sputnik_evm::backend::MemoryVicinity {
            gas_price: U256::zero(),
            origin: tx.from.unwrap_or_default(),
            block_hashes: self.db.blocks_hashes.values().copied().collect(),
            block_number: U256::from(block.number.as_u64()),
            block_coinbase: block.miner.unwrap_or_default(),
            block_timestamp: block.timestamp,
            block_difficulty: block.difficulty,
            block_gas_limit: block.gas_limit,
            chain_id: tx.chain_id.unwrap_or_default(),
            block_base_fee_per_gas: block.base_fee_per_gas.unwrap_or_default(),
            block_randomness: None,
        };

        let mut state = BTreeMap::<Address, MemoryAccount>::new();
        for account in self.db.accounts.values() {
            let entry = state.entry(account.address).or_default();
            entry.nonce = U256::from(account.nonce.as_u64());
            entry.balance = account.balance;

            if let Some(code) = self.db.code.get(&account.code_hash) {
                entry.code = code.0.to_vec();
            }

            if let Some(entries) = self.db.storage.get(&account.address) {
                entry.storage.extend(entries);
            }
        }

        let mut backend = MemoryBackend::new(&vicinity, state);
        let metadata = StackSubstateMetadata::new(gas_limit, config);
        let state = MemoryStackState::new(metadata, &mut backend);
        let precompiles = BTreeMap::new();
        let mut executor = StackExecutor::new_with_precompiles(state, config, &precompiles);

        let (exit_reason, bytes) = executor.transact_call(
            tx.from.unwrap_or_default(),
            tx.to.unwrap_or_default(),
            tx.value.unwrap_or_default(),
            tx.data.as_ref().map(|bytes| bytes.0.to_vec()).unwrap_or_default(),
            tx.gas_limit.unwrap_or(U64::MAX).as_u64(),
            Vec::new(),
        );

        // Clear contract state
        self.db.clear();

        match exit_reason {
            sputnik_evm::ExitReason::Succeed(reason) => ExecutionSucceed {
                reason: ExitSucceed::from(reason),
                gas_used: executor.used_gas(),
                gas_refunded: 0,
                logs: Vec::with_capacity(0),
                output: bytes.into(),
            }
            .into(),
            sputnik_evm::ExitReason::Revert(_) => {
                ExecutionReverted { output: bytes.into(), gas_used: executor.used_gas() }.into()
            },
            sputnik_evm::ExitReason::Error(error) |
            sputnik_evm::ExitReason::Fatal(ExitFatal::CallErrorAsFatal(error)) => {
                ExecutionError { reason: ExitError::from(error), gas_used: executor.used_gas() }
                    .into()
            },
            sputnik_evm::ExitReason::Fatal(ExitFatal::Other(error)) => {
                ExecutionError { reason: ExitError::Other(error), gas_used: executor.used_gas() }
                    .into()
            },
            sputnik_evm::ExitReason::Fatal(ExitFatal::NotSupported) => ExecutionError {
                reason: ExitError::Other("NotSupported".into()),
                gas_used: executor.used_gas(),
            }
            .into(),
            sputnik_evm::ExitReason::Fatal(ExitFatal::UnhandledInterrupt) => ExecutionError {
                reason: ExitError::Other("UnhandledInterrupt".into()),
                gas_used: executor.used_gas(),
            }
            .into(),
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn call(
        &mut self,
        tx: &TransactionCall,
        at: AtBlock,
    ) -> Result<ExecutionResult, Error<RPC::Error>> {
        let prefetch = self.db.prefetch_state(tx, at).await.map_err(Error::PrefetchFailed)?;

        // Execute transaction using VM
        let config = TestEnv::new();
        let vm_result = self.execute(&config, tx, &prefetch.block);

        if vm_result.gas_used() != prefetch.gas_used {
            tracing::warn!(
                "gas used mismatch EVM({}) != RPC({})",
                vm_result.gas_used(),
                prefetch.gas_used
            );
        }

        // Execute transaction using RPC
        let rpc_result = self
            .db
            .rpc()
            .call(tx, AtBlock::At(BlockIdentifier::Hash(prefetch.block.hash)))
            .await;

        // Check if the VM and RPC results matches
        Ok(match (vm_result, rpc_result) {
            // Check if VM and RPC results aren't equals returns the RPC result
            (ExecutionResult::Succeed(mut vm_result), Ok(ExitReason::Succeed(bytes))) => {
                if vm_result.output.ne(&bytes) {
                    tracing::warn!(
                        "result mismatch EVM({:?}) != RPC({:?})",
                        vm_result.output,
                        bytes
                    );
                    vm_result.output = bytes;
                    vm_result.gas_used = prefetch.gas_used;
                }
                ExecutionResult::Succeed(vm_result)
            },
            (ExecutionResult::Revert(mut vm_result), Ok(ExitReason::Revert(bytes))) => {
                if vm_result.output.ne(&bytes) {
                    tracing::warn!(
                        "result mismatch EVM({:?}) != RPC({:?})",
                        vm_result.output,
                        bytes
                    );
                    vm_result.output = bytes;
                    vm_result.gas_used = prefetch.gas_used;
                }
                ExecutionResult::Revert(vm_result)
            },
            (ExecutionResult::Error(mut vm_result), Ok(ExitReason::Error(_))) => {
                vm_result.gas_used = prefetch.gas_used;
                ExecutionResult::Error(vm_result)
            },
            // If the RPC returns an error, we expect the VM to return an revert or error too
            (ExecutionResult::Succeed(_), Err(error)) => {
                tracing::warn!("result mismatch EVM(Success) != RPC(Error)");
                return Err(Error::Rpc(error));
            },
            (vm_result, Ok(ExitReason::Succeed(bytes))) => {
                tracing::warn!("result mismatch EVM({vm_result:?}) != RPC(Success)");
                ExecutionResult::Succeed(ExecutionSucceed {
                    reason: ExitSucceed::Returned,
                    gas_used: prefetch.gas_used,
                    gas_refunded: 0,
                    logs: Vec::with_capacity(0),
                    output: bytes,
                })
            },
            (vm_result, Ok(ExitReason::Revert(bytes))) => {
                tracing::warn!("result mismatch EVM({vm_result:?}) != RPC(Revert)");
                ExecutionResult::Revert(ExecutionReverted {
                    gas_used: prefetch.gas_used,
                    output: bytes,
                })
            },
            (vm_result, Ok(ExitReason::Error(error))) => {
                tracing::warn!("result mismatch EVM({vm_result:?}) != RPC(Error)");
                ExecutionResult::Error(ExecutionError {
                    reason: ExitError::Other(error),
                    gas_used: prefetch.gas_used,
                })
            },
            // if the RPC result is an error, returns the VM result
            (result, Err(_)) => result,
        })
    }
}

impl From<sputnik_evm::ExitSucceed> for ExitSucceed {
    fn from(reason: sputnik_evm::ExitSucceed) -> Self {
        use sputnik_evm::ExitSucceed;
        match reason {
            ExitSucceed::Stopped => Self::Stopped,
            ExitSucceed::Returned => Self::Returned,
            ExitSucceed::Suicided => Self::SelfDestruct,
        }
    }
}

impl From<sputnik_evm::ExitError> for ExitError {
    fn from(reason: sputnik_evm::ExitError) -> Self {
        use sputnik_evm::ExitError;
        match reason {
            ExitError::StackUnderflow => Self::StackUnderflow,
            ExitError::StackOverflow => Self::StackOverflow,
            ExitError::InvalidJump => Self::InvalidJump,
            ExitError::InvalidRange => Self::Other("InvalidRange".into()),
            ExitError::DesignatedInvalid | ExitError::InvalidCode(_) => Self::InvalidCode,
            ExitError::CallTooDeep => Self::CallTooDeep,
            ExitError::CreateCollision => Self::CreateCollision,
            ExitError::CreateContractLimit => Self::CreateContractLimit,
            ExitError::OutOfOffset => Self::OutOfOffset,
            ExitError::OutOfGas => Self::OutOfGas,
            ExitError::OutOfFund => Self::OutOfFund,
            ExitError::PCUnderflow => Self::Other("PCUnderflow".into()),
            ExitError::CreateEmpty => Self::Other("CreateEmpty".into()),
            ExitError::Other(msg) => Self::Other(msg),
            ExitError::MaxNonce => Self::MaxNonce,
        }
    }
}
