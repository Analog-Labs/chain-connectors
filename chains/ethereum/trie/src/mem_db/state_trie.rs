use crate::{
    hasher::KeccakHasher,
    mem_db::{
        account_trie::{AccountTrie, AccountTrieMut},
        iterator::TrieIterator,
    },
    node_codec::HASHED_NULL_NODE,
    rstd::{
        boxed::Box, collections::btree_map::BTreeMap, convert::AsRef, default::Default,
        iter::Iterator, vec::Vec,
    },
    trie::{FatDB, FatDBMut, Result as TrieResult, SecTrieDBMut, TrieDBBuilder, TrieError},
};
use bytes::Bytes;
use hash_db::HashDB;
use hex_literal::hex;
use memory_db::{HashKey, MemoryDB};
use primitive_types::{H160, H256, U256};
use rlp::{RlpStream, NULL_RLP};
use rlp_derive::{RlpDecodable, RlpEncodable};
use trie_db::{Trie, TrieMut};

type Address = H160;

/// Type alias for the `HashDB` representation of the Database
pub type AsHashDB = Box<dyn HashDB<KeccakHasher, Vec<u8>>>;

pub type DefaultMemoryDb = MemoryDB<KeccakHasher, HashKey<KeccakHasher>, Vec<u8>>;

const KECCAK_EMPTY: H256 =
    H256(hex!("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"));

pub struct StateTrie {
    db: DefaultMemoryDb,
    root: H256,
}

#[allow(clippy::missing_errors_doc)]
impl StateTrie {
    #[must_use]
    pub const fn root(&self) -> H256 {
        self.root
    }

    #[must_use]
    pub const fn db(&self) -> &DefaultMemoryDb {
        &self.db
    }

    #[must_use]
    pub fn db_mut(&mut self) -> &mut DefaultMemoryDb {
        &mut self.db
    }

    pub fn get(&self, address: &Address) -> TrieResult<Option<AccountInfo>> {
        let trie = FatDB::new(&self.db, &self.root);
        let Some(bytes) = trie.get(address.as_bytes())? else {
            return Ok(None);
        };
        let acc = rlp::decode::<AccountInfo>(bytes.as_ref())
            .map_err(|err| TrieError::DecoderError(self.root, err))?;
        Ok(Some(acc))
    }

    pub fn contains(&self, address: &Address) -> TrieResult<bool> {
        let trie = FatDB::new(&self.db, &self.root);
        trie.contains(address.as_bytes())
    }

    pub fn remove(&mut self, address: &Address) -> TrieResult<Option<AccountInfo>> {
        let Some(mut account_trie) = self.account_mut(*address)? else {
            return Ok(None);
        };
        // Save old value
        let info = account_trie.info().clone();

        // Delete account storage
        account_trie.reset_storage()?;
        drop(account_trie);

        // Delete account info
        let mut trie = FatDBMut::from_existing(&mut self.db, &mut self.root);
        trie.remove(address.as_bytes())?;
        Ok(Some(info))
    }

    pub fn create_account(
        &mut self,
        address: &Address,
        balance: U256,
        bytecode: Option<&[u8]>,
    ) -> TrieResult<AccountInfo> {
        let code_hash = bytecode.map_or(KECCAK_EMPTY, |bytecode| self.insert_code(bytecode));
        let info = AccountInfo { balance, code_hash, ..Default::default() };
        let bytes = rlp::encode(&info).freeze();
        {
            let mut trie = FatDBMut::from_existing(&mut self.db, &mut self.root);
            trie.insert(address.as_bytes(), bytes.as_ref())?;
        }
        Ok(info)
    }

    pub fn update_account(&mut self, address: &Address, account: &AccountInfo) -> TrieResult<()> {
        let bytes = rlp::encode(account).freeze();
        {
            let mut trie = FatDBMut::from_existing(&mut self.db, &mut self.root);
            trie.insert(address.as_bytes(), bytes.as_ref())?;
        }
        Ok(())
    }

    pub fn delete_account(&mut self, address: &Address) -> TrieResult<()> {
        let mut trie = FatDBMut::from_existing(&mut self.db, &mut self.root);
        trie.remove(address.as_bytes())?;
        Ok(())
    }

