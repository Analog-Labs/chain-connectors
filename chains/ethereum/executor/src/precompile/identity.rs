use alloc::vec::Vec;

use super::LinearCostPrecompile;
use sputnik_evm::{executor::stack::PrecompileFailure, ExitSucceed};

/// The identity precompile.
pub struct Identity;

impl LinearCostPrecompile for Identity {
    const BASE: u64 = 15;
    const WORD: u64 = 3;

    fn execute(input: &[u8], _: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        Ok((ExitSucceed::Returned, input.to_vec()))
    }
}
