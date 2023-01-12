use anyhow::{Context, Result};
use rosetta_client::{Chain, Client};
use rosetta_types::{
    AccountIdentifier, Block, BlockIdentifier, BlockRequest, NetworkIdentifier, NetworkRequest,
    Operator, PartialBlockIdentifier, SearchTransactionsRequest, SearchTransactionsResponse,
    Transaction, TransactionIdentifier,
};
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionRef {
    pub block_index: u64,
    pub transaction_index: u32,
}

impl TransactionRef {
    fn new(block_index: u64, transaction_index: u32) -> Self {
        Self {
            block_index,
            transaction_index,
        }
    }

    fn to_bytes(&self) -> [u8; 12] {
        let mut buf = [0; 12];
        buf[..8].copy_from_slice(&self.block_index.to_be_bytes()[..]);
        buf[8..].copy_from_slice(&self.transaction_index.to_be_bytes()[..]);
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut block_index = [0; 8];
        block_index.copy_from_slice(&bytes[..8]);
        let block_index = u64::from_be_bytes(block_index);
        let mut transaction_index = [0; 4];
        transaction_index.copy_from_slice(&bytes[8..]);
        let transaction_index = u32::from_be_bytes(transaction_index);
        Self::new(block_index, transaction_index)
    }
}

#[derive(Clone, Debug)]
pub struct TransactionTable {
    tree: sled::Tree,
}

impl TransactionTable {
    pub fn new(tree: sled::Tree) -> Self {
        Self { tree }
    }

    pub fn height(&self) -> Result<u64> {
        Ok(if let Some(height) = self.tree.get(&[])? {
            u64::from_be_bytes(height[..].try_into()?)
        } else {
            0
        })
    }

    pub fn set_height(&self, height: u64) -> Result<()> {
        self.tree.insert(&[], &height.to_be_bytes())?;
        Ok(())
    }

    pub fn get(&self, tx: &TransactionIdentifier) -> Result<Option<TransactionRef>> {
        Ok(
            if let Some(value) = self.tree.get(hex::decode(&tx.hash)?)? {
                Some(TransactionRef::from_bytes(&value))
            } else {
                None
            },
        )
    }

    pub fn insert(&self, tx: &TransactionIdentifier, tx_ref: &TransactionRef) -> Result<()> {
        self.tree
            .insert(hex::decode(&tx.hash)?, &tx_ref.to_bytes()[..])?;
        Ok(())
    }

    pub fn remove(&self, tx: &TransactionIdentifier) -> Result<()> {
        self.tree.remove(hex::decode(&tx.hash)?)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct AccountTable {
    tree: sled::Tree,
}

impl AccountTable {
    pub fn new(tree: sled::Tree) -> Self {
        Self { tree }
    }

    pub fn get(
        &self,
        account: &AccountIdentifier,
    ) -> impl Iterator<Item = Result<TransactionIdentifier>> {
        let address_len = account.address.as_bytes().len();
        self.tree
            .scan_prefix(account.address.as_bytes())
            .keys()
            .map(move |key| {
                Ok(TransactionIdentifier {
                    hash: hex::encode(&key?[address_len..]),
                })
            })
    }

    pub fn insert(&self, account: &AccountIdentifier, tx: &TransactionRef) -> Result<()> {
        self.tree.insert(account_table_key(account, tx), &[])?;
        Ok(())
    }

    pub fn remove(&self, account: &AccountIdentifier, tx: &TransactionRef) -> Result<()> {
        self.tree.remove(account_table_key(account, tx))?;
        Ok(())
    }
}

fn account_table_key(account: &AccountIdentifier, tx: &TransactionRef) -> Vec<u8> {
    let address_len = account.address.as_bytes().len();
    let mut key = Vec::with_capacity(address_len + 12);
    key.extend(account.address.as_bytes());
    key.extend(tx.to_bytes());
    key
}

#[derive(Clone)]
pub struct Indexer {
    transaction_table: TransactionTable,
    account_table: AccountTable,
    client: Client,
    network_identifier: NetworkIdentifier,
}

impl Indexer {
    pub fn new(db: &Path, url: Option<&str>, chain: Chain) -> Result<Self> {
        let db = sled::open(db)?;
        let url = url.unwrap_or_else(|| chain.url());
        let client = Client::new(url)?;
        let network_identifier = chain.config().network;
        let transaction_table = TransactionTable::new(db.open_tree("transaction_table")?);
        let account_table = AccountTable::new(db.open_tree("account_table")?);
        let indexer = Self {
            transaction_table,
            account_table,
            client,
            network_identifier,
        };
        let indexer2 = indexer.clone();
        tokio::task::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                if let Err(err) = indexer.sync().await {
                    log::error!("{}", err);
                }
            }
        });
        Ok(indexer2)
    }

    pub fn network(&self) -> &NetworkIdentifier {
        &self.network_identifier
    }

    pub async fn block(&self, i: u64) -> Result<Block> {
        let req = BlockRequest {
            network_identifier: self.network_identifier.clone(),
            block_identifier: PartialBlockIdentifier {
                index: Some(i),
                hash: None,
            },
        };
        self.client
            .block(&req)
            .await?
            .block
            .context("missing block")
    }

    async fn get(&self, tx: &TransactionRef) -> Result<Transaction> {
        let block = self.block(tx.block_index).await?;
        block
            .transactions
            .get(tx.transaction_index as usize)
            .context("invalid transaction ref")
            .cloned()
    }

    pub async fn transaction(&self, tx: &TransactionIdentifier) -> Result<Transaction> {
        let tx = self
            .transaction_table
            .get(tx)?
            .context("missing transaction")?;
        self.get(&tx).await
    }

    async fn status(&self) -> Result<BlockIdentifier> {
        let status = self
            .client
            .network_status(&NetworkRequest {
                network_identifier: self.network_identifier.clone(),
                metadata: None,
            })
            .await?;
        Ok(status.current_block_identifier)
    }

    async fn sync(&self) -> Result<()> {
        let synced_height = self.transaction_table.height()?;
        let current_height = self.status().await?.index;
        for block_index in synced_height..current_height {
            let block = self.block(block_index).await?;
            for (transaction_index, transaction) in block.transactions.iter().enumerate() {
                let tx = TransactionRef::new(block_index, transaction_index as _);
                self.transaction_table
                    .insert(&transaction.transaction_identifier, &tx)?;
                for op in &transaction.operations {
                    if let Some(account) = op.account.as_ref() {
                        self.account_table.insert(account, &tx)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn search(
        &self,
        req: &SearchTransactionsRequest,
    ) -> Result<SearchTransactionsResponse> {
        anyhow::ensure!(req.network_identifier == self.network_identifier);
        anyhow::ensure!(req.operator == Some(Operator::And));

        let offset = req.offset.unwrap_or(0);
        let limit = std::cmp::max(req.limit.unwrap_or(100), 1000);

        Ok(SearchTransactionsResponse {
            transactions: vec![],
            total_count: 0,
            next_offset: Some(0),
        })
    }
}
