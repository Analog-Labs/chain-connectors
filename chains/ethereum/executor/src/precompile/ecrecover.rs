use alloc::vec::Vec;
use core::cmp::min;

use super::LinearCostPrecompile;
use libsecp256k1::{Message, RecoveryId, Signature};
use sputnik_evm::{executor::stack::PrecompileFailure, ExitSucceed};

/// The ecrecover precompile.
pub struct ECRecover;

impl LinearCostPrecompile for ECRecover {
    const BASE: u64 = 3000;
    const WORD: u64 = 0;

    fn execute(i: &[u8], _: u64) -> Result<(ExitSucceed, Vec<u8>), PrecompileFailure> {
        let mut input = [0u8; 128];
        input[..min(i.len(), 128)].copy_from_slice(&i[..min(i.len(), 128)]);

        // v can only be 27 or 28 on the full 32 bytes value.
        // https://github.com/ethereum/go-ethereum/blob/a907d7e81aaeea15d80b2d3209ad8e08e3bf49e0/core/vm/contracts.go#L177
        if input[32..63] != [0u8; 31] || ![27, 28].contains(&input[63]) {
            return Ok((ExitSucceed::Returned, [0u8; 0].to_vec()));
        }

        let mut msg = [0u8; 32];
        let mut sig = [0u8; 65];

        msg[0..32].copy_from_slice(&input[0..32]);
        sig[0..32].copy_from_slice(&input[64..96]); // r
        sig[32..64].copy_from_slice(&input[96..128]); // s
        sig[64] = input[63]; // v

        let result = ec_recover(&msg, &sig).unwrap_or_default();
        Ok((ExitSucceed::Returned, result))
    }
}

fn ec_recover(msg: &[u8; 32], sig: &[u8; 65]) -> Result<Vec<u8>, libsecp256k1::Error> {
    use sha3::Digest;

    let rid = RecoveryId::parse(sig[64] - 27)?;
    let sig = Signature::parse_overflowing_slice(&sig[0..64])?;
    let msg = Message::parse(msg);
    let pubkey = libsecp256k1::recover(&msg, &sig, &rid)?;
    // uncompress the key
    let uncompressed = pubkey.serialize();
    let mut hash: [u8; 32] = sha3::Keccak256::digest(&uncompressed[1..]).into();
    hash[0..12].copy_from_slice(&[0u8; 12]);
    Ok(hash.to_vec())
}

#[cfg(test)]
mod tests {
    use super::{super::test_utils::test_precompile_test_vectors, ECRecover};

    // TODO: this fails on the test "InvalidHighV-bits-1" where it is expected to return ""
    #[test]
    fn process_consensus_tests_for_ecrecover() {
        let data = include_str!("../../res/ecRecover.json");
        test_precompile_test_vectors::<ECRecover>(data);
    }
}