    pub fn account(&self, address: &Address) -> TrieResult<Option<AccountTrie<'_>>> {
        let Some(account) = self.get(address)? else {
            return Ok(None);
        };
        Ok(Some(AccountTrie::new(self, address, account)))
    }

    pub fn account_mut(&mut self, address: Address) -> TrieResult<Option<AccountTrieMut<'_>>> {
        let Some(account) = self.get(&address)? else {
            return Ok(None);
        };
        Ok(Some(AccountTrieMut::new(self, address, account)))
    }

    pub fn get_or_create(&mut self, address: Address) -> TrieResult<AccountTrieMut<'_>> {
        let Some(account) = self.get(&address)? else {
            let account = self.create_account(&address, U256::zero(), None)?;
            return Ok(AccountTrieMut::new(self, address, account));
        };
        Ok(AccountTrieMut::new(self, address, account))
    }

    #[must_use]
    pub fn code(&self, hash: &H256) -> Option<Vec<u8>> {
        if hash == &KECCAK_EMPTY {
            return None;
        }
        self.db.get(hash, Default::default())
    }

    pub fn insert_code(&mut self, bytecode: impl AsRef<[u8]>) -> H256 {
        use trie_db::Hasher;
        let bytecode = bytecode.as_ref();
        if bytecode.is_empty() {
            return KECCAK_EMPTY;
        }
        let hash = <KeccakHasher as Hasher>::hash(bytecode);
        let prefix = (hash.as_bytes(), Option::<u8>::None);
        self.db.insert(prefix, bytecode);
        hash
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn try_iter(&self) -> TrieResult<TrieIterator<'_, 'static, H160, AccountInfo>> {
        let trie = TrieDBBuilder::new(&self.db, &self.root).build();
        TrieIterator::new(
            trie,
            |bytes| H160::from_slice(bytes.as_ref()),
            |bytes| {
                #[allow(clippy::expect_used)]
                rlp::decode::<AccountInfo>(bytes.as_ref())
                    .expect("failed to decode AccountInfo from storage")
            },
        )
    }
}

impl Default for StateTrie {
    fn default() -> Self {
        let (db, root) = new_memory_db();
        Self { db, root }
    }
}

pub trait AccountInfoT {
    fn nonce(&self) -> u64;
    fn balance(&self) -> U256;
    fn code_hash(&self) -> H256;
    fn code(&self) -> Option<&[u8]>;
}

pub trait DbAccountT: AccountInfoT {
    type StorageIter<'a>: Iterator<Item = (&'a H256, &'a U256)> + Send + 'a
    where
        Self: 'a;

    fn storage(&self) -> Self::StorageIter<'_>;
}

#[must_use]
pub fn new_memory_db() -> (MemoryDB<KeccakHasher, HashKey<KeccakHasher>, Vec<u8>>, H256) {
    let db = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::from_null_node(
        &NULL_RLP,
        NULL_RLP.as_ref().into(),
    );
    (db, HASHED_NULL_NODE)
}

/// Returns storage trie of an account as `HashDB`
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn storage_trie_db<'i, I>(storage: I) -> (AsHashDB, H256)
where
    I: Iterator<Item = (&'i H256, &'i U256)> + 'i,
{
    // Populate DB with full trie from entries.
    let (db, root) = {
        let (mut db, mut root) = new_memory_db();
        {
            let mut trie = SecTrieDBMut::new(&mut db, &mut root);
            for (key, value) in storage.filter(|(_k, v)| !v.is_zero()) {
                let value = rlp::encode(value).freeze();
                #[allow(clippy::unwrap_used)]
                trie.insert(key.as_bytes(), value.as_ref()).unwrap();
            }
        }
        (db, root)
    };

    (Box::new(db), root)
}

/// Returns the account data as `HashDB`
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn trie_hash_db<'i, ACC, I>(accounts: I) -> (AsHashDB, H256)
where
    ACC: DbAccountT + 'i,
    I: Iterator<Item = (&'i Address, &'i ACC)> + 'i,
{
    // let accounts = trie_accounts(accounts);
    let accounts = accounts.map(|(address, account)| {
        let storage_root = trie_account_rlp(account, account.storage());
        (*address, storage_root)
    });

    // Populate DB with full trie from entries.
    let (db, root) = {
        let (mut db, mut root) = new_memory_db();
        {
            let mut trie = SecTrieDBMut::new(&mut db, &mut root);
            for (address, value) in accounts {
                #[allow(clippy::unwrap_used)]
                trie.insert(address.as_ref(), value.as_ref()).unwrap();
            }
        }
        (db, root)
    };

    (Box::new(db), root)
}

