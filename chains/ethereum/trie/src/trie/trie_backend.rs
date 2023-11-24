use super::{
    cache::{LocalTrieCache, TrieCache},
    NodeCodec,
};
use crate::hasher::KeccakHasher;
use hash_db::Hasher;
use trie_db::TrieCache as TrieCacheT;

/// A provider of trie caches that are compatible with [`trie_db::TrieDB`].
pub trait TrieCacheProvider {
    /// Cache type that implements [`trie_db::TrieCache`].
    type Cache<'a>: TrieCacheT<NodeCodec> + 'a
    where
        Self: 'a;

    /// Return a [`trie_db::TrieDB`] compatible cache.
    ///
    /// The `storage_root` parameter *must* be the storage root of the trie this cache is used for.
    ///
    /// NOTE: Implementors should use the `storage_root` to differentiate between storage keys that
    /// may belong to different tries.
    fn as_trie_db_cache(&self, storage_root: <KeccakHasher as Hasher>::Out) -> Self::Cache<'_>;

    /// Returns a cache that can be used with a [`trie_db::TrieDBMut`].
    ///
    /// When finished with the operation on the trie, it is required to call [`Self::merge`] to
    /// merge the cached items for the correct `storage_root`.
    fn as_trie_db_mut_cache(&self) -> Self::Cache<'_>;

    /// Merge the cached data in `other` into the provider using the given `new_root`.
    ///
    /// This must be used for the cache returned by [`Self::as_trie_db_mut_cache`] as otherwise the
    /// cached data is just thrown away.
    fn merge<'a>(&'a self, other: Self::Cache<'a>, new_root: <KeccakHasher as Hasher>::Out);
}

impl TrieCacheProvider for LocalTrieCache<KeccakHasher> {
    type Cache<'a> = TrieCache<'a, KeccakHasher> where Self: 'a;

    fn as_trie_db_cache(&self, storage_root: <KeccakHasher as Hasher>::Out) -> Self::Cache<'_> {
        Self::as_trie_db_cache(self, storage_root)
    }

    fn as_trie_db_mut_cache(&self) -> Self::Cache<'_> {
        Self::as_trie_db_mut_cache(self)
    }

    fn merge<'a>(&'a self, other: Self::Cache<'a>, new_root: <KeccakHasher as Hasher>::Out) {
        other.merge_into(self, new_root);
    }
}
