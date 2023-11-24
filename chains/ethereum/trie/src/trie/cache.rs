#![allow(dead_code)]
use super::{random_state::RandomState, NodeCodec, Result as TrieResult};
use crate::{
    hasher::KeccakHasher,
    rstd::{
        fmt::Debug,
        hash::{BuildHasherDefault, Hash as StdHash, Hasher as StdHasher},
        mem,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc, Mutex, MutexGuard, RwLock, RwLockWriteGuard,
        },
    },
};
use hash_db::Hasher;
use hashbrown::{hash_set::Entry as SetEntry, HashMap, HashSet};
use nohash_hasher::BuildNoHashHasher;
use primitive_types::H256;
use schnellru::{Limiter, LruMap};
use trie_db::{node::NodeOwned, CachedValue};

#[cfg(test)]
use crate::rstd::sync::RwLockReadGuard;

const LOG_TARGET: &str = "rosetta-ethereum-trie";

/// The maximum number of existing keys in the shared cache that a single local cache
/// can promote to the front of the LRU cache in one go.
///
/// If we have a big shared cache and the local cache hits all of those keys we don't
/// want to spend forever bumping all of them.
const SHARED_NODE_CACHE_MAX_PROMOTED_KEYS: u32 = 1792;
/// Same as [`SHARED_NODE_CACHE_MAX_PROMOTED_KEYS`].
const SHARED_VALUE_CACHE_MAX_PROMOTED_KEYS: u32 = 1792;

/// The maximum portion of the shared cache (in percent) that a single local
/// cache can replace in one go.
///
/// We don't want a single local cache instance to have the ability to replace
/// everything in the shared cache.
const SHARED_NODE_CACHE_MAX_REPLACE_PERCENT: usize = 33;
/// Same as [`SHARED_NODE_CACHE_MAX_REPLACE_PERCENT`].
const SHARED_VALUE_CACHE_MAX_REPLACE_PERCENT: usize = 33;

/// The maximum inline capacity of the local cache, in bytes.
///
/// This is just an upper limit; since the maps are resized in powers of two
/// their actual size will most likely not exactly match this.
const LOCAL_NODE_CACHE_MAX_INLINE_SIZE: usize = 512 * 1024;
/// Same as [`LOCAL_NODE_CACHE_MAX_INLINE_SIZE`].
const LOCAL_VALUE_CACHE_MAX_INLINE_SIZE: usize = 512 * 1024;

/// The maximum size of the memory allocated on the heap by the local cache, in bytes.
///
/// The size of the node cache should always be bigger than the value cache. The value
/// cache is only holding weak references to the actual values found in the nodes and
/// we account for the size of the node as part of the node cache.
const LOCAL_NODE_CACHE_MAX_HEAP_SIZE: usize = 8 * 1024 * 1024;
/// Same as [`LOCAL_NODE_CACHE_MAX_HEAP_SIZE`].
const LOCAL_VALUE_CACHE_MAX_HEAP_SIZE: usize = 2 * 1024 * 1024;

/// The size of the shared cache.
#[derive(Debug, Clone, Copy)]
pub struct CacheSize(usize);

impl CacheSize {
    /// An unlimited cache size.
    pub const fn unlimited() -> Self {
        Self(usize::MAX)
    }

    /// A cache size `bytes` big.
    pub const fn new(bytes: usize) -> Self {
        Self(bytes)
    }
}

/// An internal struct to store the cached trie nodes.
pub struct NodeCached<H> {
    /// The cached node.
    pub node: NodeOwned<H>,
    /// Whether this node was fetched from the shared cache or not.
    pub is_from_shared_cache: bool,
}

impl<H> NodeCached<H> {
    /// Returns the number of bytes allocated on the heap by this node.
    fn heap_size(&self) -> usize {
        self.node.size_in_bytes() - mem::size_of::<NodeOwned<H>>()
    }
}

pub struct SharedNodeCacheLimiter {
    /// The maximum size (in bytes) the cache can hold inline.
    ///
    /// This space is always consumed whether there are any items in the map or not.
    max_inline_size: usize,

    /// The maximum size (in bytes) the cache can hold on the heap.
    max_heap_size: usize,

    /// The current size (in bytes) of data allocated by this cache on the heap.
    ///
    /// This doesn't include the size of the map itself.
    heap_size: usize,

    /// A counter with the number of elements that got evicted from the cache.
    ///
    /// Reset to zero before every update.
    items_evicted: usize,

    /// The maximum number of elements that we allow to be evicted.
    ///
    /// Reset on every update.
    max_items_evicted: usize,
}

impl<H> Limiter<H, NodeOwned<H>> for SharedNodeCacheLimiter
where
    H: AsRef<[u8]>,
{
    type KeyToInsert<'a> = H;
    type LinkType = u32;

    #[inline]
    fn is_over_the_limit(&self, _length: usize) -> bool {
        // Once we hit the limit of max items evicted this will return `false` and prevent
        // any further evictions, but this is fine because the outer loop which inserts
        // items into this cache will just detect this and stop inserting new items.
        self.items_evicted <= self.max_items_evicted && self.heap_size > self.max_heap_size
    }

    #[inline]
    fn on_insert(
        &mut self,
        _length: usize,
        key: Self::KeyToInsert<'_>,
        node: NodeOwned<H>,
    ) -> Option<(H, NodeOwned<H>)> {
        let new_item_heap_size = node.size_in_bytes() - mem::size_of::<NodeOwned<H>>();
        if new_item_heap_size > self.max_heap_size {
            // Item's too big to add even if the cache's empty; bail.
            return None;
        }

        self.heap_size += new_item_heap_size;
        Some((key, node))
    }

    #[inline]
    fn on_replace(
        &mut self,
        _length: usize,
        old_key: &mut H,
        new_key: H,
        old_node: &mut NodeOwned<H>,
        new_node: &mut NodeOwned<H>,
    ) -> bool {
        debug_assert_eq!(old_key.as_ref(), new_key.as_ref());

        let new_item_heap_size = new_node.size_in_bytes() - mem::size_of::<NodeOwned<H>>();
        if new_item_heap_size > self.max_heap_size {
            // Item's too big to add even if the cache's empty; bail.
            return false;
        }

        let old_item_heap_size = old_node.size_in_bytes() - mem::size_of::<NodeOwned<H>>();
        self.heap_size = self.heap_size - old_item_heap_size + new_item_heap_size;
        true
    }

    #[inline]
    fn on_cleared(&mut self) {
        self.heap_size = 0;
    }

    #[inline]
    fn on_removed(&mut self, _: &mut H, node: &mut NodeOwned<H>) {
        self.heap_size -= node.size_in_bytes() - mem::size_of::<NodeOwned<H>>();
        self.items_evicted += 1;
    }

    #[inline]
    fn on_grow(&mut self, new_memory_usage: usize) -> bool {
        new_memory_usage <= self.max_inline_size
    }
}

pub struct SharedValueCacheLimiter {
    /// The maximum size (in bytes) the cache can hold inline.
    ///
    /// This space is always consumed whether there are any items in the map or not.
    max_inline_size: usize,

    /// The maximum size (in bytes) the cache can hold on the heap.
    max_heap_size: usize,

