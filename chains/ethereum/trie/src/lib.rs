#![cfg_attr(not(feature = "std"), no_std)]

pub mod db;
pub mod keccak;
pub mod layout;
pub mod node_codec;
pub mod trie_stream;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
mod rstd {
    pub use std::{
        borrow, boxed, cmp, collections::BTreeMap, convert, error::Error, fmt, hash, iter, marker,
        mem, ops, rc, result, sync, vec,
    };
}

#[cfg(not(feature = "std"))]
mod rstd {
    pub use alloc::{borrow, boxed, collections::BTreeMap, rc, sync, vec};
    pub use core::{cmp, convert, fmt, hash, iter, marker, mem, ops, result};
    pub trait Error {}
    impl<T> Error for T {}
}

use rstd::{convert::AsRef, iter::IntoIterator};

use keccak::KeccakHasher;
use primitive_types::H256;
use trie_stream::Hash256RlpTrieStream;

/// Generates a trie root hash for a vector of key-value tuples
pub fn trie_root<I, V>(input: I) -> H256
where
    I: IntoIterator<Item = V>,
    V: AsRef<[u8]>,
{
    trie_root::trie_root::<KeccakHasher, Hash256RlpTrieStream, _, _, _>(
        input.into_iter().enumerate().map(|(i, v)| (rlp::encode(&i), v)),
        None,
    )
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
