use anyhow::Result;
use clap::Parser;
use fraction::{BigDecimal, BigUint};
use rosetta_client::types::{
    AccountBalanceRequest, AccountCoinsRequest, AccountIdentifier, Amount, MetadataRequest,
    NetworkIdentifier, NetworkRequest, PartialBlockIdentifier, SubAccountIdentifier,
    SubNetworkIdentifier,
};
use rosetta_client::Client;

#[derive(Parser)]
struct Opts {
    #[clap(long, default_value = "http://127.0.0.1:8080")]
    url: String,
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
enum Command {
    Network(NetworkOpts),
    Account(AccountOpts),
    Block(BlockOpts),
    Mempool(MempoolOpts),
}

#[derive(Parser)]
struct NetworkOpts {
    #[clap(subcommand)]
    cmd: NetworkCommand,
}

#[derive(Parser)]
enum NetworkCommand {
    List,
    Options(NetworkCommandOpts),
    Status(NetworkCommandOpts),
}

#[derive(Parser)]
struct NetworkCommandOpts {
    #[clap(flatten)]
    network: NetworkIdentifierOpts,
}

#[derive(Parser)]
struct NetworkIdentifierOpts {
    #[clap(long)]
    blockchain: Option<String>,
    #[clap(long)]
    network: Option<String>,
    #[clap(long)]
    subnetwork: Option<String>,
}

impl NetworkIdentifierOpts {
    fn network_identifier(&self) -> Option<NetworkIdentifier> {
        Some(NetworkIdentifier {
            blockchain: self.blockchain.as_ref()?.into(),
            network: self.network.as_ref()?.into(),
            sub_network_identifier: self.subnetwork.as_ref().map(|subnetwork| {
                SubNetworkIdentifier {
                    network: subnetwork.clone(),
                    metadata: None,
                }
            }),
        })
    }
}

#[derive(Parser)]
struct AccountOpts {
    #[clap(subcommand)]
    cmd: AccountCommand,
}

#[derive(Parser)]
enum AccountCommand {
    Balance(AccountBalanceCommandOpts),
    Coins(AccountCoinsCommandOpts),
}

#[derive(Parser)]
struct AccountBalanceCommandOpts {
    #[clap(flatten)]
    network: NetworkIdentifierOpts,
    #[clap(flatten)]
    account: AccountIdentifierOpts,
    #[clap(flatten)]
    block: PartialBlockIdentifierOpts,
}

#[derive(Parser)]
struct AccountCoinsCommandOpts {
    #[clap(flatten)]
    network: NetworkIdentifierOpts,
    #[clap(flatten)]
    account: AccountIdentifierOpts,
    #[clap(long)]
    include_mempool: bool,
}

#[derive(Parser)]
struct AccountIdentifierOpts {
    account: String,
    #[clap(long)]
    subaccount: Option<String>,
}

impl AccountIdentifierOpts {
    fn account_identifier(&self) -> AccountIdentifier {
        AccountIdentifier {
            address: self.account.clone(),
            sub_account: self
                .subaccount
                .as_ref()
                .map(|subaccount| SubAccountIdentifier {
                    address: subaccount.clone(),
                    metadata: None,
                }),
            metadata: None,
        }
    }
}

#[derive(Parser)]
struct PartialBlockIdentifierOpts {
    #[clap(long)]
    index: Option<u64>,
    #[clap(long)]
    hash: Option<String>,
}

impl PartialBlockIdentifierOpts {
    fn block_identifier(&self) -> Option<PartialBlockIdentifier> {
        if self.index.is_none() && self.hash.is_none() {
            return None;
        }
        Some(PartialBlockIdentifier {
            index: self.index,
            hash: self.hash.clone(),
        })
    }
}

#[derive(Parser)]
struct BlockOpts {
    transaction: Option<String>,
}

#[derive(Parser)]
struct MempoolOpts {
    transaction: Option<String>,
}

async fn network_identifier(
    client: &Client,
    opts: &NetworkIdentifierOpts,
) -> Result<NetworkIdentifier> {
    Ok(if let Some(network) = opts.network_identifier() {
        network
    } else {
        client
            .network_list(&MetadataRequest::new())
            .await?
            .network_identifiers[0]
            .clone()
    })
}

fn amount_to_string(amount: &Amount) -> Result<String> {
    let value = BigUint::parse_bytes(amount.value.as_bytes(), 10)
        .ok_or_else(|| anyhow::anyhow!("invalid amount {:?}", amount))?;
    let decimals = BigUint::pow(&10u32.into(), amount.currency.decimals.into());
    let value = BigDecimal::from(value) / BigDecimal::from(decimals);
    Ok(format!("{:.256} {}", value, amount.currency.symbol))
}

#[async_std::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let client = Client::new(&opts.url)?;

    match opts.cmd {
        Command::Network(NetworkOpts { cmd }) => match cmd {
            NetworkCommand::List => {
                let list = client.network_list(&MetadataRequest::new()).await?;
                for network in &list.network_identifiers {
                    print!("{} {}", network.blockchain, network.network);
                    if let Some(subnetwork) = network.sub_network_identifier.as_ref() {
                        print!("{}", subnetwork.network);
                    }
                    println!("");
                }
            }
            NetworkCommand::Options(opts) => {
                let network = network_identifier(&client, &opts.network).await?;
                let options = client
                    .network_options(&NetworkRequest::new(network))
                    .await?;
                println!("{:#?}", options);
            }
            NetworkCommand::Status(opts) => {
                let network = network_identifier(&client, &opts.network).await?;
                let status = client.network_status(&NetworkRequest::new(network)).await?;
                println!("{:#?}", status);
            }
        },
        Command::Account(AccountOpts { cmd }) => match cmd {
            AccountCommand::Balance(opts) => {
                let req = AccountBalanceRequest {
                    network_identifier: network_identifier(&client, &opts.network).await?,
                    account_identifier: opts.account.account_identifier(),
                    block_identifier: opts.block.block_identifier(),
                    currencies: None,
                };
                let balance = client.account_balance(req).await?;
                println!(
                    "block {} {}",
                    balance.block_identifier.index, balance.block_identifier.hash
                );
                for amount in &balance.balances {
                    println!("{}", amount_to_string(amount)?);
                }
            }
            AccountCommand::Coins(opts) => {
                let req = AccountCoinsRequest {
                    network_identifier: network_identifier(&client, &opts.network).await?,
                    account_identifier: opts.account.account_identifier(),
                    currencies: None,
                    include_mempool: opts.include_mempool,
                };
                let coins = client.account_coins(req).await?;
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
        Command::Block(opts) => {}
        Command::Mempool(opts) => {}
    }
    Ok(())
}