    /// The current size (in bytes) of data allocated by this cache on the heap.
    ///
    /// This doesn't include the size of the map itself.
    heap_size: usize,

    /// A set with all of the keys deduplicated to save on memory.
    known_storage_keys: HashSet<Arc<[u8]>>,

    /// A counter with the number of elements that got evicted from the cache.
    ///
    /// Reset to zero before every update.
    items_evicted: usize,

    /// The maximum number of elements that we allow to be evicted.
    ///
    /// Reset on every update.
    max_items_evicted: usize,
}

impl<H> Limiter<ValueCacheKey<H>, CachedValue<H>> for SharedValueCacheLimiter
where
    H: AsRef<[u8]>,
{
    type KeyToInsert<'a> = ValueCacheKey<H>;
    type LinkType = u32;

    #[inline]
    fn is_over_the_limit(&self, _length: usize) -> bool {
        self.items_evicted <= self.max_items_evicted && self.heap_size > self.max_heap_size
    }

    #[inline]
    fn on_insert(
        &mut self,
        _length: usize,
        mut key: Self::KeyToInsert<'_>,
        value: CachedValue<H>,
    ) -> Option<(ValueCacheKey<H>, CachedValue<H>)> {
        match self.known_storage_keys.entry(key.storage_key.clone()) {
            SetEntry::Vacant(entry) => {
                let new_item_heap_size = key.storage_key.len();
                if new_item_heap_size > self.max_heap_size {
                    // Item's too big to add even if the cache's empty; bail.
                    return None;
                }

                self.heap_size += new_item_heap_size;
                entry.insert();
            },
            SetEntry::Occupied(entry) => {
                key.storage_key = entry.get().clone();
            },
        }

        Some((key, value))
    }

    #[inline]
    fn on_replace(
        &mut self,
        _length: usize,
        old_key: &mut ValueCacheKey<H>,
        new_key: ValueCacheKey<H>,
        _old_value: &mut CachedValue<H>,
        _new_value: &mut CachedValue<H>,
    ) -> bool {
        debug_assert_eq!(new_key.storage_key, old_key.storage_key);
        true
    }

    #[inline]
    fn on_removed(&mut self, key: &mut ValueCacheKey<H>, _: &mut CachedValue<H>) {
        if Arc::strong_count(&key.storage_key) == 2 {
            // There are only two instances of this key:
            //   1) one memoized in `known_storage_keys`,
            //   2) one inside the map.
            //
            // This means that after this remove goes through the `Arc` will be deallocated.
            self.heap_size -= key.storage_key.len();
            self.known_storage_keys.remove(&key.storage_key);
        }
        self.items_evicted += 1;
    }

    #[inline]
    fn on_cleared(&mut self) {
        self.heap_size = 0;
        self.known_storage_keys.clear();
    }

    #[inline]
    fn on_grow(&mut self, new_memory_usage: usize) -> bool {
        new_memory_usage <= self.max_inline_size
    }
}

type SharedNodeCacheMap<H> = LruMap<H, NodeOwned<H>, SharedNodeCacheLimiter, RandomState>;

/// The shared node cache.
///
/// Internally this stores all cached nodes in a [`LruMap`]. It ensures that when updating the
/// cache, that the cache stays within its allowed bounds.
pub(super) struct SharedNodeCache<H>
where
    H: AsRef<[u8]>,
{
    /// The cached nodes, ordered by least recently used.
    pub(super) lru: SharedNodeCacheMap<H>,
}

impl<H: AsRef<[u8]> + Eq + StdHash> SharedNodeCache<H> {
    /// Create a new instance.
    fn new(max_inline_size: usize, max_heap_size: usize) -> Self {
        Self {
            lru: LruMap::with_hasher(
                SharedNodeCacheLimiter {
                    max_inline_size,
                    max_heap_size,
                    heap_size: 0,
                    items_evicted: 0,
                    max_items_evicted: 0, // Will be set during `update`.
                },
                RandomState::default(),
            ),
        }
    }

    /// Update the cache with the `list` of nodes which were either newly added or accessed.
    pub fn update(&mut self, list: impl IntoIterator<Item = (H, NodeCached<H>)>) {
        let mut access_count = 0;
        let mut add_count = 0;

        self.lru.limiter_mut().items_evicted = 0;
        self.lru.limiter_mut().max_items_evicted =
            self.lru.len() * 100 / SHARED_NODE_CACHE_MAX_REPLACE_PERCENT;

        for (key, cached_node) in list {
            if cached_node.is_from_shared_cache && self.lru.get(&key).is_some() {
                access_count += 1;

                if access_count >= SHARED_NODE_CACHE_MAX_PROMOTED_KEYS {
                    // Stop when we've promoted a large enough number of items.
                    break;
                }

                continue;
            }

            self.lru.insert(key, cached_node.node);
            add_count += 1;

            if self.lru.limiter().items_evicted > self.lru.limiter().max_items_evicted {
                // Stop when we've evicted a big enough chunk of the shared cache.
                break;
            }
        }

        tracing::debug!(
            target: LOG_TARGET,
            "Updated the shared node cache: {} accesses, {} new values, {}/{} evicted (length = {}, inline size={}/{}, heap size={}/{})",
            access_count,
            add_count,
            self.lru.limiter().items_evicted,
            self.lru.limiter().max_items_evicted,
            self.lru.len(),
            self.lru.memory_usage(),
            self.lru.limiter().max_inline_size,
            self.lru.limiter().heap_size,
            self.lru.limiter().max_heap_size,
        );
    }

    /// Reset the cache.
    fn reset(&mut self) {
        self.lru.clear();
    }
}

/// The hash of [`ValueCacheKey`].
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
#[repr(transparent)]
pub struct ValueCacheKeyHash(u64);

impl ValueCacheKeyHash {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl ValueCacheKeyHash {
    pub fn from_hasher_and_storage_key(mut hasher: impl StdHasher, storage_key: &[u8]) -> Self {
        hasher.write(storage_key);
        Self(hasher.finish())
    }
}

impl nohash_hasher::IsEnabled for ValueCacheKeyHash {}

/// The key type that is being used to address a [`CachedValue`].
#[derive(Eq)]
pub(super) struct ValueCacheKey<H> {
    /// The storage root of the trie this key belongs to.
    pub storage_root: H,
    /// The key to access the value in the storage.
    pub storage_key: Arc<[u8]>,
    /// The hash that identifies this instance of `storage_root` and `storage_key`.
    pub hash: ValueCacheKeyHash,
}

/// A borrowed variant of [`ValueCacheKey`].
pub(super) struct ValueCacheRef<'a, H> {
    /// The storage root of the trie this key belongs to.
    pub storage_root: H,
    /// The key to access the value in the storage.
    pub storage_key: &'a [u8],
    /// The hash that identifies this instance of `storage_root` and `storage_key`.
    pub hash: ValueCacheKeyHash,
}

