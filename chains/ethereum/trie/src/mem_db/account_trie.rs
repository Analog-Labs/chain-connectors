use crate::{
    hasher::{Hasher, KeccakHasher},
    layout::{FatDBMut, Result as TrieResult, SecTrieDB, TrieDBBuilder, TrieError},
    mem_db::{
        account_db::{AccountDB, AccountDBMut},
        iterator::TrieIterator,
        state_trie::{AccountInfo, StateTrie},
    },
    rstd::{boxed::Box, convert::AsRef, BTreeMap},
};
use primitive_types::{H160, H256, U256};
use trie_db::{Trie, TrieMut};

type Address = H160;

pub struct AccountTrie<'db> {
    state_trie: &'db StateTrie,
    db: AccountDB<'db>,
    info: AccountInfo,
}

#[allow(clippy::missing_errors_doc)]
impl<'db> AccountTrie<'db> {
    #[must_use]
    pub fn new(state_trie: &'db StateTrie, address: &Address, info: AccountInfo) -> Self {
        let db = AccountDB::new(state_trie.db(), address);
        Self { state_trie, db, info }
    }

    #[must_use]
    pub const fn state_root(&self) -> H256 {
        self.state_trie.root()
    }

    #[must_use]
    pub const fn storage_hash(&self) -> H256 {
        self.info.storage_hash
    }

    pub fn get(&self, slot: &H256) -> TrieResult<Option<U256>> {
        let trie = SecTrieDB::new(&self.db, &self.info.storage_hash);
        let Some(bytes) = trie.get(slot.as_bytes())? else {
            return Ok(None);
        };
        let value = rlp::decode::<U256>(bytes.as_ref())
            .map_err(|err| TrieError::DecoderError(self.info.storage_hash, err))?;
        Ok(Some(value))
    }

    pub fn contains(&self, slot: &H256) -> TrieResult<bool> {
        let trie = SecTrieDB::new(&self.db, &self.info.storage_hash);
        trie.contains(slot.as_bytes())
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn try_iter(&self) -> TrieResult<TrieIterator<'_, 'static, H256, U256>> {
        let trie = TrieDBBuilder::new(&self.db, &self.info.storage_hash).build();
        TrieIterator::new(
            trie,
            |bytes| H256::from_slice(bytes.as_ref()),
            |bytes| {
                #[allow(clippy::expect_used)]
                rlp::decode::<U256>(bytes.as_ref()).expect("failed to decode U256 from storage")
            },
        )
    }
}

pub struct AccountTrieMut<'db> {
    state_trie: &'db mut StateTrie,
    changes: BTreeMap<H256, U256>,
    info: AccountInfo,
    address: Address,
    address_hash: H256,
}

impl<'db> AccountTrieMut<'db> {
    #[must_use]
    pub fn new(state_trie: &'db mut StateTrie, address: Address, info: AccountInfo) -> Self {
        let address_hash = KeccakHasher::hash(address.as_bytes());
        Self { state_trie, changes: BTreeMap::new(), info, address, address_hash }
    }
}

#[allow(clippy::missing_errors_doc)]
impl<'db> AccountTrieMut<'db> {
    #[must_use]
    pub const fn state_root(&self) -> H256 {
        self.state_trie.root()
    }

    #[must_use]
    pub const fn storage_hash(&self) -> H256 {
        self.info.storage_hash
    }

    pub fn get(&self, slot: &H256) -> TrieResult<U256> {
        if let Some(value) = self.changes.get(slot).copied() {
            return Ok(value);
        };
        let db = AccountDB::from_hash(self.state_trie.db(), self.address_hash);
        let trie = TrieDBBuilder::new(&db, &self.info.storage_hash).build();
        let Some(bytes) = trie.get(slot.as_bytes())? else {
            return Ok(U256::zero());
        };
        rlp::decode::<U256>(bytes.as_ref())
            .map_err(|err| Box::new(TrieError::DecoderError(self.info.storage_hash, err)))
    }

    pub fn contains(&self, slot: &H256) -> TrieResult<bool> {
        if self.changes.contains_key(slot) {
            return Ok(true);
        }
        let db = AccountDB::from_hash(self.state_trie.db(), self.address_hash);
        let trie = TrieDBBuilder::new(&db, &self.info.storage_hash).build();
        trie.contains(slot.as_bytes())
    }

    pub fn insert(&mut self, slot: H256, value: U256) {
        self.changes.insert(slot, value);
    }

    pub fn remove(&mut self, slot: H256) {
        self.changes.insert(slot, U256::zero());
    }

    pub fn commit(&mut self) -> TrieResult<()> {
        let mut db = AccountDBMut::from_hash(self.state_trie.db_mut(), self.address_hash);
        let mut trie = FatDBMut::from_existing(&mut db, &mut self.info.storage_hash);
        while let Some((k, v)) = self.changes.pop_first() {
            if v.is_zero() {
                trie.remove(k.as_bytes())?;
            } else {
                trie.insert(k.as_bytes(), &rlp::encode(&v))?;
            }
        }
        drop(trie);
        self.state_trie.insert_account(&self.address, &self.info)?;
        Ok(())
    }
}
