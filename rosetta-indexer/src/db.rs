use anyhow::{Context, Result};
use rosetta_client::{Chain, Client};
use rosetta_types::{
    AccountIdentifier, Block, BlockIdentifier, BlockRequest, BlockTransaction, CoinIdentifier,
    Currency, NetworkIdentifier, NetworkRequest, Operator, PartialBlockIdentifier,
    SearchTransactionsRequest, SearchTransactionsResponse, Transaction, TransactionIdentifier,
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

    pub fn iter(&self) -> impl Iterator<Item = Result<TransactionRef>> {
        self.tree
            .iter()
            .values()
            .map(|res| Ok(TransactionRef::from_bytes(&res?)))
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
        let address_len = account.address.as_bytes().len();
        self.tree
            .scan_prefix(account.address.as_bytes())
            .keys()
            .map(move |key| Ok(TransactionRef::from_bytes(&key?[address_len..])))
    }

    pub fn insert(&self, account: &AccountIdentifier, tx: &TransactionRef) -> Result<()> {
        self.tree.insert(account_table_key(account, tx), &[])?;
        Ok(())
    }

    #[allow(unused)]
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

    async fn get(&self, tx: &TransactionRef) -> Result<Option<BlockTransaction>> {
        let block = self.block(tx.block_index).await?;
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

    pub async fn transaction(
        &self,
        tx: &TransactionIdentifier,
    ) -> Result<Option<BlockTransaction>> {
        if let Some(tx) = self.transaction_table.get(tx)? {
            self.get(&tx).await
        } else {
            Ok(None)
        }
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
            self.transaction_table.set_height(block_index)?;
        }
        Ok(())
    }

    pub async fn search(
        &self,
        req: &SearchTransactionsRequest,
    ) -> Result<SearchTransactionsResponse> {
        anyhow::ensure!(req.network_identifier == self.network_identifier);

        let height = self.transaction_table.height()?;
        let max_block = req.max_block.unwrap_or(height as _) as u64;
        let mut offset = req.offset.unwrap_or(0);
        let limit = std::cmp::max(req.limit.unwrap_or(100), 1000) as usize;
        let account = if let Some(account) = &req.account_identifier {
            Some(account.clone())
        } else if let Some(address) = &req.address {
            Some(AccountIdentifier {
                address: address.clone(),
                sub_account: None,
                metadata: None,
            })
        } else {
            None
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
        let next_offset = if let Some(tx) = req.transaction_identifier.as_ref() {
            if let Some(tx) = self.transaction(&tx).await? {
                if matcher.matches(&tx.transaction) {
                    transactions.push(tx);
                }
            };
            None
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
                    block = Some(self.block(tx.block_index).await?);
                }
                let block = block.as_ref().unwrap();
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
                offset += 1;
            }
            Some(offset)
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
                    block = Some(self.block(tx.block_index).await?);
                }
                let block = block.as_ref().unwrap();
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
                offset += 1;
            }
            Some(offset)
        };
        let total_count = transactions.len() as _;
        Ok(SearchTransactionsResponse {
            transactions,
            total_count,
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
        if op.r#type.to_lowercase().contains("fail") {
            false
        } else {
            true
        }
    } else {
        false
    }
}