impl<'a, H> ValueCacheRef<'a, H> {
    pub fn new(storage_key: &'a [u8], storage_root: H) -> Self
    where
        H: AsRef<[u8]>,
    {
        let hash = ValueCacheKey::<H>::hash_data(storage_key, &storage_root);
        Self { storage_root, storage_key, hash }
    }
}

impl<'a, H> From<ValueCacheRef<'a, H>> for ValueCacheKey<H> {
    fn from(value: ValueCacheRef<'a, H>) -> Self {
        Self {
            storage_root: value.storage_root,
            storage_key: value.storage_key.into(),
            hash: value.hash,
        }
    }
}

impl<'a, H: StdHash> StdHash for ValueCacheRef<'a, H> {
    fn hash<Hasher: StdHasher>(&self, state: &mut Hasher) {
        self.hash.hash(state);
    }
}

impl<'a, H> PartialEq<ValueCacheKey<H>> for ValueCacheRef<'a, H>
where
    H: AsRef<[u8]>,
{
    fn eq(&self, rhs: &ValueCacheKey<H>) -> bool {
        self.storage_root.as_ref() == rhs.storage_root.as_ref() &&
            self.storage_key == &*rhs.storage_key
    }
}

impl<H> ValueCacheKey<H> {
    /// Constructs [`Self::Value`].
    #[cfg(test)] // Only used in tests.
    pub fn new_value(storage_key: impl Into<Arc<[u8]>>, storage_root: H) -> Self
    where
        H: AsRef<[u8]>,
    {
        let storage_key = storage_key.into();
        let hash = Self::hash_data(&storage_key, &storage_root);
        Self { storage_root, storage_key, hash }
    }

    /// Returns a hasher prepared to build the final hash to identify [`Self`].
    ///
    /// See [`Self::hash_data`] for building the hash directly.
    pub fn hash_partial_data(storage_root: &H) -> impl StdHasher + Clone
    where
        H: AsRef<[u8]>,
    {
        let storage_root = storage_root.as_ref();
        let mut hasher = RandomState::global_build_hasher();
        hasher.write(storage_root);
        hasher
    }

    /// Hash the `key` and `storage_root` that identify [`Self`].
    ///
    /// Returns a `u64` which represents the unique hash for the given inputs.
    pub fn hash_data(key: &[u8], storage_root: &H) -> ValueCacheKeyHash
    where
        H: AsRef<[u8]>,
    {
        let hasher = Self::hash_partial_data(storage_root);

        ValueCacheKeyHash::from_hasher_and_storage_key(hasher, key)
    }

    /// Checks whether the key is equal to the given `storage_key` and `storage_root`.
    #[inline]
    pub fn is_eq(&self, storage_root: &H, storage_key: &[u8]) -> bool
    where
        H: PartialEq,
    {
        self.storage_root == *storage_root && *self.storage_key == *storage_key
    }
}

// Implement manually so that only `hash` is accessed.
impl<H: StdHash> StdHash for ValueCacheKey<H> {
    fn hash<Hasher: StdHasher>(&self, state: &mut Hasher) {
        self.hash.hash(state);
    }
}

impl<H> nohash_hasher::IsEnabled for ValueCacheKey<H> {}

// Implement manually to not have to compare `hash`.
impl<H: PartialEq> PartialEq for ValueCacheKey<H> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.is_eq(&other.storage_root, &other.storage_key)
    }
}

type SharedValueCacheMap<H> = schnellru::LruMap<
    ValueCacheKey<H>,
    CachedValue<H>,
    SharedValueCacheLimiter,
    BuildNoHashHasher<ValueCacheKey<H>>,
>;

/// The shared value cache.
///
/// The cache ensures that it stays in the configured size bounds.
pub(super) struct SharedValueCache<H>
where
    H: AsRef<[u8]>,
{
    /// The cached nodes, ordered by least recently used.
    pub(super) lru: SharedValueCacheMap<H>,
}

impl<H: Eq + StdHash + Clone + Copy + AsRef<[u8]>> SharedValueCache<H> {
    /// Create a new instance.
    fn new(max_inline_size: usize, max_heap_size: usize) -> Self {
        Self {
            lru: schnellru::LruMap::with_hasher(
                SharedValueCacheLimiter {
                    max_inline_size,
                    max_heap_size,
                    heap_size: 0,
                    known_storage_keys: HashSet::default(),
                    items_evicted: 0,
                    max_items_evicted: 0, // Will be set during `update`.
                },
                BuildNoHashHasher::default(),
            ),
        }
    }

    /// Update the cache with the `added` values and the `accessed` values.
    ///
    /// The `added` values are the ones that have been collected by doing operations on the trie and
    /// now should be stored in the shared cache. The `accessed` values are only referenced by the
    /// [`ValueCacheKeyHash`] and represent the values that were retrieved from this shared cache.
    /// These `accessed` values are being put to the front of the internal [`LruMap`] like the
    /// `added` ones.
    pub fn update(
        &mut self,
        added: impl IntoIterator<Item = (ValueCacheKey<H>, CachedValue<H>)>,
        accessed: impl IntoIterator<Item = ValueCacheKeyHash>,
    ) {
        let mut access_count = 0;
        let mut add_count = 0;

        for hash in accessed {
            // Access every node in the map to put it to the front.
            //
            // Since we are only comparing the hashes here it may lead us to promoting the wrong
            // values as the most recently accessed ones. However this is harmless as the only
            // consequence is that we may accidentally prune a recently used value too early.
            self.lru.get_by_hash(hash.raw(), |existing_key, _| existing_key.hash == hash);
            access_count += 1;
        }

        // Insert all of the new items which were *not* found in the shared cache.
        //
        // Limit how many items we'll replace in the shared cache in one go so that
        // we don't evict the whole shared cache nor we keep spinning our wheels
        // evicting items which we've added ourselves in previous iterations of this loop.

        self.lru.limiter_mut().items_evicted = 0;
        self.lru.limiter_mut().max_items_evicted =
            self.lru.len() * 100 / SHARED_VALUE_CACHE_MAX_REPLACE_PERCENT;

        for (key, value) in added {
            self.lru.insert(key, value);
            add_count += 1;

            if self.lru.limiter().items_evicted > self.lru.limiter().max_items_evicted {
                // Stop when we've evicted a big enough chunk of the shared cache.
                break;
            }
        }

        tracing::debug!(
            target: LOG_TARGET,
            "Updated the shared value cache: {} accesses, {} new values, {}/{} evicted (length = {}, known_storage_keys = {}, inline size={}/{}, heap size={}/{})",
            access_count,
            add_count,
            self.lru.limiter().items_evicted,
            self.lru.limiter().max_items_evicted,
            self.lru.len(),
            self.lru.limiter().known_storage_keys.len(),
            self.lru.memory_usage(),
            self.lru.limiter().max_inline_size,
            self.lru.limiter().heap_size,
            self.lru.limiter().max_heap_size
        );
    }

    /// Reset the cache.
    fn reset(&mut self) {
        self.lru.clear();
    }
}

/// The inner of [`SharedTrieCache`].
pub(super) struct SharedTrieCacheInner<H: Hasher> {
    node_cache: SharedNodeCache<H::Out>,
    value_cache: SharedValueCache<H::Out>,
}

impl<H: Hasher> Clone for SharedTrieCache<H> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<H: Hasher> SharedTrieCacheInner<H> {
    /// Returns a reference to the [`SharedValueCache`].
    #[cfg(test)]
    pub const fn value_cache(&self) -> &SharedValueCache<H::Out> {
        &self.value_cache
    }

    /// Returns a mutable reference to the [`SharedValueCache`].
    pub(super) fn value_cache_mut(&mut self) -> &mut SharedValueCache<H::Out> {
        &mut self.value_cache
    }

