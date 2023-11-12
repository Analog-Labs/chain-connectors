use alloc::{
    fmt::{Debug, Display},
    vec::Vec,
};

use crate::{
    state::{PrefetchError, StateDB},
    types::{
        ExecutionError, ExecutionResult, ExecutionReverted, ExecutionSucceed, ExitError,
        ExitSucceed, Log,
    },
};
use revm::{evm_inner, inspectors::NoOpInspector};
use rosetta_ethereum_backend::{AtBlock, EthereumRpc, ExitReason};
use rosetta_ethereum_primitives::{rpc::CallRequest, Address, Block, BlockIdentifier, Bytes, H256};

pub type EvmError = revm::primitives::EVMError<StateError>;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum StateError {
    #[cfg_attr(feature = "std", error("code not found: {0}"))]
    CodeNotFound(H256),
    #[cfg_attr(feature = "std", error("storage not found for address {0} at index {1}"))]
    StorageNotFound(Address, H256),
    #[cfg_attr(feature = "std", error("block not found: {0}"))]
    BlockNotFound(u64),
}

impl<T> revm::Database for StateDB<T>
where
    T: EthereumRpc + Send + Sync,
{
    type Error = StateError;

    fn basic(
        &mut self,
        address: revm::primitives::Address,
    ) -> Result<Option<revm::primitives::AccountInfo>, Self::Error> {
        let address = Address::from(address.into_array());
        let account = self.accounts.get(&address).map(|account| {
            let code = self.code.get(&account.code_hash).cloned().map(|code| {
                revm::primitives::Bytecode::new_raw(revm::primitives::Bytes::from(code.0))
            });
            revm::primitives::AccountInfo {
                balance: revm::primitives::U256::from_limbs(account.balance.0),
                nonce: account.nonce,
                code_hash: revm::primitives::B256::from(account.code_hash.0),
                code,
            }
        });
        Ok(account)
    }

    fn code_by_hash(
        &mut self,
        code_hash: revm::primitives::B256,
    ) -> Result<revm::primitives::Bytecode, Self::Error> {
        let code_hash = H256::from(code_hash.0);
        let Some(code) = self.code.get(&code_hash).cloned() else {
            tracing::warn!("code_by_hash: {code_hash:?} => ???");
            return Err(StateError::CodeNotFound(code_hash));
        };
        Ok(revm::primitives::Bytecode::new_raw(revm::primitives::Bytes::from(code.0)))
    }

    fn storage(
        &mut self,
        address: revm::primitives::Address,
        index: revm::primitives::U256,
    ) -> Result<revm::primitives::U256, Self::Error> {
        let address = Address::from(address.into_array());
        let index = H256::from(index.to_be_bytes());
        let Some(contract_storage) = self.storage.get(&address) else {
            tracing::warn!("Contract not found {address:?}");
            return Err(StateError::StorageNotFound(address, index));
        };
        let Some(value) = contract_storage.get(&index).copied() else {
            tracing::warn!("Storage not found {address:?} {index:?}");
            return Err(StateError::StorageNotFound(address, index));
        };
        Ok(revm::primitives::U256::from_be_bytes(value.0))
    }

    fn block_hash(
        &mut self,
        number: revm::primitives::U256,
    ) -> Result<revm::primitives::B256, Self::Error> {
        let Ok(number) = u64::try_from(number) else {
            tracing::warn!("block_hash: BlockNotFound {number}");
            return Err(StateError::BlockNotFound(u64::MAX));
        };
        let Some(block_hash) = self.blocks_hashes.get(&number).copied() else {
            tracing::warn!("block_hash: BlockNotFound {number}");
            return Err(StateError::BlockNotFound(number));
        };
        Ok(revm::primitives::B256::from(block_hash.0))
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum Error<ERR>
where
    ERR: Display,
{
    #[cfg_attr(feature = "std", error("RPC error: {0:?}"))]
    Rpc(ERR),
    #[cfg_attr(feature = "std", error("Prefetch failed: {0:?}"))]
    PrefetchFailed(PrefetchError<ERR>),
    #[cfg_attr(feature = "std", error("EVM error: {0:?}"))]
    EvmError(EvmError),
}

impl<E: Display> From<EvmError> for Error<E> {
    fn from(error: revm::primitives::EVMError<StateError>) -> Self {
        Self::EvmError(error)
    }
}

pub struct Executor<RPC: EthereumRpc + Send + Sync> {
    chain_id: u64,
    db: StateDB<RPC>,
}

impl<RPC> Executor<RPC>
where
    RPC: EthereumRpc + Send + Sync,
{
    #[allow(clippy::missing_errors_doc)]
    pub async fn new(client: RPC) -> Result<Self, Error<RPC::Error>> {
        let chain_id = client.chain_id().await.map_err(Error::Rpc)?.as_u64();
        let db = StateDB::new(client);
        Ok(Self { chain_id, db })
    }

    // Execute transaction using VM
    #[allow(clippy::missing_errors_doc)]
    pub fn execute(
        &mut self,
        tx: &CallRequest,
        block: &Block<H256>,
    ) -> Result<ExecutionResult, EvmError> {
        let mut env = revm::primitives::Env::default();

        // Configure chain environment
        env.cfg.spec_id = revm::primitives::SpecId::SHANGHAI;
        // Necessary for CHAINID opcode
        // [EIP-1344]: https://eips.ethereum.org/EIPS/eip-1344
        env.cfg.chain_id = self.chain_id;
        // Disable base fee to allow method calls with zero gas price
        env.cfg.disable_base_fee = true;
        // Allow method calls with gas limit that's higher than block's gas limit
        env.cfg.disable_block_gas_limit = true;

        // Configure block
        env.block.number = revm::primitives::U256::from(block.header.number);
        env.block.coinbase = revm::primitives::Address::from(block.header.beneficiary.0);
        env.block.timestamp = revm::primitives::U256::from(block.header.timestamp);
        env.block.difficulty = revm::primitives::U256::from_limbs(block.header.difficulty.0);
        env.block.basefee =
            revm::primitives::U256::from(block.header.base_fee_per_gas.unwrap_or_default());
        env.block.gas_limit = revm::primitives::U256::from(block.header.gas_limit);
        env.block.prevrandao = Some(revm::primitives::B256::ZERO);
        env.block.blob_excess_gas_and_price = Some(revm::primitives::BlobExcessGasAndPrice::new(0));

        // Configure transaction
        env.tx.transact_to = revm::primitives::TransactTo::Call(revm::primitives::Address::from(
            tx.to.unwrap_or_default().0,
        ));
        env.tx.caller = revm::primitives::Address::from(tx.from.unwrap_or_default().0);
        env.tx.value = revm::primitives::U256::from_limbs(tx.value.unwrap_or_default().0);
        env.tx.data = revm::primitives::Bytes::from(tx.data.clone().unwrap_or_default().0);
        env.tx.gas_limit = tx.gas_limit.unwrap_or(u64::MAX);
        env.tx.gas_price = tx
            .gas_price
            .map(|v| revm::primitives::U256::from_limbs(v.0))
            .unwrap_or_default();
        env.tx.chain_id = tx.chain_id;
        env.tx.max_fee_per_blob_gas =
            tx.max_fee_per_gas.map(|v| revm::primitives::U256::from_limbs(v.0));
        env.tx.gas_priority_fee =
            tx.max_priority_fee_per_gas.map(|v| revm::primitives::U256::from_limbs(v.0));

        // Execute transaction
        let vm_result =
            evm_inner::<StateDB<RPC>, false>(&mut env, &mut self.db, &mut NoOpInspector)
                .transact()
                .map(|result| result.result);

        // Clear Contract State
        self.db.clear();

        Ok(ExecutionResult::from(vm_result?))
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn call(
        &mut self,
        tx: &CallRequest,
        at: AtBlock,
    ) -> Result<ExecutionResult, Error<RPC::Error>> {
        let prefetch = self.db.prefetch_state(tx, at).await.map_err(Error::PrefetchFailed)?;

        // Execute transaction using VM
        let vm_result = self.execute(tx, &prefetch.block)?;

        // Execute transaction using RPC
        let rpc_result = self
            .db
            .rpc()
            .call(tx, AtBlock::At(BlockIdentifier::Hash(prefetch.block.hash)))
            .await;

        if vm_result.gas_used() != prefetch.gas_used {
            tracing::warn!(
                "gas used mismatch EVM({}) != RPC({})",
                vm_result.gas_used(),
                prefetch.gas_used
            );
        }

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

impl From<revm::primitives::Halt> for ExitError {
    fn from(halt: revm::primitives::Halt) -> Self {
        use revm::primitives::Halt;
        match halt {
            Halt::OutOfGas(_) => Self::OutOfGas,
            Halt::OpcodeNotFound | Halt::InvalidFEOpcode | Halt::CreateContractStartingWithEF => {
                Self::InvalidCode
            },
            Halt::InvalidJump => Self::InvalidJump,
            Halt::NotActivated => Self::Other("NotActivated".into()),
            Halt::StackUnderflow => Self::StackUnderflow,
            Halt::StackOverflow => Self::StackOverflow,
            Halt::OutOfOffset => Self::OutOfOffset,
            Halt::CreateCollision => Self::CreateCollision,
            Halt::PrecompileError => Self::Other("PrecompileError".into()),
            Halt::NonceOverflow => Self::MaxNonce,
            Halt::CreateContractSizeLimit | Halt::CreateInitcodeSizeLimit => {
                Self::CreateContractLimit
            },
            Halt::OverflowPayment => Self::Other("OverflowPayment".into()),
            Halt::StateChangeDuringStaticCall => Self::Other("StateChangeDuringStaticCall".into()),
            Halt::CallNotAllowedInsideStatic => Self::Other("CallNotAllowedInsideStatic".into()),
            Halt::OutOfFund => Self::OutOfFund,
            Halt::CallTooDeep => Self::CallTooDeep,
        }
    }
}

impl From<revm::primitives::Log> for Log {
    fn from(log: revm::primitives::Log) -> Self {
        Self {
            address: Address::from(log.address.into_array()),
            topics: log.topics.into_iter().map(|hash| H256::from(hash.0)).collect(),
            data: Bytes::from(log.data.0),
        }
    }
}

impl From<revm::primitives::Eval> for crate::types::ExitSucceed {
    fn from(eval: revm::primitives::Eval) -> Self {
        match eval {
            revm::primitives::Eval::Stop => Self::Stopped,
            revm::primitives::Eval::Return => Self::Returned,
            revm::primitives::Eval::SelfDestruct => Self::SelfDestruct,
        }
    }
}

impl From<revm::primitives::ExecutionResult> for ExecutionResult {
    fn from(result: revm::primitives::ExecutionResult) -> Self {
        use revm::primitives::Output;
        match result {
            revm::primitives::ExecutionResult::Success {
                reason,
                gas_used,
                gas_refunded,
                logs,
                output,
            } => {
                let output = match output {
                    Output::Call(bytes) | Output::Create(bytes, _) => Bytes::from(bytes.0),
                };
                Self::Succeed(ExecutionSucceed {
                    reason: ExitSucceed::from(reason),
                    gas_used,
                    gas_refunded,
                    logs: logs.into_iter().map(Into::into).collect(),
                    output,
                })
            },
            revm::primitives::ExecutionResult::Revert { gas_used, output } => {
                Self::Revert(ExecutionReverted { gas_used, output: Bytes(output.0) })
            },
            revm::primitives::ExecutionResult::Halt { reason, gas_used } => {
                Self::Error(ExecutionError { reason: ExitError::from(reason), gas_used })
            },
        }
    }
}
