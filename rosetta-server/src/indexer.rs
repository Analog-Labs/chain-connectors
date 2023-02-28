use crate::types::{
    AccountIdentifier, Block, BlockTransaction, CoinIdentifier, Currency, Operator,
    PartialBlockIdentifier, SearchTransactionsRequest, SearchTransactionsResponse, Transaction,
    TransactionIdentifier,
};
use crate::BlockchainClient;
use anyhow::Result;
use std::ops::Deref;
use std::path::Path;

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

    fn to_bytes(self) -> [u8; 12] {
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
        Ok(if let Some(height) = self.tree.get([])? {
            u64::from_be_bytes(height[..].try_into()?)
        } else {
            0
        })
    }

    pub fn set_height(&self, height: u64) -> Result<()> {
        self.tree.insert([], &height.to_be_bytes())?;
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = Result<TransactionRef>> {
        self.tree
            .iter()
            .values()
            .map(|res| Ok(TransactionRef::from_bytes(&res?)))
    }

    pub fn get(&self, tx: &TransactionIdentifier) -> Result<Option<TransactionRef>> {
        Ok(self
            .tree
            .get(hex::decode(&tx.hash)?)?
            .map(|value| TransactionRef::from_bytes(&value)))
    }

    pub fn insert(&self, tx: &TransactionIdentifier, tx_ref: &TransactionRef) -> Result<()> {
        self.tree
            .insert(hex::decode(&tx.hash)?, &tx_ref.to_bytes()[..])?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.tree.len()
    }

    #[allow(unused)]
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

    pub fn get(&self, account: &AccountIdentifier) -> impl Iterator<Item = Result<TransactionRef>> {
        let address = preprocess_acc_address(&account.address);
        let address_len = address.as_bytes().len();
        self.tree
            .scan_prefix(address.to_lowercase().as_bytes())
            .keys()
            .map(move |key| Ok(TransactionRef::from_bytes(&key?[address_len..])))
    }

    pub fn insert(&self, account: &AccountIdentifier, tx: &TransactionRef) -> Result<()> {
        self.tree.insert(account_table_key(account, tx), &[])?;
        Ok(())
    }

    pub fn len(&self, account: &AccountIdentifier) -> usize {
        let address = preprocess_acc_address(&account.address);
        self.tree.scan_prefix(address.as_bytes()).keys().count()
    }

    #[allow(unused)]
    pub fn remove(&self, account: &AccountIdentifier, tx: &TransactionRef) -> Result<()> {
        self.tree.remove(account_table_key(account, tx))?;
        Ok(())
    }
}

fn account_table_key(account: &AccountIdentifier, tx: &TransactionRef) -> Vec<u8> {
    let address = preprocess_acc_address(&account.address);
    let address_len = address.as_bytes().len();
    let mut key = Vec::with_capacity(address_len + 12);
    key.extend(address.as_bytes());
    key.extend(tx.to_bytes());
    key
}

fn preprocess_acc_address(address: &str) -> String {
    address
        .strip_prefix("0x")
        .unwrap_or(&address)
        .to_lowercase()
}

#[derive(Clone)]
pub struct Indexer<C: BlockchainClient> {
    transaction_table: TransactionTable,
    account_table: AccountTable,
    client: C,
}

impl<C: BlockchainClient> Deref for Indexer<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<C: BlockchainClient> Indexer<C> {
    pub fn new(db: &Path, client: C) -> Result<Self> {
        let db = sled::open(db)?;
        let transaction_table = TransactionTable::new(db.open_tree("transaction_table")?);
        let account_table = AccountTable::new(db.open_tree("account_table")?);
        Ok(Self {
            transaction_table,
            account_table,
            client,
        })
    }

    async fn block_by_index(&self, index: u64) -> Result<Block> {
        self.client
            .block(&PartialBlockIdentifier {
                index: Some(index),
                hash: None,
            })
            .await
    }

    async fn get(&self, tx: &TransactionRef) -> Result<Option<BlockTransaction>> {
        let block = self.block_by_index(tx.block_index).await?;
        Ok(
            if let Some(transaction) = block
                .transactions
                .get(tx.transaction_index as usize)
                .cloned()
            {
                Some(BlockTransaction {
                    block_identifier: block.block_identifier.clone(),
                    transaction,
                })
            } else {
                None
            },
        )
    }

    async fn transaction(&self, tx: &TransactionIdentifier) -> Result<Option<BlockTransaction>> {
        if let Some(tx) = self.transaction_table.get(tx)? {
            self.get(&tx).await
        } else {
            Ok(None)
        }
    }

    pub async fn sync(&self) -> Result<()> {
        let synced_height = self.transaction_table.height()?;
        let current_height = self.client.current_block().await?.index;
        for block_index in (synced_height + 1)..current_height + 1 {
            let block = self.block_by_index(block_index).await?;
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
            log::info!("indexed blocks to {}", block_index);
            self.transaction_table.set_height(block_index)?;
        }
        Ok(())
    }