    /// Returns a reference to the [`SharedNodeCache`].
    #[cfg(test)]
    pub const fn node_cache(&self) -> &SharedNodeCache<H::Out> {
        &self.node_cache
    }

    /// Returns a mutable reference to the [`SharedNodeCache`].
    pub(super) fn node_cache_mut(&mut self) -> &mut SharedNodeCache<H::Out> {
        &mut self.node_cache
    }
}

/// The shared trie cache.
///
/// It should be instantiated once per node. It will hold the trie nodes and values of all
/// operations to the state. To not use all available memory it will ensure to stay in the
/// bounds given via the [`CacheSize`] at startup.
///
/// The instance of this object can be shared between multiple threads.
pub struct SharedTrieCache<H: Hasher> {
    inner: Arc<RwLock<SharedTrieCacheInner<H>>>,
}

impl<H: Hasher> SharedTrieCache<H> {
    /// Create a new [`SharedTrieCache`].
    pub fn new(cache_size: CacheSize) -> Self {
        let total_budget = cache_size.0;

        // Split our memory budget between the two types of caches.
        let value_cache_budget = total_budget / 5; // 20% for the value cache
        let node_cache_budget = total_budget - value_cache_budget; // 80% for the node cache

        // Split our memory budget between what we'll be holding inline in the map,
        // and what we'll be holding on the heap.
        let value_cache_inline_budget = (value_cache_budget * 7) / 10; // 70%
        let node_cache_inline_budget = (node_cache_budget * 7) / 10; // 70%

        // Calculate how much memory the maps will be allowed to hold inline given our budget.
        let value_cache_max_inline_size =
            SharedValueCacheMap::<H::Out>::memory_usage_for_memory_budget(
                value_cache_inline_budget,
            );

        let node_cache_max_inline_size =
            SharedNodeCacheMap::<H::Out>::memory_usage_for_memory_budget(node_cache_inline_budget);

        // And this is how much data we'll at most keep on the heap for each cache.
        let value_cache_max_heap_size = value_cache_budget - value_cache_max_inline_size;
        let node_cache_max_heap_size = node_cache_budget - node_cache_max_inline_size;

        tracing::debug!(
            target: LOG_TARGET,
            "Configured a shared trie cache with a budget of ~{} bytes (node_cache_max_inline_size = {}, node_cache_max_heap_size = {}, value_cache_max_inline_size = {}, value_cache_max_heap_size = {})",
            total_budget,
            node_cache_max_inline_size,
            node_cache_max_heap_size,
            value_cache_max_inline_size,
            value_cache_max_heap_size,
        );

        Self {
            inner: Arc::new(RwLock::new(SharedTrieCacheInner {
                node_cache: SharedNodeCache::new(
                    node_cache_max_inline_size,
                    node_cache_max_heap_size,
                ),
                value_cache: SharedValueCache::new(
                    value_cache_max_inline_size,
                    value_cache_max_heap_size,
                ),
            })),
        }
    }

    /// Create a new [`LocalTrieCache`](super::LocalTrieCache) instance from this shared cache.
    pub fn local_cache(&self) -> LocalTrieCache<H> {
        LocalTrieCache {
            shared: self.clone(),
            node_cache: Mutex::default(),
            value_cache: Mutex::default(),
            shared_value_cache_access: Mutex::new(ValueAccessSet::with_hasher(
                schnellru::ByLength::new(SHARED_VALUE_CACHE_MAX_PROMOTED_KEYS),
                BuildHasherDefault::default(),
            )),
            stats: TrieHitStats::default(),
        }
    }

    /// Get a copy of the node for `key`.
    ///
    /// This will temporarily lock the shared cache for reading.
    ///
    /// This doesn't change the least recently order in the internal [`LruMap`].
    #[inline]
    pub fn peek_node(&self, key: &H::Out) -> Option<NodeOwned<H::Out>> {
        self.inner.read().node_cache.lru.peek(key).cloned()
    }

    /// Get a copy of the [`CachedValue`] for `key`.
    ///
    /// This will temporarily lock the shared cache for reading.
    ///
    /// This doesn't reorder any of the elements in the internal [`LruMap`].
    pub fn peek_value_by_hash(
        &self,
        hash: ValueCacheKeyHash,
        storage_root: &H::Out,
        storage_key: &[u8],
    ) -> Option<CachedValue<H::Out>> {
        self.inner
            .read()
            .value_cache
            .lru
            .peek_by_hash(hash.0, |existing_key, _| existing_key.is_eq(storage_root, storage_key))
            .cloned()
    }

    /// Returns the used memory size of this cache in bytes.
    pub fn used_memory_size(&self) -> usize {
        let inner = self.inner.read();
        let value_cache_size =
            inner.value_cache.lru.memory_usage() + inner.value_cache.lru.limiter().heap_size;
        let node_cache_size =
            inner.node_cache.lru.memory_usage() + inner.node_cache.lru.limiter().heap_size;
        drop(inner);
        node_cache_size + value_cache_size
    }

    /// Reset the node cache.
    pub fn reset_node_cache(&self) {
        self.inner.write().node_cache.reset();
    }

    /// Reset the value cache.
    pub fn reset_value_cache(&self) {
        self.inner.write().value_cache.reset();
    }

    /// Reset the entire cache.
    pub fn reset(&self) {
        self.reset_node_cache();
        self.reset_value_cache();
    }

    /// Returns the read locked inner.
    #[cfg(test)]
    pub(super) fn read_lock_inner(&self) -> RwLockReadGuard<'_, SharedTrieCacheInner<H>> {
        self.inner.read()
    }

    /// Returns the write locked inner.
    pub(super) fn write_lock_inner(&self) -> Option<RwLockWriteGuard<'_, SharedTrieCacheInner<H>>> {
        // This should never happen, but we *really* don't want to deadlock. So let's have it
        // timeout, just in case. At worst it'll do nothing, and at best it'll avert a catastrophe
        // and notify us that there's a problem.
        #[cfg(feature = "std")]
        {
            use crate::rstd::time::Duration;
            /// The maximum amount of time we'll wait trying to acquire the shared cache lock
            /// when the local cache is dropped and synchronized with the share cache.
            ///
            /// This is just a failsafe; normally this should never trigger.
            const SHARED_CACHE_WRITE_LOCK_TIMEOUT: Duration = Duration::from_millis(100);
            self.inner.try_write_for(SHARED_CACHE_WRITE_LOCK_TIMEOUT)
        }
        #[cfg(not(feature = "std"))]
        {
            self.inner.try_write()
        }
    }
}

// ------------- //

/// A limiter for the local node cache. This makes sure the local cache doesn't grow too big.
#[derive(Default)]
pub struct LocalNodeCacheLimiter {
    /// The current size (in bytes) of data allocated by this cache on the heap.
    ///
    /// This doesn't include the size of the map itself.
    current_heap_size: usize,
}

