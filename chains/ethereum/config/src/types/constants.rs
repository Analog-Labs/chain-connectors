use ethereum_types::H256;
use hex_literal::hex;

/// Keccak256 over empty array.
pub const KECCAK_EMPTY: H256 =
    H256(hex!("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"));

/// Ommer root of empty list.
pub const EMPTY_OMMER_ROOT_HASH: H256 =
    H256(hex!("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"));

/// Root hash of an empty trie.
pub const EMPTY_ROOT_HASH: H256 =
    H256(hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"));
