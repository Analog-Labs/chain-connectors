use alloc::vec::Vec;

use super::LinearCostPrecompile;
use sputnik_evm::{executor::stack::PrecompileFailure, ExitSucceed};

/// The ripemd precompile.
pub struct Ripemd160;

impl LinearCostPrecompile for Ripemd160 {
    const BASE: u64 = 600;
    const WORD: u64 = 120;

    fn execute(input: &[u8], _cost: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        use ripemd::Digest;

        let mut ret = [0u8; 32];
        ret[12..32].copy_from_slice(&ripemd::Ripemd160::digest(input));
        Ok((ExitSucceed::Returned, ret.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_utils::test_precompile_test_vectors, Ripemd160};

    // TODO: this fails on the test "InvalidHighV-bits-1" where it is expected to return ""
    #[test]
    fn process_consensus_tests_for_ecrecover() {
        let data = include_str!("../res/common_ripemd.json");
        test_precompile_test_vectors::<Ripemd160>(data);
    }
}