impl<H> schnellru::Limiter<H, NodeCached<H>> for LocalNodeCacheLimiter
where
    H: AsRef<[u8]>,
    // H: AsRef<[u8]> + Debug,
{
    type KeyToInsert<'a> = H;
    type LinkType = u32;

    #[inline]
    fn is_over_the_limit(&self, length: usize) -> bool {
        // Only enforce the limit if there's more than one element to make sure
        // we can always add a new element to the cache.
        if length <= 1 {
            return false;
        }

        self.current_heap_size > LOCAL_NODE_CACHE_MAX_HEAP_SIZE
    }

    #[inline]
    fn on_insert<'a>(
        &mut self,
        _length: usize,
        key: H,
        cached_node: NodeCached<H>,
    ) -> Option<(H, NodeCached<H>)> {
        self.current_heap_size += cached_node.heap_size();
        Some((key, cached_node))
    }

    #[inline]
    fn on_replace(
        &mut self,
        _length: usize,
        old_key: &mut H,
        new_key: H,
        old_node: &mut NodeCached<H>,
        new_node: &mut NodeCached<H>,
    ) -> bool {
        debug_assert_eq!(old_key.as_ref().len(), new_key.as_ref().len());
        self.current_heap_size =
            self.current_heap_size + new_node.heap_size() - old_node.heap_size();
        true
    }

    #[inline]
    fn on_removed(&mut self, _key: &mut H, cached_node: &mut NodeCached<H>) {
        self.current_heap_size -= cached_node.heap_size();
    }

    #[inline]
    fn on_cleared(&mut self) {
        self.current_heap_size = 0;
    }

    #[inline]
    fn on_grow(&mut self, new_memory_usage: usize) -> bool {
        new_memory_usage <= LOCAL_NODE_CACHE_MAX_INLINE_SIZE
    }
}

/// A limiter for the local value cache. This makes sure the local cache doesn't grow too big.
#[derive(Default)]
pub struct LocalValueCacheLimiter {
    /// The current size (in bytes) of data allocated by this cache on the heap.
    ///
    /// This doesn't include the size of the map itself.
    current_heap_size: usize,
}

impl<H> schnellru::Limiter<ValueCacheKey<H>, CachedValue<H>> for LocalValueCacheLimiter
where
    H: AsRef<[u8]>,
{
    type KeyToInsert<'a> = ValueCacheRef<'a, H>;
    type LinkType = u32;

    #[inline]
    fn is_over_the_limit(&self, length: usize) -> bool {
        // Only enforce the limit if there's more than one element to make sure
        // we can always add a new element to the cache.
        if length <= 1 {
            return false;
        }

        self.current_heap_size > LOCAL_VALUE_CACHE_MAX_HEAP_SIZE
    }

    #[inline]
    fn on_insert(
        &mut self,
        _length: usize,
        key: Self::KeyToInsert<'_>,
        value: CachedValue<H>,
    ) -> Option<(ValueCacheKey<H>, CachedValue<H>)> {
        self.current_heap_size += key.storage_key.len();
        Some((key.into(), value))
    }

    #[inline]
    fn on_replace(
        &mut self,
        _length: usize,
        old_key: &mut ValueCacheKey<H>,
        new_key: ValueCacheRef<H>,
        _old_value: &mut CachedValue<H>,
        _new_value: &mut CachedValue<H>,
    ) -> bool {
        debug_assert_eq!(old_key.storage_key.len(), new_key.storage_key.len());
        true
    }

    #[inline]
    fn on_removed(&mut self, key: &mut ValueCacheKey<H>, _: &mut CachedValue<H>) {
        self.current_heap_size -= key.storage_key.len();
    }

    #[inline]
    fn on_cleared(&mut self) {
        self.current_heap_size = 0;
    }

    #[inline]
    fn on_grow(&mut self, new_memory_usage: usize) -> bool {
        new_memory_usage <= LOCAL_VALUE_CACHE_MAX_INLINE_SIZE
    }
}

/// A struct to gather hit/miss stats to aid in debugging the performance of the cache.
#[derive(Debug, Default)]
struct HitStats {
    shared_hits: AtomicU64,
    shared_fetch_attempts: AtomicU64,
    local_hits: AtomicU64,
    local_fetch_attempts: AtomicU64,
}

#[cfg(feature = "std")]
impl crate::rstd::fmt::Display for HitStats {
    fn fmt(&self, fmt: &mut crate::rstd::fmt::Formatter) -> crate::rstd::fmt::Result {
        let shared_hits = self.shared_hits.load(Ordering::Relaxed);
        let shared_fetch_attempts = self.shared_fetch_attempts.load(Ordering::Relaxed);
        let local_hits = self.local_hits.load(Ordering::Relaxed);
        let local_fetch_attempts = self.local_fetch_attempts.load(Ordering::Relaxed);
        if shared_fetch_attempts == 0 && local_hits == 0 {
            write!(fmt, "empty")
        } else {
            let percent_local = (local_hits * 100) / local_fetch_attempts;
            let percent_shared = (shared_hits * 100) / shared_fetch_attempts;
            write!(
				fmt,
				"local hit rate = {percent_local}% [{local_hits}/{local_fetch_attempts}], shared hit rate = {percent_shared}% [{shared_hits}/{shared_fetch_attempts}]",
			)
        }
    }
}

/// A struct to gather hit/miss stats for the node cache and the value cache.
#[derive(Debug, Default)]
struct TrieHitStats {
    node_cache: HitStats,
    value_cache: HitStats,
}

type NodeCacheMap<H> = LruMap<H, NodeCached<H>, LocalNodeCacheLimiter, RandomState>;

type ValueCacheMap<H> = LruMap<
    ValueCacheKey<H>,
    CachedValue<H>,
    LocalValueCacheLimiter,
    BuildNoHashHasher<ValueCacheKey<H>>,
>;

type ValueAccessSet =
    LruMap<ValueCacheKeyHash, (), schnellru::ByLength, BuildNoHashHasher<ValueCacheKeyHash>>;

/// The local trie cache.
///
/// This cache should be used per state instance created by the backend. One state instance is
/// referring to the state of one block. It will cache all the accesses that are done to the state
/// which could not be fullfilled by the [`SharedTrieCache`]. These locally cached items are merged
/// back to the shared trie cache when this instance is dropped.
///
/// When using [`Self::as_trie_db_cache`] or [`Self::as_trie_db_mut_cache`], it will lock Mutexes.
/// So, it is important that these methods are not called multiple times, because they otherwise
/// deadlock.
pub struct LocalTrieCache<H: Hasher> {
    /// The shared trie cache that created this instance.
    shared: SharedTrieCache<H>,

    /// The local cache for the trie nodes.
    node_cache: Mutex<NodeCacheMap<H::Out>>,

    /// The local cache for the values.
    value_cache: Mutex<ValueCacheMap<H::Out>>,

    /// Keeps track of all values accessed in the shared cache.
    ///
    /// This will be used to ensure that these nodes are brought to the front of the lru when this
    /// local instance is merged back to the shared cache. This can actually lead to collision when
    /// two [`ValueCacheKey`]s with different storage roots and keys map to the same hash. However,
    /// as we only use this set to update the lru position it is fine, even if we bring the wrong
    /// value to the top. The important part is that we always get the correct value from the value
    /// cache for a given key.
    shared_value_cache_access: Mutex<ValueAccessSet>,

    stats: TrieHitStats,
}

