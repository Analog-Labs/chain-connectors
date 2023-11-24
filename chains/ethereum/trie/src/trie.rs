mod cache;
mod prefixed_db;
mod random_state;
mod recorder;
mod storage_proof;
mod trie_backend;

use crate::{
    hasher::KeccakHasher, node_codec::RlpNodeCodec, rstd::vec::Vec,
    trie_stream::Hash256RlpTrieStream,
};
pub use hash_db;
pub use prefixed_db::{KeySpacedDB, KeySpacedDBMut};
use primitive_types::H256;
use rlp::DecoderError;
pub use trie_db;
use trie_db::{MerkleValue, TrieConfiguration, TrieLayout};

/// Trie layout using extension nodes.
#[derive(Default, Clone)]
pub struct Layout;

impl TrieLayout for Layout {
    const USE_EXTENSION: bool = true;
    const ALLOW_EMPTY: bool = false;
    const MAX_INLINE_VALUE: Option<u32> = None;
    type Hash = KeccakHasher;
    type Codec = RlpNodeCodec<KeccakHasher>;
}

impl TrieConfiguration for Layout {
    fn trie_root<I, A, B>(input: I) -> H256
    where
        I: IntoIterator<Item = (A, B)>,
        A: AsRef<[u8]> + Ord,
        B: AsRef<[u8]>,
    {
        trie_root::trie_root_no_extension::<KeccakHasher, Hash256RlpTrieStream, _, _, _>(
            input,
            Self::MAX_INLINE_VALUE,
        )
    }

    fn trie_root_unhashed<I, A, B>(input: I) -> Vec<u8>
    where
        I: IntoIterator<Item = (A, B)>,
        A: AsRef<[u8]> + Ord,
        B: AsRef<[u8]>,
    {
        trie_root::unhashed_trie_no_extension::<KeccakHasher, Hash256RlpTrieStream, _, _, _>(
            input,
            Self::MAX_INLINE_VALUE,
        )
    }

    fn encode_index(input: u32) -> Vec<u8> {
        rlp::encode(&input).freeze().to_vec()
    }
}

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `NodeCodec`
pub type NodeCodec = RlpNodeCodec<KeccakHasher>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDB`
pub type TrieDB<'db, 'cache> = trie_db::TrieDB<'db, 'cache, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDBBuilder`
pub type TrieDBBuilder<'a, 'cache> = trie_db::TrieDBBuilder<'a, 'cache, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDBMut`
pub type TrieDBMut<'db> = trie_db::TrieDBMut<'db, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDBBuilder`
pub type TrieDBMutBuilder<'db> = trie_db::TrieDBMutBuilder<'db, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `SecTrieDB`
pub type SecTrieDB<'db, 'cache> = trie_db::SecTrieDB<'db, 'cache, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `SecTrieDBMut`
pub type SecTrieDBMut<'db> = trie_db::SecTrieDBMut<'db, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `FatDB`
pub type FatDB<'db, 'cache> = trie_db::FatDB<'db, 'cache, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `FatDBMut`
pub type FatDBMut<'db> = trie_db::FatDBMut<'db, Layout>;

/// Convenience type alias for instantiate Keccak/Rlp-flavoured `Lookup`
pub type Lookup<'db, 'cache, Q> = trie_db::Lookup<'db, 'cache, Layout, Q>;

/// Convenience type alias for Keccak/Rlp flavoured trie errors
pub type TrieError = trie_db::TrieError<H256, DecoderError>;

/// Convenience type alias for Keccak/Rlp flavoured trie results
pub type Result<T> = trie_db::Result<T, H256, DecoderError>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDBIterator`
pub type TrieDBIterator<'a, 'cache> = trie_db::TrieDBIterator<'a, 'cache, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDBKeyIterator`
pub type TrieDBKeyIterator<'a, 'cache> = trie_db::TrieDBKeyIterator<'a, 'cache, Layout>;

/// Convenience type alias for Keccak/Rlp flavoured trie hash
pub type TrieHash = trie_db::TrieHash<Layout>;

/// Convenience type alias for Keccak/Rlp flavoured trie hash
pub type CError = trie_db::CError<Layout>;

// /// Convenience type alias for Keccak/Rlp flavoured trie cache
// pub type TrieCache<'cache> = cache::TrieCache<'cache, RlpNodeCodec<KeccakHasher>>;

/// Reexport from `hash_db`, with genericity set for `Hasher` trait.
/// This uses a noops `KeyFunction` (key addressing must be hashed or using
/// an encoding scheme that avoid key conflict).
pub type MemoryDB =
    memory_db::MemoryDB<KeccakHasher, memory_db::HashKey<KeccakHasher>, trie_db::DBValue>;

/// Reexport from `hash_db`, with genericity set for `Hasher` trait.
/// This uses a `KeyFunction` for prefixing keys internally (avoiding
/// key conflict for non random keys).
pub type PrefixedMemoryDB<H> = memory_db::MemoryDB<H, memory_db::PrefixedKey<H>, trie_db::DBValue>;

pub mod predule {
    pub use hash_db::AsHashDB;
    pub use trie_db::{HashDB, HashDBRef, Trie, TrieIterator, TrieMut};
}

/// Read a value from the trie.
/// # Errors
/// If the trie is corrupted, an error is returned.
pub fn read_trie_value<DB: hash_db::HashDBRef<KeccakHasher, trie_db::DBValue>>(
    db: &DB,
    root: &TrieHash,
    key: &[u8],
    recorder: Option<&mut dyn trie_db::TrieRecorder<TrieHash>>,
    cache: Option<&mut dyn trie_db::TrieCache<<Layout as TrieLayout>::Codec>>,
) -> Result<Option<Vec<u8>>> {
    use trie_db::Trie;
    TrieDBBuilder::new(db, root)
        .with_optional_cache(cache)
        .with_optional_recorder(recorder)
        .build()
        .get(key)
}

