use anyhow::Result;
use args::OperatorEnum;
use clap::Parser;
use rosetta_client::amount_to_string;
use rosetta_client::types::{
    AccountBalanceRequest, AccountCoinsRequest, BlockRequest, BlockTransactionRequest,
    EventsBlocksRequest, MempoolTransactionRequest, Operator, SearchTransactionsRequest,
};

mod args;
mod identifiers;

use crate::args::{AccountCommand, AccountOpts, Command, NetworkCommand, NetworkOpts, Opts};

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::parse();
    let (config, client) =
        rosetta_client::create_client(opts.blockchain, opts.network, opts.url).await?;
    match opts.cmd {
        Command::Network(NetworkOpts { cmd }) => match cmd {
            NetworkCommand::List => {
                let networks = client.network_list().await?;
                for network in &networks {
                    print!("{} {}", network.blockchain, network.network);
                    if let Some(subnetwork) = network.sub_network_identifier.as_ref() {
                        print!("{}", subnetwork.network);
                    }
                    println!();
                }
            }
            NetworkCommand::Options => {
                let options = client.network_options(config.network()).await?;
                println!("{options:#?}");
            }
            NetworkCommand::Status => {
                let status = client.network_status(config.network()).await?;
                println!("{status:#?}");
            }
        },
        Command::Account(AccountOpts { cmd }) => match cmd {
            AccountCommand::Balance(opts) => {
                let account_identifier = match opts.account.account_identifier() {
                    Some(account_identifier) => account_identifier,
                    None => {
                        anyhow::bail!("No account provided");
                    }
                };
                let req = AccountBalanceRequest {
                    network_identifier: config.network(),
                    account_identifier,
                    block_identifier: opts.block.partial_block_identifier(),
                    currencies: None,
                };
                let balance = client.account_balance(&req).await?;
                println!(
                    "block {} {}",
                    balance.block_identifier.index, balance.block_identifier.hash
                );
                for amount in &balance.balances {
                    println!("{}", amount_to_string(amount)?);
                }
            }
            AccountCommand::Coins(opts) => {
                let account_identifier = match opts.account.account_identifier() {
                    Some(account_identifier) => account_identifier,
                    None => {
                        anyhow::bail!("No account provided");
                    }
                };
                let req = AccountCoinsRequest {
                    network_identifier: config.network(),
                    account_identifier,
                    currencies: None,
                    include_mempool: opts.include_mempool,
                };
                let coins = client.account_coins(&req).await?;
                println!(
                    "block {} {}",
                    coins.block_identifier.index, coins.block_identifier.hash
                );
                for coin in &coins.coins {
                    println!(
                        "{} {}",
                        coin.coin_identifier.identifier,
                        amount_to_string(&coin.amount)?
                    );
                }
            }
        },
        Command::Block(opts) => {
            let network_identifier = config.network();
            if let Some(transaction_identifier) = opts.transaction.transaction_identifier() {
                let block_identifier = opts
                    .block
                    .block_identifier()
                    .ok_or_else(|| anyhow::anyhow!("missing block identifier"))?;
                let req = BlockTransactionRequest {
                    network_identifier,
                    block_identifier,
                    transaction_identifier,
                };
                let res = client.block_transaction(&req).await?;
                println!("{res:#?}");
            } else {
                let block_identifier = opts
                    .block
                    .partial_block_identifier()
                    .ok_or_else(|| anyhow::anyhow!("missing partial block identifier"))?;
                let req = BlockRequest {
                    network_identifier,
                    block_identifier,
                };
                let res = client.block(&req).await?;
                println!("{res:#?}");
            }
        }
        Command::Mempool(opts) => {
            let network_identifier = config.network();
            if let Some(transaction_identifier) = opts.transaction.transaction_identifier() {
                let req = MempoolTransactionRequest {
                    network_identifier,
                    transaction_identifier,
                };
                let res = client.mempool_transaction(&req).await?;
                println!("{:#?}", res.transaction);
            } else {
                let res = client.mempool(network_identifier).await?;
                if res.transaction_identifiers.is_empty() {
                    println!("no pending transactions");
                }
                for transaction in &res.transaction_identifiers {
                    println!("{}", &transaction.hash);
                }
            }
        }
        Command::Events(opts) => {
            let network_identifier = config.network();
            let req = EventsBlocksRequest {
                network_identifier,
                offset: opts.offset,
                limit: opts.limit,
            };
            let res = client.events_blocks(&req).await?;
            println!("{res:#?}");
        }
        Command::Search(opts) => {
            let network_identifier = config.network();
            let operator = match opts.operator {
                Some(OperatorEnum::And) => Some(Operator::And),
                Some(OperatorEnum::Or) => Some(Operator::Or),
                None => None,
            };
            let req = SearchTransactionsRequest {
                network_identifier,
                max_block: opts.max_block,
                offset: opts.offset,
                limit: opts.limit,
                transaction_identifier: opts.transaction.transaction_identifier(),
                account_identifier: opts.account.account_identifier(),
                r#type: opts.r#type,
                success: opts.success,
                operator,
                coin_identifier: opts.coin.coin_identifier(),
                currency: opts.currency.currency_identifier(),
                address: opts.address,
                status: opts.status,
            };
            let res = client.search_transactions(&req).await?;
            println!("{res:#?}");
        }
    }
    Ok(())
}
