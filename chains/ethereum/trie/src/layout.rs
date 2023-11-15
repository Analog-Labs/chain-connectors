use crate::{keccak::KeccakHasher, node_codec::RlpNodeCodec};
use primitive_types::H256;
use rlp::DecoderError;
use trie_db::TrieLayout;

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

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDB`
pub type TrieDB<'db, 'cache> = trie_db::TrieDB<'db, 'cache, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDBMut`
pub type TrieDBMut<'db> = trie_db::TrieDBMut<'db, Layout>;

/// Convenience type alias to instantiate a Keccak/Rlp-flavoured `TrieDBBuilder`
pub type TrieDBBuilder<'a, 'cache> = trie_db::TrieDBBuilder<'a, 'cache, Layout>;

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

/// Convenience type alias for Keccak/Rlp flavoured trie errors
pub type TrieError = trie_db::TrieError<H256, DecoderError>;

/// Convenience type alias for Keccak/Rlp flavoured trie results
pub type Result<T> = trie_db::Result<T, H256, DecoderError>;

#[cfg(test)]
mod tests {

    // use primitive_types::H256;
    use memory_db::{HashKey, MemoryDB};
    use trie_db::{Trie, TrieMut};

    use super::{KeccakHasher, TrieDBBuilder, TrieDBMutBuilder};
    use crate::node_codec::HASHED_NULL_NODE;

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
}