/// Read the [`trie_db::MerkleValue`] of the node that is the closest descendant for
/// the provided key.
/// # Errors
/// Returns error if the trie is corrupted, an error is returned.
pub fn read_trie_first_descedant_value<DB>(
    db: &DB,
    root: &TrieHash,
    key: &[u8],
    recorder: Option<&mut dyn trie_db::TrieRecorder<TrieHash>>,
    cache: Option<&mut dyn trie_db::TrieCache<<Layout as TrieLayout>::Codec>>,
) -> Result<Option<MerkleValue<TrieHash>>>
where
    DB: hash_db::HashDBRef<<Layout as TrieLayout>::Hash, trie_db::DBValue>,
{
    use trie_db::Trie;
    TrieDBBuilder::new(db, root)
        .with_optional_cache(cache)
        .with_optional_recorder(recorder)
        .build()
        .lookup_first_descendant(key)
}

/// Read a value from the child trie.
/// # Errors
/// Returns error if the trie is corrupted, an error is returned.
pub fn read_child_trie_value<DB>(
    keyspace: &[u8],
    db: &DB,
    root: &TrieHash,
    key: &[u8],
    recorder: Option<&mut dyn trie_db::TrieRecorder<TrieHash>>,
    cache: Option<&mut dyn trie_db::TrieCache<<Layout as TrieLayout>::Codec>>,
) -> Result<Option<Vec<u8>>>
where
    DB: hash_db::HashDBRef<<Layout as TrieLayout>::Hash, trie_db::DBValue>,
{
    use trie_db::Trie;
    let db = KeySpacedDB::new(db, keyspace);
    #[allow(clippy::redundant_clone)]
    TrieDBBuilder::new(&db, root)
        .with_optional_recorder(recorder)
        .with_optional_cache(cache)
        .build()
        .get(key)
        .map(|x| x.clone())
}

#[cfg(test)]
mod tests {
    use crate::rstd::vec::Vec;
    use hash_db::Hasher;
    use hex_literal::hex;
    use memory_db::{HashKey, MemoryDB};
    use primitive_types::{H160, H256, U256};
    use trie_db::{Trie, TrieMut};

    use super::{KeccakHasher, SecTrieDBMut, TrieDBBuilder, TrieDBMutBuilder};
    use crate::{mem_db::account_db::AccountDBMut, node_codec::HASHED_NULL_NODE};

    type Address = H160;

    #[test]
    fn test_inline_encoding_branch() {
        let mut memdb = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::from_null_node(
            &rlp::NULL_RLP,
            rlp::NULL_RLP.as_ref().into(),
        );
        let mut root = HASHED_NULL_NODE;
        {
            let mut triedbmut = TrieDBMutBuilder::new(&mut memdb, &mut root).build();
            assert!(triedbmut.is_empty());
            triedbmut.insert(b"foo", b"bar").unwrap();
            triedbmut.insert(b"fog", b"b").unwrap();
            triedbmut.insert(b"fot", &vec![0u8; 33][..]).unwrap();
        }
        let t = TrieDBBuilder::new(&memdb, &root).build();
        assert!(t.contains(b"foo").unwrap());
        assert!(t.contains(b"fog").unwrap());
        assert_eq!(t.get(b"foo").unwrap().unwrap(), b"bar".to_vec());
        assert_eq!(t.get(b"fog").unwrap().unwrap(), b"b".to_vec());
        assert_eq!(t.get(b"fot").unwrap().unwrap(), vec![0u8; 33]);
    }

    #[test]
    fn test_inline_encoding_extension() {
        let mut memdb = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::from_null_node(
            &rlp::NULL_RLP,
            rlp::NULL_RLP.as_ref().into(),
        );
        let mut root = HASHED_NULL_NODE;
        {
            let mut triedbmut = TrieDBMutBuilder::new(&mut memdb, &mut root).build();
            triedbmut.insert(b"foo", b"b").unwrap();
            triedbmut.insert(b"fog", b"a").unwrap();
        }
        let t = TrieDBBuilder::new(&memdb, &root).build();
        assert!(t.contains(b"foo").unwrap());
        assert!(t.contains(b"fog").unwrap());
        assert_eq!(t.get(b"foo").unwrap().unwrap(), b"b".to_vec());
        assert_eq!(t.get(b"fog").unwrap().unwrap(), b"a".to_vec());
    }

    #[test]
    fn test_contract_storage_root() {
        let mut root = HASHED_NULL_NODE;
        let mut db = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::from_null_node(
            &rlp::NULL_RLP,
            rlp::NULL_RLP.as_ref().into(),
        );
        let mut db =
            AccountDBMut::from_hash(&mut db, KeccakHasher::hash(Address::zero().as_bytes()));
        {
            let mut triedbmut = SecTrieDBMut::from_existing(&mut db, &mut root);
            let key = H256::zero();
            let value = rlp::encode(&U256::from(0x1234)).freeze();
            triedbmut.insert(key.as_bytes(), value.as_ref()).unwrap();
        }
        assert_eq!(
            root,
            H256(hex!("c57e1afb758b07f8d2c8f13a3b6e44fa5ff94ab266facc5a4fd3f062426e50b2"))
        );
        {
            let mut triedbmut = SecTrieDBMut::from_existing(&mut db, &mut root);
            let key = H256::zero();
            let value = rlp::encode(&U256::from(0x1)).freeze();
            triedbmut.insert(key.as_bytes(), value.as_ref()).unwrap();
        }
        assert_eq!(
            root,
            H256(hex!("821e2556a290c86405f8160a2d662042a431ba456b9db265c79bb837c04be5f0"))
        );
    }
}
