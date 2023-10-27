use rosetta_ethereum_backend::{AccessListItem, AtBlock, EthereumRpc, TransactionCall, AccessListWithGasUsed};
use rosetta_ethereum_primitives::{Block, BlockIdentifier, H256, EIP1186ProofResponse, Address, Bytes};
use alloc::{
    vec::Vec,
    collections::BTreeMap,
    borrow::ToOwned,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDB<T> {
    client: T,
    pub accounts: BTreeMap<Address, EIP1186ProofResponse>,
    pub code: BTreeMap<H256, Bytes>,
    pub storage: BTreeMap<Address, BTreeMap<H256, H256>>,
    pub blocks_hashes: BTreeMap<u64, H256>,
}

impl<T> StateDB<T>
where
    T: EthereumRpc + Send + Sync,
{
    pub fn new(client: T) -> Self {
        Self {
            client,
            accounts: BTreeMap::new(),
            code: BTreeMap::new(),
            storage: BTreeMap::new(),
            blocks_hashes: BTreeMap::new(),
        }
    }

    pub const fn rpc(&self) -> &T {
        &self.client
    }

    pub fn clear(&mut self) {
        self.accounts.clear();
        self.blocks_hashes.clear();
        self.storage.clear();
        self.blocks_hashes.clear();
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn prefetch_state(
        &mut self,
        tx: &TransactionCall,
        at: AtBlock,
    ) -> Result<PrefetchResult, PrefetchError<T::Error>> {
        // Load block
        let Some(block) = self.client.block(at).await.map_err(PrefetchError::Client)? else {
            return Err(PrefetchError::BlockNotFound(at));
        };

        // Make sure we use the same block hash for all calls
        let at = AtBlock::At(BlockIdentifier::Hash(block.hash));

        // Store block hash
        self.blocks_hashes.insert(
            block.number.as_u64(),
            block.hash,
        );

        // Load storages
        let mut access_list = self.client.create_access_list(tx, at).await.map_err(PrefetchError::Client)?;

        // Load contract if not in the access list
        if let Some(address) = tx.to {
            if !access_list.access_list.iter().any(|item| item.address == address) {
                access_list.access_list.push(AccessListItem { address, storage_keys: Vec::with_capacity(0) });
            }
        }

        // Load accounts
        for (address, storage_keys) in access_list
            .access_list
            .iter()
            .map(|item| (item.address, &item.storage_keys))
        {
            let account = self.client.get_proof(address, storage_keys, at).await.map_err(PrefetchError::Client)?;
            let bytecode =  self.client.get_code(address, at).await.map_err(PrefetchError::Client)?;
            self.code.insert(account.code_hash, bytecode);
            self.accounts.insert(address, account);

            let storage = self.storage.entry(address).or_default();
            for key in storage_keys.iter().map(ToOwned::to_owned) {
                let value = self.client.storage(address, key, at).await.map_err(PrefetchError::Client)?;
                storage.insert(key, value);
            }
        }
        Ok(PrefetchResult {
            block,
            gas_used: u64::try_from(access_list.gas_used).unwrap_or(u64::MAX),
            access_list,
        })
    }
}

#[derive(Debug)]
#[cfg_attr(feature="std", derive(thiserror::Error))]
pub enum PrefetchError<ERR> {
    #[cfg_attr(feature="std", error("client call failed: {0}"))]
    Client(ERR),
    #[cfg_attr(feature="std", error("block not found: {0}"))]
    BlockNotFound(AtBlock),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefetchResult {
    pub block: Block<H256>,
    pub gas_used: u64,
    pub access_list: AccessListWithGasUsed,
}