/// Returns the RLP for this account.
pub fn trie_account_rlp<'t, 'i, T, I>(info: &'t T, storage: I) -> Bytes
where
    T: AccountInfoT,
    I: Iterator<Item = (&'i H256, &'i U256)> + 'i,
{
    let mut stream = RlpStream::new_list(4);
    stream.append(&info.nonce());
    stream.append(&info.balance());
    stream.append(&storage_trie_db(storage).1);
    stream.append(&info.code_hash().as_bytes());
    stream.out().freeze()
}

/// `AccountInfo` account information.
#[derive(Clone, Debug, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct AccountInfo {
    /// Account nonce.
    pub nonce: u64,
    /// Account balance.
    pub balance: U256,
    /// storage hash.
    pub storage_hash: H256,
    /// code hash.
    pub code_hash: H256,
}

impl AccountInfo {
    /// Returns if an account is empty.
    ///
    /// An account is empty if the following conditions are met.
    /// - code hash is zero or set to the Keccak256 hash of the empty string `""`
    /// - balance is zero
    /// - nonce is zero
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let code_empty = self.is_empty_code_hash() || self.code_hash == H256::zero();
        self.balance.is_zero() && self.nonce == 0 && code_empty
    }

    /// Returns true if the code hash is the Keccak256 hash of the empty string `""`.
    #[inline]
    #[must_use]
    pub fn is_empty_code_hash(&self) -> bool {
        self.code_hash == KECCAK_EMPTY
    }

    /// Returns `true` if account has no nonce and code.
    #[must_use]
    pub fn has_no_code_and_nonce(&self) -> bool {
        self.is_empty_code_hash() && self.nonce == 0
    }
}

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            nonce: 0,
            balance: U256::zero(),
            storage_hash: HASHED_NULL_NODE,
            code_hash: KECCAK_EMPTY,
        }
    }
}

impl AccountInfoT for AccountInfo {
    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn balance(&self) -> U256 {
        self.balance
    }

    fn code_hash(&self) -> H256 {
        self.code_hash
    }

