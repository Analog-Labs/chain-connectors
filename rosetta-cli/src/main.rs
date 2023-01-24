use anyhow::Result;
use args::OperatorEnum;
use clap::Parser;
use rosetta_client::types::{
    AccountBalanceRequest, AccountCoinsRequest, BlockRequest, BlockTransactionRequest,
    EventsBlocksRequest, MempoolTransactionRequest, MetadataRequest, NetworkIdentifier,
    NetworkRequest, Operator, SearchTransactionsRequest,
};
use rosetta_client::{amount_to_string, Client};

mod args;
mod identifiers;

use crate::args::{AccountCommand, AccountOpts, Command, NetworkCommand, NetworkOpts, Opts};
use crate::identifiers::NetworkIdentifierOpts;

async fn network_identifier(
    client: &Client,
    opts: &NetworkIdentifierOpts,
) -> Result<NetworkIdentifier> {
    Ok(if let Some(network) = opts.network_identifier() {
        network
    } else {
        client.network_list().await?[0].clone()
    })
}

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::parse();
    let url = if let Some(url) = opts.url.as_ref() {
        url
    } else if let Some(chain) = opts.chain {
        chain.url()
    } else {
        "http://127.0.0.1:8080"
    };
    let client = Client::new(url)?;

    match opts.cmd {
        Command::Network(NetworkOpts { cmd }) => match cmd {
            NetworkCommand::List => {
                let list = client.network_list().await?;
                for network in &list.network_identifiers {
                    print!("{} {}", network.blockchain, network.network);
                    if let Some(subnetwork) = network.sub_network_identifier.as_ref() {
                        print!("{}", subnetwork.network);
                    }
                    println!();
                }
            }
            NetworkCommand::Options(opts) => {
                let network = network_identifier(&client, &opts.network).await?;
                let options = client.network_options(&network).await?;
                println!("{:#?}", options);
            }
            NetworkCommand::Status(opts) => {
                let network = network_identifier(&client, &opts.network).await?;
                let status = client.network_status(&network).await?;
                println!("{:#?}", status);
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
                    network_identifier: network_identifier(&client, &opts.network).await?,
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
                    network_identifier: network_identifier(&client, &opts.network).await?,
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
            let network_identifier = network_identifier(&client, &opts.network).await?;
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
                println!("{:#?}", res);
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
                println!("{:#?}", res);
            }
        }
        Command::Mempool(opts) => {
            let network_identifier = network_identifier(&client, &opts.network).await?;
            if let Some(transaction_identifier) = opts.transaction.transaction_identifier() {
                let req = MempoolTransactionRequest {
                    network_identifier,
                    transaction_identifier,
                };
                let res = client.mempool_transaction(&req).await?;
                println!("{:#?}", res.transaction);
            } else {
                let res = client
                    .mempool(&NetworkRequest::new(network_identifier))
                    .await?;
                if res.transaction_identifiers.is_empty() {
                    println!("no pending transactions");
                }
                for transaction in &res.transaction_identifiers {
                    println!("{}", &transaction.hash);
                }
            }
        }
        Command::Events(opts) => {
            let req = EventsBlocksRequest {
                network_identifier: network_identifier(&client, &opts.network).await?,
                offset: opts.offset,
                limit: opts.limit,
            };
            let res = client.events_blocks(&req).await?;
            println!("{:#?}", res);
        }
        Command::Search(search_opts) => {
            let url = if let Some(url) = search_opts.indexer_url {
                url
            } else if let Some(chain) = opts.chain {
                chain.indexer_url().into()
            } else {
                anyhow::bail!("No indexer url provided");
            };
            let indexer_client = Client::new(&url)?;

            let operator = match search_opts.operator {
                Some(OperatorEnum::And) => Some(Operator::And),
                Some(OperatorEnum::Or) => Some(Operator::Or),
                None => None,
            };

            let req = SearchTransactionsRequest {
                network_identifier: network_identifier(&client, &search_opts.network).await?,
                max_block: search_opts.max_block,
                offset: search_opts.offset,
                limit: search_opts.limit,
                transaction_identifier: search_opts.transaction.transaction_identifier(),
                account_identifier: search_opts.account.account_identifier(),
                r#type: search_opts.r#type,
                success: search_opts.success,
                operator,
                coin_identifier: search_opts.coin.coin_identifier(),
                currency: search_opts.currency.currency_identifier(),
                address: search_opts.address,
                status: search_opts.status,
            };
            let res = indexer_client.search_transactions(&req).await?;
            println!("{:#?}", res);
        }
    }
    Ok(())
}
