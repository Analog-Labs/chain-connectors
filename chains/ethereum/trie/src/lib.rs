#![cfg_attr(not(feature = "std"), no_std)]

pub mod hasher;
// pub mod layout;
#[cfg(any(test, feature = "memory-db"))]
pub mod mem_db;
pub mod node_codec;
pub mod trie;
pub mod trie_stream;

#[cfg(not(feature = "std"))]
#[cfg_attr(test, macro_use)]
extern crate alloc;

#[cfg(feature = "std")]
mod rstd {
    pub mod jose {
        pub use spin::Mutex;
    }
    pub mod collections {
        pub use std::collections::{btree_map, btree_set};
    }
    pub use std::{
        borrow, boxed, cmp, convert, default, error::Error, fmt, hash, iter, marker, mem, ops, rc,
        result, time, vec,
    };
    pub mod sync {
        pub use std::sync::Arc;
        pub mod atomic {
            pub use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
        }
        pub use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
    }
}

#[cfg(not(feature = "std"))]
mod rstd {
    pub mod collections {
        pub use alloc::collections::{btree_map, btree_set};
    }
    pub mod sync {
        pub use alloc::sync::Arc;
        pub use spin::{
            lock_api::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard},
            Once,
        };
        pub mod atomic {
            pub use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
        }
    }
    pub use alloc::{borrow, boxed, rc, vec};
    pub use core::{cmp, convert, default, fmt, hash, iter, marker, mem, ops, result, time};
    pub trait Error {}
    impl<T> Error for T {}
}

use rstd::{convert::AsRef, iter::IntoIterator};

use hasher::KeccakHasher;
use primitive_types::H256;
use trie_stream::Hash256RlpTrieStream;

/// Generates a trie root hash for a vector of key-value tuples
pub fn trie_root<I, K, V>(input: I) -> H256
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<[u8]> + Ord,
    V: AsRef<[u8]>,
{
    trie_root::trie_root::<KeccakHasher, Hash256RlpTrieStream, _, _, _>(input, None)
}

/// Generates a key-hashed (secure) trie root hash for a vector of key-value tuples.
pub fn sec_trie_root<I, K, V>(input: I) -> H256
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    trie_root::sec_trie_root::<KeccakHasher, Hash256RlpTrieStream, _, _, _>(input, None)
}

/// Generates a trie root hash for a vector of values
pub fn ordered_trie_root<I, V>(input: I) -> H256
where
    I: IntoIterator<Item = V>,
    V: AsRef<[u8]>,
{
    trie_root::trie_root::<KeccakHasher, Hash256RlpTrieStream, _, _, _>(
        input.into_iter().enumerate().map(|(i, v)| (rlp::encode(&i), v)),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::{ordered_trie_root, rstd::vec, sec_trie_root, trie_root, H256};
    use hex_literal::hex;

    #[test]
    fn simple_test() {
        let expected =
            H256(hex!("d23786fb4a010da3ce639d66d5e904a11dbc02746d1ce25029e53290cabf28ab"));
        let actual =
            trie_root(vec![(b"A", b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" as &[u8])]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_trie_root() {
        let v = vec![("doe", "reindeer"), ("dog", "puppy"), ("dogglesworth", "cat")];
        let expected =
            H256(hex!("8aad789dff2f538bca5d8ea56e8abe10f4c7ba3a5dea95fea4cd6e7c3a1168d3"));
        let actual = trie_root::<_, _, _>(v);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_sec_trie_root() {
        let v = vec![("doe", "reindeer"), ("dog", "puppy"), ("dogglesworth", "cat")];
        let expected =
            H256(hex!("d4cd937e4a4368d7931a9cf51686b7e10abb3dce38a39000fd7902a092b64585"));
        let actual = sec_trie_root::<_, _, _>(v);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_ordered_trie_root() {
        let v = &["doe", "reindeer"];
        let expected =
            H256(hex!("e766d5d51b89dc39d981b41bda63248d7abce4f0225eefd023792a540bcffee3"));
        let actual = ordered_trie_root::<_, _>(v);
        assert_eq!(actual, expected);
    }
}