    fn code(&self) -> Option<&[u8]> {
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct DbAccount {
    pub info: AccountInfo,
    /// If account is selfdestructed or newly created, storage will be cleared.
    pub account_state: AccountState,
    /// storage slots
    pub storage: BTreeMap<H256, U256>,
}

impl AccountInfoT for DbAccount {
    fn nonce(&self) -> u64 {
        self.info.nonce
    }

    fn balance(&self) -> U256 {
        self.info.balance
    }

    fn code_hash(&self) -> H256 {
        self.info.code_hash
    }

    fn code(&self) -> Option<&[u8]> {
        None
    }
}

impl DbAccountT for DbAccount {
    type StorageIter<'a> = crate::rstd::collections::btree_map::Iter<'a, H256, U256>;
    fn storage(&self) -> Self::StorageIter<'_> {
        self.storage.iter()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum AccountState {
    /// Before Spurious Dragon hardfork there was a difference between empty and not existing.
    /// And we are flagging it here.
    NotExisting,
    /// EVM touched this account. For newer hardfork this means it can be cleared/removed from
    /// state.
    Touched,
    /// EVM cleared storage of this account, mostly by selfdestruct, we don't ask database for
    /// storage slots and assume they are U256::ZERO
    StorageCleared,
    /// EVM didn't interacted with this account
    #[default]
    None,
}

#[cfg(test)]
mod tests {
    use super::{trie_hash_db, AccountInfo, AccountState, DbAccount, StateTrie};
    use crate::node_codec::HASHED_NULL_NODE;
    use hex_literal::hex;
    use primitive_types::{H160, H256, U256};
    use serde::{Deserialize, Serialize};
    use std::collections::{BTreeMap, BTreeSet};

    // Ethereum mainnet genesis accounts
    // ref: https://github.com/openethereum/parity-ethereum/blob/v3.0.1/ethcore/res/ethereum/foundation.json
    const MAINNET_GENESIS: &str = include_str!("../../testdata/mainnet_genesis.json");
    const GENESIS_STATE_ROOT: H256 =
        H256(hex!("d7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544"));

    /// Creates a new memory db with the genesis accounts.
    pub fn create_genesis_db() -> StateTrie {
        let genesis_accounts: Vec<GenesisAccount> = serde_json::from_str(MAINNET_GENESIS).unwrap();
        let mut db = StateTrie::default();
        for acc in genesis_accounts {
            db.create_account(&acc.address, acc.balance, None).unwrap();
        }
        assert_eq!(db.root(), GENESIS_STATE_ROOT);
        db
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    struct GenesisAccount {
        pub address: H160,
        pub balance: U256,
    }

    #[test]
    fn test_trie_hash_db() {
        let genesis_accounts: Vec<GenesisAccount> = serde_json::from_str(MAINNET_GENESIS).unwrap();
        assert_eq!(genesis_accounts.len(), 8893);

        let empty_storage = BTreeMap::new();
        let mut accounts = BTreeMap::new();

        for acc in genesis_accounts {
            let info = AccountInfo { balance: acc.balance, ..Default::default() };
            let account = DbAccount {
                info,
                account_state: AccountState::None,
                storage: empty_storage.clone(),
            };
            accounts.insert(acc.address, account);
        }

        let (_, root) = trie_hash_db(accounts.iter());
        assert_eq!(root, GENESIS_STATE_ROOT);

        // Add a new account
        accounts.insert(
            H160(hex!("05a56e2d52c817161883f50c441c3228cfe54d9f")),
            DbAccount {
                info: AccountInfo {
                    balance: U256::from(5u128 * 10u128.pow(18)), // 5 eth
                    ..Default::default()
                },
                account_state: AccountState::None,
                storage: empty_storage,
            },
        );
        let (_, root) = trie_hash_db(accounts.iter());
        assert_eq!(
            root,
            H256(hex!("d67e4d450343046425ae4271474353857ab860dbc0a1dde64b41b5cd3a532bf3"))
        );
    }

    #[test]
    fn test_eth_storage_state_root() {
        let genesis_accounts: BTreeSet<GenesisAccount> =
            serde_json::from_str(MAINNET_GENESIS).unwrap();
        let mut db = StateTrie::default();

        // Check if the state root is the same as the genesis state root
        for acc in &genesis_accounts {
            db.create_account(&acc.address, acc.balance, None).unwrap();
        }
        assert_eq!(db.root(), GENESIS_STATE_ROOT);

        // Check if all accounts are in the db
        let accounts = db
            .try_iter()
            .unwrap()
            .map(|(k, v)| GenesisAccount { address: k, balance: v.balance })
            .collect::<BTreeSet<_>>();
        assert_eq!(accounts.len(), genesis_accounts.len());
        assert_eq!(accounts, genesis_accounts);
    }

    #[test]
    fn test_eth_storage_persist_account() {
        let mut db = StateTrie::default();
        assert_eq!(db.root(), HASHED_NULL_NODE);

        let addr = H160(hex!("05a56e2d52c817161883f50c441c3228cfe54d9f"));
        let expected =
            AccountInfo { balance: U256::from(10_000_000_000_000_u128), ..Default::default() };

        db.create_account(&addr, expected.balance, None).unwrap();
        assert_eq!(
            db.root(),
            H256(hex!("10350a33cc949e08346b43631f6abc0350c1f2d33f842625f86087e32e2dd7a5"))
        );

        let actual = db.get(&addr).unwrap().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_account_storage() {
        let mut db = create_genesis_db();

        let addr = H160(hex!("05a56e2d52c817161883f50c441c3228cfe54d9f"));
        let expected =
            AccountInfo { balance: U256::from(10_000_000_000_000_u128), ..Default::default() };
        assert!(db.account(&addr).unwrap().is_none());

        db.create_account(&addr, expected.balance, None).unwrap();
        {
            let acc_db = db.account(&addr).unwrap().unwrap();
            assert_eq!(acc_db.storage_hash(), HASHED_NULL_NODE);
            let mut keys = acc_db.try_iter().unwrap();
            assert!(keys.next().is_none());
        }
        let mut acc_db = db.account_mut(addr).unwrap().unwrap();
        assert_eq!(acc_db.storage_hash(), HASHED_NULL_NODE);
        acc_db.insert(H256::zero(), U256::one());
        assert_eq!(acc_db.storage_hash(), HASHED_NULL_NODE);
        acc_db.commit().unwrap();
        assert_eq!(
            acc_db.storage_hash(),
            H256(hex!("821e2556a290c86405f8160a2d662042a431ba456b9db265c79bb837c04be5f0"))
        );
        assert_eq!(
            acc_db.state_root(),
            H256(hex!("e6754d97bec1c5bbbebaad9042335733cba771784fafa2fbe3ba0d2b6bac5245"))
        );
        acc_db.insert(H256::zero(), U256::zero());
        acc_db.commit().unwrap();
        assert_eq!(acc_db.storage_hash(), HASHED_NULL_NODE);
    }
}
