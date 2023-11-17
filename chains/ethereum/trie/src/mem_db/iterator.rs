use crate::{
    hasher::Hasher,
    layout::{Layout, Result as TrieResult, TrieDB, TrieDBIterator},
    rstd::{boxed::Box, convert::AsRef, default::Default, iter::Iterator, vec::Vec},
};
use trie_db::{TrieIterator as TrieIteratorTrait, TrieLayout};

type StaticDecodeFn<K> = fn(Vec<u8>) -> K;

/// Iterator over inserted pairs of key values.
pub struct TrieIterator<'db, 'cache, K, V> {
    decode_key: StaticDecodeFn<K>,
    decode_value: StaticDecodeFn<V>,
    trie: Box<TrieDB<'db, 'cache>>,
    trie_iterator: TrieDBIterator<'db, 'cache>,
}

impl<'db, 'cache, K, V> TrieIterator<'db, 'cache, K, V> {
    /// Creates new iterator.
    /// # Errors
    /// Returns `TrieError::DecoderError` if the trie iterator fails to decode the key.
    pub fn new(
        trie: TrieDB<'db, 'cache>,
        k: StaticDecodeFn<K>,
        v: StaticDecodeFn<V>,
    ) -> TrieResult<Self> {
        let trie = Box::new(trie);
        let trie_ref = {
            // Create a 'static reference to the trie
            Box::leak(trie)
        };
        let trie = unsafe { Box::from_raw(trie_ref) };
        let trie_iterator = TrieDBIterator::new(trie_ref)?;
        Ok(Self { trie, trie_iterator, decode_key: k, decode_value: v })
    }
}

impl<'db, 'cache, K, V> TrieIteratorTrait<Layout> for TrieIterator<'db, 'cache, K, V> {
    fn seek(&mut self, key: &[u8]) -> TrieResult<()> {
        let hashed_key = <<Layout as TrieLayout>::Hash as Hasher>::hash(key);
        self.trie_iterator.seek(hashed_key.as_ref())
    }
}

impl<'db, 'cache, K, V> Iterator for TrieIterator<'db, 'cache, K, V> {
    type Item = (K, V);

    #[allow(clippy::expect_used)]
    fn next(&mut self) -> Option<Self::Item> {
        self.trie_iterator.next().map(|res| {
            let (hash, v) = res.expect("trie iterator error");
            let aux_hash = <<Layout as TrieLayout>::Hash as Hasher>::hash(&hash);
            let k = self.trie.db().get(&aux_hash, Default::default()).expect("Missing fatdb hash");
            let k = (self.decode_key)(k);
            let v = (self.decode_value)(v);
            (k, v)
        })
    }
}