impl<H: Hasher> LocalTrieCache<H> {
    /// Return self as a [`TrieDB`](trie_db::TrieDB) compatible cache.
    ///
    /// The given `storage_root` needs to be the storage root of the trie this cache is used for.
    pub fn as_trie_db_cache(&self, storage_root: H::Out) -> TrieCache<'_, H> {
        let value_cache = ValueCache::ForStorageRoot {
            storage_root,
            local_value_cache: self.value_cache.lock(),
            shared_value_cache_access: self.shared_value_cache_access.lock(),
            buffered_value: None,
        };

        TrieCache {
            shared_cache: self.shared.clone(),
            local_cache: self.node_cache.lock(),
            value_cache,
            stats: &self.stats,
        }
    }

    /// Return self as [`TrieDBMut`](trie_db::TrieDBMut) compatible cache.
    ///
    /// After finishing all operations with [`TrieDBMut`](trie_db::TrieDBMut) and having obtained
    /// the new storage root, [`TrieCache::merge_into`] should be called to update this local
    /// cache instance. If the function is not called, cached data is just thrown away and not
    /// propagated to the shared cache. So, accessing these new items will be slower, but nothing
    /// would break because of this.
    pub fn as_trie_db_mut_cache(&self) -> TrieCache<'_, H> {
        TrieCache {
            shared_cache: self.shared.clone(),
            local_cache: self.node_cache.lock(),
            value_cache: ValueCache::Fresh(HashMap::default()),
            stats: &self.stats,
        }
    }
}

impl<H: Hasher> Drop for LocalTrieCache<H> {
    fn drop(&mut self) {
        tracing::debug!(
            target: LOG_TARGET,
            "Local node trie cache dropped: {:?}",
            self.stats.node_cache
        );

        tracing::debug!(
            target: LOG_TARGET,
            "Local value trie cache dropped: {:?}",
            self.stats.value_cache
        );

        let Some(mut shared_inner) = self.shared.write_lock_inner() else {
            tracing::warn!(
                target: LOG_TARGET,
                "Timeout while trying to acquire a write lock for the shared trie cache"
            );
            return;
        };

        shared_inner.node_cache_mut().update(self.node_cache.get_mut().drain());
        shared_inner.value_cache_mut().update(
            self.value_cache.get_mut().drain(),
            self.shared_value_cache_access.get_mut().drain().map(|(key, ())| key),
        );
    }
}

/// The abstraction of the value cache for the [`TrieCache`].
enum ValueCache<'a, H: Hasher> {
    /// The value cache is fresh, aka not yet associated to any storage root.
    /// This is used for example when a new trie is being build, to cache new values.
    Fresh(HashMap<Arc<[u8]>, CachedValue<H::Out>>),
    /// The value cache is already bound to a specific storage root.
    ForStorageRoot {
        shared_value_cache_access: MutexGuard<'a, ValueAccessSet>,
        local_value_cache: MutexGuard<'a, ValueCacheMap<H::Out>>,
        storage_root: H::Out,
        // The shared value cache needs to be temporarily locked when reading from it
        // so we need to clone the value that is returned, but we need to be able to
        // return a reference to the value, so we just buffer it here.
        buffered_value: Option<CachedValue<H::Out>>,
    },
}

impl<H: Hasher> ValueCache<'_, H> {
    /// Get the value for the given `key`.
    fn get(
        &mut self,
        key: &[u8],
        shared_cache: &SharedTrieCache<H>,
        stats: &HitStats,
    ) -> Option<&CachedValue<H::Out>> {
        stats.local_fetch_attempts.fetch_add(1, Ordering::Relaxed);

        match self {
            Self::Fresh(map) => map.get(key).map(|value| {
                stats.local_hits.fetch_add(1, Ordering::Relaxed);
                value
            }),
            Self::ForStorageRoot {
                local_value_cache,
                shared_value_cache_access,
                storage_root,
                buffered_value,
            } => {
                // We first need to look up in the local cache and then the shared cache.
                // It can happen that some value is cached in the shared cache, but the
                // weak reference of the data can not be upgraded anymore. This for example
                // happens when the node is dropped that contains the strong reference to the data.
                //
                // So, the logic of the trie would lookup the data and the node and store both
                // in our local caches.

                let hash = ValueCacheKey::hash_data(key, storage_root);

                if let Some(value) = local_value_cache
                    .peek_by_hash(hash.raw(), |existing_key, _| {
                        existing_key.is_eq(storage_root, key)
                    })
                {
                    stats.local_hits.fetch_add(1, Ordering::Relaxed);

                    return Some(value);
                }

                stats.shared_fetch_attempts.fetch_add(1, Ordering::Relaxed);
                if let Some(value) = shared_cache.peek_value_by_hash(hash, storage_root, key) {
                    stats.shared_hits.fetch_add(1, Ordering::Relaxed);
                    shared_value_cache_access.insert(hash, ());
                    #[allow(clippy::redundant_clone)]
                    {
                        // Clone here is important for update the LRUs.
                        *buffered_value = Some(value.clone());
                    }
                    return buffered_value.as_ref();
                }

                None
            },
        }
    }

    /// Insert some new `value` under the given `key`.
    fn insert(&mut self, key: &[u8], value: CachedValue<H::Out>) {
        match self {
            Self::Fresh(map) => {
                map.insert(key.into(), value);
            },
            Self::ForStorageRoot { local_value_cache, storage_root, .. } => {
                local_value_cache.insert(ValueCacheRef::new(key, *storage_root), value);
            },
        }
    }
}

/// The actual [`TrieCache`](trie_db::TrieCache) implementation.
///
/// If this instance was created for using it with a [`TrieDBMut`](trie_db::TrieDBMut), it needs to
/// be merged back into the [`LocalTrieCache`] with [`Self::merge_into`] after all operations are
/// done.
pub struct TrieCache<'a, H: Hasher> {
    shared_cache: SharedTrieCache<H>,
    local_cache: MutexGuard<'a, NodeCacheMap<H::Out>>,
    value_cache: ValueCache<'a, H>,
    stats: &'a TrieHitStats,
}

impl<'a, H: Hasher> TrieCache<'a, H> {
    /// Merge this cache into the given [`LocalTrieCache`].
    ///
    /// This function is only required to be called when this instance was created through
    /// [`LocalTrieCache::as_trie_db_mut_cache`], otherwise this method is a no-op. The given
    /// `storage_root` is the new storage root that was obtained after finishing all operations
    /// using the [`TrieDBMut`](trie_db::TrieDBMut).
    pub fn merge_into(self, local: &LocalTrieCache<H>, storage_root: H::Out) {
        let ValueCache::Fresh(cache) = self.value_cache else { return };

        if !cache.is_empty() {
            let mut value_cache = local.value_cache.lock();
            let partial_hash = ValueCacheKey::hash_partial_data(&storage_root);

            cache.into_iter().for_each(|(k, v)| {
                let hash = ValueCacheKeyHash::from_hasher_and_storage_key(partial_hash.clone(), &k);
                let k = ValueCacheRef { storage_root, storage_key: &k, hash };
                value_cache.insert(k, v);
            });
        }
    }
}

