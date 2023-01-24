use crate::identifiers::{
    AccountIdentifierOpts, BlockIdentifierOpts, CoinIdentifierOpts, CurrencyIdentifierOpts,
    TransactionIdentifierOpts,
};
use clap::{Parser, ValueEnum};

#[derive(Parser)]
pub struct Opts {
    #[clap(long)]
    pub url: Option<String>,
    #[clap(long, requires = "network")]
    pub blockchain: Option<String>,
    #[clap(long, requires = "blockchain")]
    pub network: Option<String>,
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Parser)]
pub enum Command {
    Network(NetworkOpts),
    Account(AccountOpts),
    Block(BlockOpts),
    Mempool(MempoolOpts),
    Events(EventsOpts),
    Search(SearchOpts),
}

#[derive(Parser)]
pub struct NetworkOpts {
    #[clap(subcommand)]
    pub cmd: NetworkCommand,
}

#[derive(Parser)]
pub enum NetworkCommand {
    List,
    Options,
    Status,
}

#[derive(Parser)]
pub struct AccountOpts {
    #[clap(subcommand)]
    pub cmd: AccountCommand,
}

#[derive(Parser)]
pub enum AccountCommand {
    Balance(AccountBalanceCommandOpts),
    Coins(AccountCoinsCommandOpts),
}

#[derive(Parser)]
pub struct AccountBalanceCommandOpts {
    #[clap(flatten)]
    pub account: AccountIdentifierOpts,
    #[clap(flatten)]
    pub block: BlockIdentifierOpts,
}

#[derive(Parser)]
pub struct AccountCoinsCommandOpts {
    #[clap(flatten)]
    pub account: AccountIdentifierOpts,
    #[clap(long)]
    pub include_mempool: bool,
}

#[derive(Parser)]
pub struct BlockOpts {
    #[clap(flatten)]
    pub block: BlockIdentifierOpts,
    #[clap(flatten)]
    pub transaction: TransactionIdentifierOpts,
}

#[derive(Parser)]
pub struct MempoolOpts {
    #[clap(flatten)]
    pub transaction: TransactionIdentifierOpts,
}

#[derive(Parser)]
pub struct EventsOpts {
    #[clap(long)]
    pub offset: Option<u64>,
    #[clap(long)]
    pub limit: Option<u64>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OperatorEnum {
    Or,
    And,
}

#[derive(Parser)]
pub struct SearchOpts {
    #[clap(long, value_enum)]
    pub operator: Option<OperatorEnum>,
    #[clap(long)]
    pub max_block: Option<i64>,
    #[clap(long)]
    pub offset: Option<i64>,
    #[clap(long)]
    pub limit: Option<i64>,
    #[clap(flatten)]
    pub transaction: TransactionIdentifierOpts,
    #[clap(flatten)]
    pub account: AccountIdentifierOpts,
    #[clap(flatten)]
    pub coin: CoinIdentifierOpts,
    #[clap(flatten)]
    pub currency: CurrencyIdentifierOpts,
    #[clap(long)]
    pub r#type: Option<String>,
    #[clap(long)]
    pub address: Option<String>,
    #[clap(long)]
    pub status: Option<String>,
    #[clap(long)]
    pub success: Option<bool>,
}
