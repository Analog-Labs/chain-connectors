pub mod blake2f;
pub mod bn128;
pub mod ecrecover;
pub mod identity;
pub mod modexp;
pub mod ripemd160;
pub mod sha256;

use alloc::vec::Vec;
use blake2f::Blake2F;
use bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use ecrecover::ECRecover;
use identity::Identity;
use modexp::Modexp;
use ripemd160::Ripemd160;
use rosetta_ethereum_primitives::Address;
use sha256::Sha256;
use sputnik_evm::{
    executor::stack::{
        IsPrecompileResult, PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileSet,
    },
    ExitError, ExitSucceed,
};

pub type PrecompileResult = Result<PrecompileOutput, PrecompileFailure>;

/// One single precompile used by EVM engine.
pub trait Precompile {
    /// Try to execute the precompile with given `handle` which provides all call data
    /// and allow to register costs and logs.
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult;
}

pub trait LinearCostPrecompile {
    const BASE: u64;
    const WORD: u64;

    fn execute(
        input: &[u8],
        cost: u64,
    ) -> core::result::Result<(ExitSucceed, Vec<u8>), PrecompileFailure>;
}

impl<T: LinearCostPrecompile> Precompile for T {
    fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let target_gas = handle.gas_limit();
        let cost = ensure_linear_cost(target_gas, handle.input().len() as u64, T::BASE, T::WORD)?;

        handle.record_cost(cost)?;
        let (exit_status, output) = T::execute(handle.input(), cost)?;
        Ok(PrecompileOutput { exit_status, output })
    }
}

/// Linear gas cost
fn ensure_linear_cost(
    target_gas: Option<u64>,
    len: u64,
    base: u64,
    word: u64,
) -> Result<u64, PrecompileFailure> {
    let cost = base
        .checked_add(
            word.checked_mul(len.saturating_add(31) / 32)
                .ok_or(PrecompileFailure::Error { exit_status: ExitError::OutOfGas })?,
        )
        .ok_or(PrecompileFailure::Error { exit_status: ExitError::OutOfGas })?;

    if let Some(target_gas) = target_gas {
        if cost > target_gas {
            return Err(PrecompileFailure::Error { exit_status: ExitError::OutOfGas });
        }
    }

    Ok(cost)
}

pub struct DefaultPrecompileSet;

impl PrecompileSet for DefaultPrecompileSet {
    fn execute(
        &self,
        handle: &mut impl PrecompileHandle,
    ) -> Option<Result<PrecompileOutput, PrecompileFailure>> {
        let addr = handle.code_address();
        if addr > Address::from_low_u64_ne(9) {
            return None;
        }
        let lsb = addr.0[19];
        match lsb {
            1 => Some(<ECRecover as Precompile>::execute(handle)),
            2 => Some(<Sha256 as Precompile>::execute(handle)),
            3 => Some(<Ripemd160 as Precompile>::execute(handle)),
            4 => Some(<Identity as Precompile>::execute(handle)),
            5 => Some(Modexp::execute(handle)),
            6 => Some(Bn128Add::execute(handle)),
            7 => Some(Bn128Mul::execute(handle)),
            8 => Some(Bn128Pairing::execute(handle)),
            9 => Some(Blake2F::execute(handle)),
            _ => None,
        }
    }

    fn is_precompile(&self, address: Address, _remaining_gas: u64) -> IsPrecompileResult {
        IsPrecompileResult::Answer {
            is_precompile: !address.is_zero() && address < Address::from_low_u64_ne(10),
            extra_cost: 0,
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    extern crate hex;
    use super::Precompile;
    use rosetta_ethereum_primitives::{Address, H256, U256};
    use sputnik_evm::{
        executor::stack::PrecompileHandle, Context, ExitError, ExitReason, ExitSucceed, Transfer,
    };

    pub struct MockHandle {
        pub input: Vec<u8>,
        pub gas_limit: Option<u64>,
        pub context: Context,
        pub is_static: bool,
        pub gas_used: u64,
    }

    impl MockHandle {
        pub fn new(input: Vec<u8>, gas_limit: Option<u64>, context: Context) -> Self {
            Self { input, gas_limit, context, is_static: false, gas_used: 0 }
        }
    }

    impl PrecompileHandle for MockHandle {
        /// Perform subcall in provided context.
        /// Precompile specifies in which context the subcall is executed.
        fn call(
            &mut self,
            _: Address,
            _: Option<Transfer>,
            _: Vec<u8>,
            _: Option<u64>,
            _: bool,
            _: &Context,
        ) -> (ExitReason, Vec<u8>) {
            unimplemented!()
        }

        fn record_cost(&mut self, cost: u64) -> Result<(), ExitError> {
            self.gas_used += cost;
            Ok(())
        }

        fn record_external_cost(
            &mut self,
            _: Option<u64>,
            _: Option<u64>,
            _: Option<u64>,
        ) -> Result<(), ExitError> {
            Ok(())
        }

        fn refund_external_cost(&mut self, _: Option<u64>, _: Option<u64>) {}

        fn log(&mut self, _: Address, _: Vec<H256>, _: Vec<u8>) -> Result<(), ExitError> {
            unimplemented!()
        }

        fn remaining_gas(&self) -> u64 {
            unimplemented!()
        }

        fn code_address(&self) -> Address {
            unimplemented!()
        }

        fn input(&self) -> &[u8] {
            &self.input
        }

        fn context(&self) -> &Context {
            &self.context
        }

        fn is_static(&self) -> bool {
            self.is_static
        }

        fn gas_limit(&self) -> Option<u64> {
            self.gas_limit
        }
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct EthConsensusTest {
        input: String,
        expected: String,
        name: String,
        gas: Option<u64>,
    }

    /// Tests a precompile against the ethereum consensus tests defined in the given file at
    /// filepath. The file is expected to be in JSON format and contain an array of test
    /// vectors, where each vector can be deserialized into an `EthConsensusTest`.
    pub fn test_precompile_test_vectors<P: Precompile>(json: &'static str) {
        let tests: Vec<EthConsensusTest> = serde_json::from_str(json).expect("expected json array");
        for test in tests {
            let input: Vec<u8> =
                hex::decode(test.input).expect("Could not hex-decode test input data");
            let cost: u64 = 10_000_000;
            let context: Context = Context {
                address: Address::zero(),
                caller: Address::zero(),
                apparent_value: U256::zero(),
            };

            let mut handle = MockHandle::new(input, Some(cost), context);
            let result = P::execute(&mut handle)
                .unwrap_or_else(|err| panic!("Test '{}' returned error: {:?}", test.name, err));
            let as_hex: String = hex::encode(result.output);
            assert_eq!(
                result.exit_status,
                ExitSucceed::Returned,
                "test '{}' returned {:?} (expected 'Returned')",
                test.name,
                result.exit_status
            );
            assert_eq!(as_hex, test.expected, "test '{}' failed (different output)", test.name);
            if let Some(expected_gas) = test.gas {
                assert_eq!(
                    handle.gas_used, expected_gas,
                    "test '{}' failed (different gas cost)",
                    test.name
                );
            }
        }
    }
}