impl<'a> trie_db::TrieCache<NodeCodec> for TrieCache<'a, KeccakHasher> {
    fn get_or_insert_node(
        &mut self,
        hash: H256,
        fetch_node: &mut dyn FnMut() -> TrieResult<NodeOwned<H256>>,
    ) -> TrieResult<&NodeOwned<H256>> {
        let mut is_local_cache_hit = true;
        self.stats.node_cache.local_fetch_attempts.fetch_add(1, Ordering::Relaxed);

        // First try to grab the node from the local cache.
        let node = self.local_cache.get_or_insert_fallible(hash, || {
            is_local_cache_hit = false;

            // It was not in the local cache; try the shared cache.
            self.stats.node_cache.shared_fetch_attempts.fetch_add(1, Ordering::Relaxed);
            if let Some(node) = self.shared_cache.peek_node(&hash) {
                self.stats.node_cache.shared_hits.fetch_add(1, Ordering::Relaxed);
                tracing::trace!(target: LOG_TARGET, ?hash, "Serving node from shared cache");
                return Ok(NodeCached::<H256> { node, is_from_shared_cache: true });
            }

            // It was not in the shared cache; try fetching it from the database.
            match fetch_node() {
                Ok(node) => {
                    tracing::trace!(target: LOG_TARGET, ?hash, "Serving node from database");
                    Ok(NodeCached::<H256> { node, is_from_shared_cache: false })
                },
                Err(error) => {
                    tracing::trace!(target: LOG_TARGET, ?hash, "Serving node from database failed");
                    Err(error)
                },
            }
        });

        if is_local_cache_hit {
            tracing::trace!(target: LOG_TARGET, ?hash, "Serving node from local cache");
            self.stats.node_cache.local_hits.fetch_add(1, Ordering::Relaxed);
        }

        #[allow(clippy::expect_used)]
        Ok(&node?
            .expect("you can always insert at least one element into the local cache; qed")
            .node)
    }

    fn get_node(&mut self, hash: &H256) -> Option<&NodeOwned<H256>> {
        let mut is_local_cache_hit = true;
        self.stats.node_cache.local_fetch_attempts.fetch_add(1, Ordering::Relaxed);

        // First try to grab the node from the local cache.
        let cached_node = self.local_cache.get_or_insert_fallible(*hash, || {
            is_local_cache_hit = false;

            // It was not in the local cache; try the shared cache.
            self.stats.node_cache.shared_fetch_attempts.fetch_add(1, Ordering::Relaxed);
            if let Some(node) = self.shared_cache.peek_node(hash) {
                self.stats.node_cache.shared_hits.fetch_add(1, Ordering::Relaxed);
                tracing::trace!(target: LOG_TARGET, ?hash, "Serving node from shared cache");
                Ok(NodeCached::<H256> { node, is_from_shared_cache: true })
            } else {
                tracing::trace!(target: LOG_TARGET, ?hash, "Serving node from cache failed");
                Err(())
            }
        });

        if is_local_cache_hit {
            tracing::trace!(target: LOG_TARGET, ?hash, "Serving node from local cache");
            self.stats.node_cache.local_hits.fetch_add(1, Ordering::Relaxed);
        }

        match cached_node {
            Ok(Some(cached_node)) => Some(&cached_node.node),
            Ok(None) => {
                unreachable!(
                    "you can always insert at least one element into the local cache; qed"
                );
            },
            Err(()) => None,
        }
    }

    fn lookup_value_for_key(&mut self, key: &[u8]) -> Option<&CachedValue<H256>> {
        let res = self.value_cache.get(key, &self.shared_cache, &self.stats.value_cache);

        tracing::trace!(
            target: LOG_TARGET,
            found = res.is_some(),
            "Looked up value for key",
        );

        res
    }

