use crate::{
    keccak::KeccakHasher,
    layout::SecTrieDBMut,
    rstd::{vec::Vec, BTreeMap},
};
use bytes::Bytes;
use hash_db::HashDB;
use memory_db::{HashKey, MemoryDB};
use primitive_types::{H160, H256, U256};
use rlp::RlpStream;
use trie_db::TrieMut;

type Address = H160;

/// Type alias for the `HashDB` representation of the Database
pub type AsHashDB = Box<dyn HashDB<KeccakHasher, Vec<u8>>>;

/// Returns storage trie of an account as `HashDB`
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn storage_trie_db(storage: &BTreeMap<U256, U256>) -> (AsHashDB, H256) {
    // Populate DB with full trie from entries.
    let (db, root) = {
        let mut db = <MemoryDB<KeccakHasher, HashKey<_>, _>>::default();
        let mut root = H256::zero();
        {
            let mut trie = SecTrieDBMut::new(&mut db, &mut root);
            for (k, v) in storage.iter().filter(|(_k, v)| *v != &U256::zero()) {
                let mut temp: [u8; 32] = [0; 32];
                (*k).to_big_endian(&mut temp);
                let key = H256::from(temp);
                let value = rlp::encode(v).freeze();
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
pub fn trie_hash_db(accounts: &BTreeMap<Address, DbAccount>) -> (AsHashDB, H256) {
    // let accounts = trie_accounts(accounts);
    let accounts = accounts
        .iter()
        .map(|(address, account)| {
            let storage_root = trie_account_rlp(&account.info, &account.storage);
            (*address, storage_root)
        })
        .collect::<Vec<_>>();

    // Populate DB with full trie from entries.
    let (db, root) = {
        let mut db = <memory_db::MemoryDB<_, HashKey<_>, _>>::default();
        let mut root = H256::zero();
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
pub fn trie_account_rlp(info: &AccountInfo, storage: &BTreeMap<U256, U256>) -> Bytes {
    let mut stream = RlpStream::new_list(4);
    stream.append(&info.nonce);
    stream.append(&info.balance);
    stream.append(&storage_trie_db(storage).1);
    stream.append(&info.code_hash.as_bytes());
    stream.out().freeze()
}

/// `AccountInfo` account information.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct AccountInfo {
    /// Account balance.
    pub balance: U256,
    /// Account nonce.
    pub nonce: u64,
    /// code hash,
    pub code_hash: H256,
    /// code: if None, `code_by_hash` will be used to fetch it if code needs to be loaded from
    /// inside of `revm`.
    pub code: Option<Bytes>,
}

#[derive(Debug, Clone, Default)]
pub struct DbAccount {
    pub info: AccountInfo,
    /// If account is selfdestructed or newly created, storage will be cleared.
    pub account_state: AccountState,
    /// storage slots
    pub storage: BTreeMap<U256, U256>,
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