    pub async fn search(
        &self,
        req: &SearchTransactionsRequest,
    ) -> Result<SearchTransactionsResponse> {
        let height = self.transaction_table.height()?;
        let max_block = req.max_block.unwrap_or(height as _) as u64;
        let mut offset = req.offset.unwrap_or(0);
        let limit = std::cmp::min(req.limit.unwrap_or(100), 1000) as usize;
        let account = if let Some(account) = &req.account_identifier {
            Some(account.clone())
        } else {
            req.address.as_ref().map(|address| AccountIdentifier {
                address: address.clone(),
                sub_account: None,
                metadata: None,
            })
        };
        let matcher = Matcher {
            op: req.operator.unwrap_or(Operator::And),
            status: req.status.as_deref(),
            r#type: req.r#type.as_deref(),
            success: req.success,
            currency: req.currency.as_ref(),
            coin: req.coin_identifier.as_ref(),
        };

        let mut transactions = Vec::with_capacity(limit as _);
        let (next_offset, total_count) = if let Some(tx) = req.transaction_identifier.as_ref() {
            let total_count = if let Some(tx) = self.transaction(tx).await? {
                if matcher.matches(&tx.transaction) {
                    transactions.push(tx);
                }
                1
            } else {
                0
            };
            (None, total_count)
        } else if let Some(account) = account.as_ref() {
            let mut block: Option<Block> = None;
            for tx in self.account_table.get(account).skip(offset as usize) {
                let tx = tx?;
                let cached = block
                    .as_ref()
                    .map(|block| block.block_identifier.index == tx.block_index)
                    .unwrap_or_default();
                if !cached {
                    if tx.block_index > max_block {
                        break;
                    }
                    block = Some(self.block_by_index(tx.block_index).await?);
                }
                let block = block.as_ref().unwrap();
                offset += 1;
                if let Some(tx) = block.transactions.get(tx.transaction_index as usize) {
                    if matcher.matches(tx) {
                        transactions.push(BlockTransaction {
                            block_identifier: block.block_identifier.clone(),
                            transaction: tx.clone(),
                        });
                        if transactions.len() >= limit {
                            break;
                        }
                    }
                }
            }
            (Some(offset), self.account_table.len(account))
        } else {
            let mut block: Option<Block> = None;
            for tx in self.transaction_table.iter().skip(offset as usize) {
                let tx = tx?;
                let cached = block
                    .as_ref()
                    .map(|block| block.block_identifier.index == tx.block_index)
                    .unwrap_or_default();
                if !cached {
                    if tx.block_index > max_block {
                        break;
                    }
                    block = Some(self.block_by_index(tx.block_index).await?);
                }
                let block = block.as_ref().unwrap();
                offset += 1;
                if let Some(tx) = block.transactions.get(tx.transaction_index as usize) {
                    if matcher.matches(tx) {
                        transactions.push(BlockTransaction {
                            block_identifier: block.block_identifier.clone(),
                            transaction: tx.clone(),
                        });
                        if transactions.len() >= limit {
                            break;
                        }
                    }
                }
            }
            (Some(offset), self.transaction_table.len())
        };
        Ok(SearchTransactionsResponse {
            transactions,
            total_count: total_count as _,
            next_offset,
        })
    }
}

struct Matcher<'a> {
    op: Operator,
    r#type: Option<&'a str>,
    status: Option<&'a str>,
    currency: Option<&'a Currency>,
    coin: Option<&'a CoinIdentifier>,
    success: Option<bool>,
}

impl<'a> Matcher<'a> {
    fn matches(&self, tx: &Transaction) -> bool {
        let mut matches_success = false;
        if let Some(success) = self.success {
            matches_success = success == is_success(tx);
        }
        let mut matches_type = false;
        let mut matches_status = false;
        let mut matches_currency = false;
        let mut matches_coin = false;
        for op in &tx.operations {
            if let Some(ty) = self.r#type {
                if ty == op.r#type {
                    matches_type = true;
                }
            }
            if let Some(status) = self.status {
                if let Some(op_status) = op.status.as_deref() {
                    if status == op_status {
                        matches_status = true;
                    }
                }
            }
            if let Some(currency) = self.currency {
                if let Some(amount) = op.amount.as_ref() {
                    if currency == &amount.currency {
                        matches_currency = true;
                    }
                }
            }
            if let Some(coin) = self.coin {
                if let Some(coin_change) = op.coin_change.as_ref() {
                    if coin == &coin_change.coin_identifier {
                        matches_coin = true;
                    }
                }
            }
        }
        match self.op {
            Operator::And => {
                (matches_success || self.success.is_none())
                    && (matches_type || self.r#type.is_none())
                    && (matches_status || self.status.is_none())
                    && (matches_currency || self.currency.is_none())
                    && (matches_coin || self.coin.is_none())
            }
            Operator::Or => {
                matches_success
                    || matches_type
                    || matches_status
                    || matches_currency
                    || matches_coin
            }
        }
    }
}

// TODO: is this really correct?
fn is_success(tx: &Transaction) -> bool {
    if let Some(op) = tx.operations.last() {
        !op.r#type.to_lowercase().contains("fail")
    } else {
        false
    }
}