    fn cache_value_for_key(&mut self, key: &[u8], data: CachedValue<H256>) {
        tracing::trace!(
            target: LOG_TARGET,
            "Caching value for key",
        );

        self.value_cache.insert(key, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hasher::KeccakHasher, mem_db::new_memory_db, trie::Layout};
    use memory_db::HashKey;
    use std::vec::Vec;
    use trie_db::{Bytes, Trie, TrieDBBuilder, TrieDBMutBuilder, TrieHash, TrieMut};

    type MemoryDB = memory_db::MemoryDB<KeccakHasher, HashKey<KeccakHasher>, Vec<u8>>;
    type Cache = super::SharedTrieCache<KeccakHasher>;
    // type Recorder = crate::recorder::Recorder<sp_core::Blake2Hasher>;

    const TEST_DATA: &[(&[u8], &[u8])] =
        &[(b"key1", b"val1"), (b"key2", &[2; 64]), (b"key3", b"val3"), (b"key4", &[4; 64])];
    const CACHE_SIZE_RAW: usize = 1024 * 10;
    const CACHE_SIZE: CacheSize = CacheSize::new(CACHE_SIZE_RAW);

    fn create_trie() -> (MemoryDB, TrieHash<Layout>) {
        let (mut db, mut root) = new_memory_db();
        {
            let mut trie = TrieDBMutBuilder::<Layout>::from_existing(&mut db, &mut root).build();
            for (k, v) in TEST_DATA {
                trie.insert(k, v).expect("Inserts data");
            }
        }
        (db, root)
    }

    #[test]
    #[allow(clippy::significant_drop_tightening)]
    fn basic_cache_works() {
        let (db, root) = create_trie();

        let shared_cache = Cache::new(CACHE_SIZE);
        let local_cache = shared_cache.local_cache();

        {
            let mut cache = local_cache.as_trie_db_cache(root);
            let trie = TrieDBBuilder::<Layout>::new(&db, &root).with_cache(&mut cache).build();
            assert_eq!(TEST_DATA[0].1.to_vec(), trie.get(TEST_DATA[0].0).unwrap().unwrap());
        }

        // Local cache wasn't dropped yet, so there should nothing in the shared caches.
        assert!(shared_cache.read_lock_inner().value_cache().lru.is_empty());
        assert!(shared_cache.read_lock_inner().node_cache().lru.is_empty());

        drop(local_cache);

        // Now we should have the cached items in the shared cache.
        assert!(!shared_cache.read_lock_inner().node_cache().lru.is_empty());
        let cached_data = shared_cache
            .read_lock_inner()
            .value_cache()
            .lru
            .peek(&ValueCacheKey::new_value(TEST_DATA[0].0, root))
            .unwrap()
            .clone();
        assert_eq!(Bytes::from(TEST_DATA[0].1.to_vec()), cached_data.data().flatten().unwrap());

        let fake_data = Bytes::from(&b"fake_data"[..]);

        let local_cache = shared_cache.local_cache();

        #[allow(clippy::redundant_clone)]
        shared_cache.write_lock_inner().unwrap().value_cache_mut().lru.insert(
            ValueCacheKey::new_value(TEST_DATA[1].0, root),
            (fake_data.clone(), H256::default()).into(),
        );

        #[allow(clippy::significant_drop_tightening)]
        {
            let mut cache = local_cache.as_trie_db_cache(root);
            let trie = TrieDBBuilder::<Layout>::new(&db, &root).with_cache(&mut cache).build();

            // We should now get the "fake_data", because we inserted this manually to the cache.
            assert_eq!(b"fake_data".to_vec(), trie.get(TEST_DATA[1].0).unwrap().unwrap());
        }
    }

    #[test]
    fn trie_db_mut_cache_works() {
        let (mut db, root) = create_trie();

        let new_key = b"new_key".to_vec();
        // Use some long value to not have it inlined
        let new_value = vec![23; 64];

        let shared_cache = Cache::new(CACHE_SIZE);
        let mut new_root = root;

        {
            let local_cache = shared_cache.local_cache();

            let mut cache = local_cache.as_trie_db_mut_cache();

            {
                let mut trie = TrieDBMutBuilder::<Layout>::from_existing(&mut db, &mut new_root)
                    .with_cache(&mut cache)
                    .build();

                trie.insert(&new_key, &new_value).unwrap();
            }

            cache.merge_into(&local_cache, new_root);
        }

        // After the local cache is dropped, all changes should have been merged back to the shared
        // cache.
        let cached_data = shared_cache
            .read_lock_inner()
            .value_cache()
            .lru
            .peek(&ValueCacheKey::new_value(new_key, new_root))
            .unwrap()
            .clone();
        assert_eq!(Bytes::from(new_value), cached_data.data().flatten().unwrap());
    }

    /*
    #[test]
    fn trie_db_cache_and_recorder_work_together() {
        let (db, root) = create_trie();

        let shared_cache = Cache::new(CACHE_SIZE);

        for i in 0..5 {
            // Clear some of the caches.
            if i == 2 {
                shared_cache.reset_node_cache();
            } else if i == 3 {
                shared_cache.reset_value_cache();
            }

            let local_cache = shared_cache.local_cache();
            let recorder = Recorder::default();

            {
                let mut cache = local_cache.as_trie_db_cache(root);
                let mut recorder = recorder.as_trie_recorder(root);
                let trie = TrieDBBuilder::<Layout>::new(&db, &root)
                    .with_cache(&mut cache)
                    .with_recorder(&mut recorder)
                    .build();

                for (key, value) in TEST_DATA {
                    assert_eq!(*value, trie.get(&key).unwrap().unwrap());
                }
            }

            let storage_proof = recorder.drain_storage_proof();
            let memory_db: MemoryDB = storage_proof.into_memory_db();

            {
                let trie = TrieDBBuilder::<Layout>::new(&memory_db, &root).build();

                for (key, value) in TEST_DATA {
                    assert_eq!(*value, trie.get(&key).unwrap().unwrap());
                }
            }
        }
    }

    #[test]
    fn trie_db_mut_cache_and_recorder_work_together() {
        const DATA_TO_ADD: &[(&[u8], &[u8])] = &[(b"key11", &[45; 78]), (b"key33", &[78; 89])];

        let (db, root) = create_trie();

        let shared_cache = Cache::new(CACHE_SIZE);

        // Run this twice so that we use the data cache in the second run.
        for i in 0..5 {
            // Clear some of the caches.
            if i == 2 {
                shared_cache.reset_node_cache();
            } else if i == 3 {
                shared_cache.reset_value_cache();
            }

            let recorder = Recorder::default();
            let local_cache = shared_cache.local_cache();
            let mut new_root = root;

            {
                let mut db = db.clone();
                let mut cache = local_cache.as_trie_db_cache(root);
                let mut recorder = recorder.as_trie_recorder(root);
                let mut trie = TrieDBMutBuilder::<Layout>::from_existing(&mut db, &mut new_root)
                    .with_cache(&mut cache)
                    .with_recorder(&mut recorder)
                    .build();

                for (key, value) in DATA_TO_ADD {
                    trie.insert(key, value).unwrap();
                }
            }

            let storage_proof = recorder.drain_storage_proof();
            let mut memory_db: MemoryDB = storage_proof.into_memory_db();
            let mut proof_root = root;

            {
                let mut trie =
                    TrieDBMutBuilder::<Layout>::from_existing(&mut memory_db, &mut proof_root)
                        .build();

                for (key, value) in DATA_TO_ADD {
                    trie.insert(key, value).unwrap();
                }
            }

            assert_eq!(new_root, proof_root)
        }
    }

    */
    #[test]
    #[allow(clippy::significant_drop_tightening)]
    fn cache_lru_works() {
        let (db, root) = create_trie();

        let shared_cache = Cache::new(CACHE_SIZE);

        {
            let local_cache = shared_cache.local_cache();
            let mut cache = local_cache.as_trie_db_cache(root);
            let trie = TrieDBBuilder::<Layout>::new(&db, &root).with_cache(&mut cache).build();
            for (k, _) in TEST_DATA {
                trie.get(k).unwrap().unwrap();
            }
        }

        // Check that all items are there.
        assert!(shared_cache
            .read_lock_inner()
            .value_cache()
            .lru
            .iter()
            .map(|d| d.0)
            .all(|l| TEST_DATA.iter().any(|d| &*l.storage_key == d.0)));

        // Run this in a loop. The first time we check that with the filled value cache,
        // the expected values are at the top of the LRU.
        // The second run is using an empty value cache to ensure that we access the nodes.
        for _ in 0..2 {
            {
                let local_cache = shared_cache.local_cache();
                let mut cache = local_cache.as_trie_db_cache(root);
                let trie = TrieDBBuilder::<Layout>::new(&db, &root).with_cache(&mut cache).build();

                for (k, _) in TEST_DATA.iter().take(2) {
                    trie.get(k).unwrap().unwrap();
                }
            }

            // Ensure that the accessed items are most recently used items of the shared value
            // cache.
            assert!(shared_cache
                .read_lock_inner()
                .value_cache()
                .lru
                .iter()
                .take(2)
                .map(|d| d.0)
                .all(|l| { TEST_DATA.iter().take(2).any(|d| &*l.storage_key == d.0) }));

            // Delete the value cache, so that we access the nodes.
            shared_cache.reset_value_cache();
        }

        let most_recently_used_nodes = shared_cache
            .read_lock_inner()
            .node_cache()
            .lru
            .iter()
            .map(|d| *d.0)
            .collect::<Vec<_>>();

        {
            let local_cache = shared_cache.local_cache();
            let mut cache = local_cache.as_trie_db_cache(root);
            let trie = TrieDBBuilder::<Layout>::new(&db, &root).with_cache(&mut cache).build();
            for (k, _) in TEST_DATA.iter().skip(2) {
                trie.get(k).unwrap().unwrap();
            }
        }

        // Ensure that the most recently used nodes changed as well.
        assert_ne!(
            most_recently_used_nodes,
            shared_cache
                .read_lock_inner()
                .node_cache()
                .lru
                .iter()
                .map(|d| *d.0)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn cache_respects_bounds() {
        let (mut db, root) = create_trie();

        let shared_cache = Cache::new(CACHE_SIZE);
        {
            let local_cache = shared_cache.local_cache();
            let mut new_root = root;
            {
                let mut cache = local_cache.as_trie_db_cache(root);
                {
                    let mut trie =
                        TrieDBMutBuilder::<Layout>::from_existing(&mut db, &mut new_root)
                            .with_cache(&mut cache)
                            .build();

                    let value = vec![10u8; 100];
                    // Ensure we add enough data that would overflow the cache.
                    for i in 0..CACHE_SIZE_RAW / 100 * 2 {
                        trie.insert(format!("key{i}").as_bytes(), &value).unwrap();
                    }
                }

                cache.merge_into(&local_cache, new_root);
            }
        }

        assert!(shared_cache.used_memory_size() < CACHE_SIZE_RAW);
    }
}
