use crate::{hasher::KeccakHasher, node_codec::HASHED_NULL_NODE, rstd::vec::Vec};
use hash_db::{AsHashDB, HashDB, Hasher, Prefix};
use primitive_types::H256;
use rlp::NULL_RLP;

pub type DBValue = Vec<u8>;

// Combines a key with an address hash to ensure uniqueness.
// leaves the first 96 bits untouched in order to support partial key lookup.
#[inline]
fn combine_key<'a>(address_hash: &'a H256, key: &'a H256) -> H256 {
    let mut dst = *key;
    {
        let last_src: &[u8] = address_hash.as_bytes();
        let last_dst: &mut [u8] = dst.as_bytes_mut();
        for (k, a) in last_dst[12..].iter_mut().zip(&last_src[12..]) {
            *k ^= *a;
        }
    }
    dst
}

/// DB backend wrapper for Account trie
pub struct AccountDBMut<'db, DB> {
    db: &'db mut DB,
    address_hash: H256,
}

impl<'db, DB> AccountDBMut<'db, DB>
where
    DB: HashDB<KeccakHasher, DBValue>,
{
    /// Create a new `AccountDBMut` from an address' hash.
    pub fn from_hash(db: &'db mut DB, address_hash: H256) -> Self {
        Self { db, address_hash }
    }
}

impl<'db, DB> HashDB<KeccakHasher, DBValue> for AccountDBMut<'db, DB>
where
    DB: HashDB<KeccakHasher, DBValue>,
{
    fn get(&self, key: &H256, prefix: Prefix) -> Option<DBValue> {
        if key == &HASHED_NULL_NODE {
            return Some(NULL_RLP.to_vec());
        }
        self.db.get(&combine_key(&self.address_hash, key), prefix)
    }

    fn contains(&self, key: &H256, prefix: Prefix) -> bool {
        if key == &HASHED_NULL_NODE {
            return true;
        }
        self.db.contains(&combine_key(&self.address_hash, key), prefix)
    }

    fn insert(&mut self, prefix: Prefix, value: &[u8]) -> H256 {
        if value == NULL_RLP {
            return HASHED_NULL_NODE;
        }
        let k = KeccakHasher::hash(value);
        let ak = combine_key(&self.address_hash, &k);
        self.db.emplace(ak, prefix, value.to_vec());
        k
    }

    fn emplace(&mut self, key: H256, prefix: Prefix, value: DBValue) {
        if key == HASHED_NULL_NODE {
            return;
        }
        let key = combine_key(&self.address_hash, &key);
        self.db.emplace(key, prefix, value);
    }

    fn remove(&mut self, key: &H256, prefix: Prefix) {
        if key == &HASHED_NULL_NODE {
            return;
        }
        let key = combine_key(&self.address_hash, key);
        self.db.remove(&key, prefix);
    }
}

impl<'db, DB> AsHashDB<KeccakHasher, DBValue> for AccountDBMut<'db, DB>
where
    DB: HashDB<KeccakHasher, DBValue>,
{
    fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, DBValue> {
        self
    }
    fn as_hash_db_mut(&mut self) -> &mut dyn HashDB<KeccakHasher, DBValue> {
        self
    }
}
