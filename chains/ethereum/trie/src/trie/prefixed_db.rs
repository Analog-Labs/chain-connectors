use crate::rstd::{marker::PhantomData, vec, vec::Vec};
use hash_db::{AsHashDB, HashDB, HashDBRef, Hasher, Prefix};

/// `HashDB` implementation that append a encoded prefix (unique id bytes) in addition to the
/// prefix of every key value.
pub struct KeySpacedDB<'a, DB: ?Sized, H>(&'a DB, &'a [u8], PhantomData<H>);

/// `HashDBMut` implementation that append a encoded prefix (unique id bytes) in addition to the
/// prefix of every key value.
///
/// Mutable variant of `KeySpacedDB`, see [`KeySpacedDB`].
pub struct KeySpacedDBMut<'a, DB: ?Sized, H>(&'a mut DB, &'a [u8], PhantomData<H>);

/// Utility function used to merge some byte data (keyspace) and `prefix` data
/// before calling key value database primitives.
fn keyspace_as_prefix_alloc(ks: &[u8], prefix: Prefix) -> (Vec<u8>, Option<u8>) {
    let mut result = vec![0; ks.len() + prefix.0.len()];
    result[..ks.len()].copy_from_slice(ks);
    result[ks.len()..].copy_from_slice(prefix.0);
    (result, prefix.1)
}

impl<'a, DB: ?Sized, H> KeySpacedDB<'a, DB, H> {
    /// instantiate new keyspaced db
    #[inline]
    pub const fn new(db: &'a DB, ks: &'a [u8]) -> Self {
        Self(db, ks, PhantomData)
    }
}

impl<'a, DB: ?Sized, H> KeySpacedDBMut<'a, DB, H> {
    /// instantiate new keyspaced db
    pub fn new(db: &'a mut DB, ks: &'a [u8]) -> Self {
        Self(db, ks, PhantomData)
    }
}

impl<'a, DB, H, T> HashDBRef<H, T> for KeySpacedDB<'a, DB, H>
where
    DB: HashDBRef<H, T> + ?Sized,
    H: Hasher,
    T: From<&'static [u8]>,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<T> {
        let derived_prefix = keyspace_as_prefix_alloc(self.1, prefix);
        self.0.get(key, (&derived_prefix.0, derived_prefix.1))
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        let derived_prefix = keyspace_as_prefix_alloc(self.1, prefix);
        self.0.contains(key, (&derived_prefix.0, derived_prefix.1))
    }
}

impl<'a, DB, H, T> HashDB<H, T> for KeySpacedDBMut<'a, DB, H>
where
    DB: HashDB<H, T>,
    H: Hasher,
    T: Default + PartialEq<T> + for<'b> From<&'b [u8]> + Clone + Send + Sync,
{
    fn get(&self, key: &H::Out, prefix: Prefix) -> Option<T> {
        let derived_prefix = keyspace_as_prefix_alloc(self.1, prefix);
        self.0.get(key, (&derived_prefix.0, derived_prefix.1))
    }

    fn contains(&self, key: &H::Out, prefix: Prefix) -> bool {
        let derived_prefix = keyspace_as_prefix_alloc(self.1, prefix);
        self.0.contains(key, (&derived_prefix.0, derived_prefix.1))
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H::Out {
        let derived_prefix = keyspace_as_prefix_alloc(self.1, prefix);
        self.0.insert((&derived_prefix.0, derived_prefix.1), value)
    }

    fn emplace(&mut self, key: H::Out, prefix: Prefix, value: T) {
        let derived_prefix = keyspace_as_prefix_alloc(self.1, prefix);
        self.0.emplace(key, (&derived_prefix.0, derived_prefix.1), value);
    }

    fn remove(&mut self, key: &H::Out, prefix: Prefix) {
        let derived_prefix = keyspace_as_prefix_alloc(self.1, prefix);
        self.0.remove(key, (&derived_prefix.0, derived_prefix.1));
    }
}

impl<'a, DB, H, T> AsHashDB<H, T> for KeySpacedDBMut<'a, DB, H>
where
    DB: HashDB<H, T>,
    H: Hasher,
    T: Default + PartialEq<T> + for<'b> From<&'b [u8]> + Clone + Send + Sync,
{
    fn as_hash_db(&self) -> &dyn hash_db::HashDB<H, T> {
        self
    }

    fn as_hash_db_mut<'b>(&'b mut self) -> &'b mut (dyn hash_db::HashDB<H, T> + 'b) {
        &mut *self
    }
}
