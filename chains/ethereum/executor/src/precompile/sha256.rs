use alloc::vec::Vec;

use super::LinearCostPrecompile;
use sputnik_evm::{executor::stack::PrecompileFailure, ExitSucceed};

/// The sha256 precompile.
pub struct Sha256;

impl LinearCostPrecompile for Sha256 {
    const BASE: u64 = 60;
    const WORD: u64 = 12;

    fn execute(input: &[u8], _cost: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        use sha2::Digest;
        let ret: [u8; 32] = sha2::Sha256::digest(input).into();
        Ok((ExitSucceed::Returned, ret.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_utils::test_precompile_test_vectors, Sha256};

    #[test]
    fn process_consensus_tests_for_sha256() {
        let data = include_str!("../../res/common_sha256.json");
        test_precompile_test_vectors::<Sha256>(data);
    }
}
