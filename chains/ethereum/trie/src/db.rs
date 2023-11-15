use crate::{
    hasher::KeccakHasher,
    layout::SecTrieDBMut,
    node_codec::HASHED_NULL_NODE,
    rstd::{boxed::Box, default::Default, vec::Vec, BTreeMap},
};
use bytes::Bytes;
use hash_db::HashDB;
use hex_literal::hex;
use memory_db::{HashKey, MemoryDB};
use primitive_types::{H160, H256, U256};
use rlp::{RlpStream, NULL_RLP};
use trie_db::TrieMut;

type Address = H160;

/// Type alias for the `HashDB` representation of the Database
pub type AsHashDB = Box<dyn HashDB<KeccakHasher, Vec<u8>>>;

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
pub fn storage_trie_db(storage: &BTreeMap<H256, U256>) -> (AsHashDB, H256) {
    // Populate DB with full trie from entries.
    let (db, root) = {
        let (mut db, mut root) = new_memory_db();
        {
            let mut trie = SecTrieDBMut::new(&mut db, &mut root);
            for (key, value) in storage.iter().filter(|(_k, v)| !v.is_zero()) {
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
pub fn trie_account_rlp(info: &AccountInfo, storage: &BTreeMap<H256, U256>) -> Bytes {
    let mut stream = RlpStream::new_list(4);
    stream.append(&info.nonce);
    stream.append(&info.balance);
    stream.append(&storage_trie_db(storage).1);
    stream.append(&info.code_hash.as_bytes());
    stream.out().freeze()
}

/// `AccountInfo` account information.
#[derive(Clone, Debug, PartialEq, Eq)]
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

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            balance: U256::zero(),
            nonce: 0,
            code_hash: H256(hex!(
                "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
            )),
            code: None,
        }
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
    use super::{trie_hash_db, AccountInfo, AccountState, DbAccount};
    use hex_literal::hex;
    use primitive_types::{H160, H256, U256};
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct GenesisAccount {
        pub address: H160,
        pub balance: U256,
    }

    #[test]
    fn test_compute_state_root() {
        // Ethereum mainnet genesis accounts
        // ref: https://github.com/openethereum/parity-ethereum/blob/v3.0.1/ethcore/res/ethereum/foundation.json
        let json = include_str!("../testdata/mainnet_genesis.json");
        let genesis_accounts: Vec<GenesisAccount> = serde_json::from_str(json).unwrap();
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

        let (_, root) = trie_hash_db(&accounts);
        assert_eq!(
            root,
            H256(hex!("d7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544"))
        );
    }
}
