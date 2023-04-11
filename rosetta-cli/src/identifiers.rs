use clap::Parser;
use rosetta_client::types::{
    AccountIdentifier, BlockIdentifier, CoinIdentifier, Currency, PartialBlockIdentifier,
    SubAccountIdentifier, TransactionIdentifier,
};

#[derive(Parser)]
pub struct AccountIdentifierOpts {
    account: Option<String>,
    #[clap(long)]
    subaccount: Option<String>,
}

impl AccountIdentifierOpts {
    pub fn account_identifier(&self) -> Option<AccountIdentifier> {
        if self.account.is_none() {
            None
        } else {
            Some(AccountIdentifier {
                address: self.account.as_ref()?.clone(),
                sub_account: self
                    .subaccount
                    .as_ref()
                    .map(|subaccount| SubAccountIdentifier {
                        address: subaccount.clone(),
                        metadata: None,
                    }),
                metadata: None,
            })
        }
    }
}

#[derive(Parser)]
pub struct BlockIdentifierOpts {
    #[clap(long)]
    index: Option<u64>,
    #[clap(name = "block", long)]
    hash: Option<String>,
}

impl BlockIdentifierOpts {
    pub fn partial_block_identifier(&self) -> PartialBlockIdentifier {
        PartialBlockIdentifier {
            index: self.index,
            hash: self.hash.clone(),
        }
    }

    pub fn block_identifier(&self) -> Option<BlockIdentifier> {
        if let (Some(index), Some(hash)) = (self.index, &self.hash) {
            Some(BlockIdentifier {
                index,
                hash: hash.clone(),
            })
        } else {
            None
        }
    }
}

#[derive(Parser)]
pub struct TransactionIdentifierOpts {
    #[clap(name = "transaction", long)]
    hash: Option<String>,
}

impl TransactionIdentifierOpts {
    pub fn transaction_identifier(&self) -> Option<TransactionIdentifier> {
        Some(TransactionIdentifier {
            hash: self.hash.as_ref()?.clone(),
        })
    }
}

#[derive(Parser)]
pub struct CoinIdentifierOpts {
    #[clap(long)]
    identifier: Option<String>,
}

impl CoinIdentifierOpts {
    pub fn coin_identifier(&self) -> Option<CoinIdentifier> {
        Some(CoinIdentifier {
            identifier: self.identifier.as_ref()?.clone(),
        })
    }
}

#[derive(Parser)]
pub struct CurrencyIdentifierOpts {
    #[clap(long)]
    symbol: Option<String>,
    #[clap(long)]
    decimals: Option<u32>,
}

impl CurrencyIdentifierOpts {
    pub fn currency_identifier(&self) -> Option<Currency> {
        Some(Currency {
            symbol: self.symbol.as_ref()?.clone(),
            decimals: *self.decimals.as_ref()?,
            metadata: None,
        })
    }
}
