use crate::{
    hasher::{Hasher, KeccakHasher},
    mem_db::{
        account_db::{AccountDB, AccountDBMut},
        iterator::TrieIterator,
        state_trie::{AccountInfo, StateTrie},
    },
    node_codec::HASHED_NULL_NODE,
    rstd::{boxed::Box, collections::btree_map::BTreeMap, convert::AsRef, mem, vec::Vec},
    trie::{FatDB, FatDBMut, Result as TrieResult, SecTrieDB, TrieDBBuilder, TrieError},
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

    #[must_use]
    pub fn code(&self) -> Option<Vec<u8>> {
        self.state_trie.code(&self.info.code_hash)
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

    #[must_use]
    pub const fn info(&self) -> &AccountInfo {
        &self.info
    }

    #[must_use]
    pub fn info_mut(&mut self) -> &mut AccountInfo {
        &mut self.info
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

    #[must_use]
    pub fn code(&self) -> Option<Vec<u8>> {
        self.state_trie.code(&self.info.code_hash)
    }

    #[must_use]
    pub const fn code_hash(&self) -> H256 {
        self.info.code_hash
    }

    pub fn set_code(&mut self, bytecode: impl AsRef<[u8]>) -> H256 {
        let code_hash = self.state_trie.insert_code(bytecode);
        mem::replace(&mut self.info.code_hash, code_hash)
    }

    #[must_use]
    pub const fn balance(&self) -> U256 {
        self.info.balance
    }

    #[must_use]
    pub const fn nonce(&self) -> u64 {
        self.info.nonce
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.info.storage_hash == HASHED_NULL_NODE
    }

    pub fn set_balance(&mut self, balance: U256) -> U256 {
        mem::replace(&mut self.info.balance, balance)
    }

    pub fn insert(&mut self, slot: H256, value: U256) {
        self.changes.insert(slot, value);
    }

    pub fn remove(&mut self, slot: H256) {
        self.changes.insert(slot, U256::zero());
    }

    pub fn reset_storage(&mut self) -> TrieResult<()> {
        let mut db = AccountDBMut::from_hash(self.state_trie.db_mut(), self.address_hash);
        let keys = {
            let trie = FatDB::new(&db, &self.info.storage_hash);
            let iterator = trie.key_iter()?;
            iterator
                .filter_map(|res| res.ok().map(|bytes| H256::from_slice(bytes.as_ref())))
                .collect::<Vec<_>>()
        };
        {
            let mut trie = FatDBMut::from_existing(&mut db, &mut self.info.storage_hash);
            for key in keys {
                trie.remove(key.as_bytes())?;
            }
            debug_assert!(trie.is_empty());
        }
        self.info.storage_hash = HASHED_NULL_NODE;
        self.state_trie.update_account(&self.address, &self.info)?;
        Ok(())
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
        self.state_trie.update_account(&self.address, &self.info)?;
        Ok(())
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn try_iter(&self) -> TrieResult<TrieIterator<'_, 'static, H256, U256>> {
        let trie = TrieDBBuilder::new(self.state_trie.db(), &self.info.storage_hash).build();
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

impl<'a> AsMut<AccountInfo> for AccountTrieMut<'a> {
    fn as_mut(&mut self) -> &mut AccountInfo {
        &mut self.info
    }
}
