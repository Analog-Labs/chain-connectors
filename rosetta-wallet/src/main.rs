use anyhow::Result;
use clap::Parser;
use rosetta_client::types::AccountIdentifier;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Opts {
    #[clap(long)]
    pub keyfile: Option<PathBuf>,
    #[clap(long)]
    pub url: Option<String>,
    #[clap(long)]
    pub blockchain: Option<String>,
    #[clap(long)]
    pub network: Option<String>,
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Parser)]
pub enum Command {
    Pubkey,
    Account,
    Balance,
    Faucet(FaucetOpts),
    Transfer(TransferOpts),
    Transactions,
}

#[derive(Parser)]
pub struct TransferOpts {
    pub account: String,
    pub amount: u128,
}

#[derive(Parser)]
pub struct FaucetOpts {
    pub amount: u128,
}

#[async_std::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opts = Opts::parse();
    let wallet = rosetta_client::create_wallet(
        opts.blockchain,
        opts.network,
        opts.url,
        opts.keyfile.as_deref(),
    )
    .await?;
    match opts.cmd {
        Command::Pubkey => {
            println!("0x{}", wallet.public_key().hex_bytes);
        }
        Command::Account => {
            println!("{}", wallet.account().address);
        }
        Command::Balance => {
            let balance = wallet.balance().await?;
            println!("{}", rosetta_client::amount_to_string(&balance)?);
        }
        Command::Transfer(TransferOpts { account, amount }) => {
            let account = AccountIdentifier {
                address: account,
                sub_account: None,
                metadata: None,
            };
            let txid = wallet.transfer(&account, amount).await?;
            println!("{}", txid.hash);
        }
        Command::Faucet(FaucetOpts { amount }) => match wallet.config().blockchain {
            "bitcoin" => {
                let url_str = wallet.config().node_url();
                let url_obj = match surf::Url::parse(&url_str) {
                    Ok(url) => url,
                    Err(e) => {
                        anyhow::bail!("Url parse error: {}", e);
                    }
                };
                let url = match url_obj.host() {
                    Some(url) => url,
                    None => {
                        anyhow::bail!("Invalid Url");
                    }
                };

                use std::process::Command;
                let status = Command::new("bitcoin-cli")
                    .arg("-regtest")
                    .arg(format!("-rpcconnect={}", url))
                    .arg("-rpcuser=rosetta")
                    .arg("-rpcpassword=rosetta")
                    .arg("generatetoaddress")
                    .arg(amount.to_string())
                    .arg(&wallet.account().address)
                    .status()?;
                if !status.success() {
                    anyhow::bail!("cmd failed");
                }
            }
            "ethereum" => {
                let url_str = wallet.config().node_url();
                let url_obj = match surf::Url::parse(&url_str) {
                    Ok(url) => url,
                    Err(e) => {
                        anyhow::bail!("Url parse error: {}", e);
                    }
                };
                let url = match url_obj.host() {
                    Some(url) => format!("{}{}{}{}", url_obj.scheme(), "://", url, ":8545"),
                    None => {
                        anyhow::bail!("Invalid Url");
                    }
                };

                use std::process::Command;
                let status = Command::new("geth")
                    .arg("attach")
                    .arg("--exec")
                    .arg(format!(
                        "eth.sendTransaction({{from: eth.coinbase, to: '{}', value: {}}})",
                        &wallet.account().address,
                        amount,
                    ))
                    .arg(&url)
                    .status()?;
                if !status.success() {
                    anyhow::bail!("cmd failed");
                }
            }
            "polkadot" => {
                match wallet.faucet(amount).await {
                    Ok(data) => {
                        println!("success: {}", data.hash);
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        return Ok(());
                    }
                };
            }
            _ => anyhow::bail!("unsupported chain"),
        },
        Command::Transactions => {
            let transactions = wallet.transactions().await?;
            if transactions.transactions.is_empty() {
                println!("No transactions found");
                return Ok(());
            } else {
                println!("{: <10} | {: <20} | {: <50}", "Block", "Amount", "Account");
                for tx in transactions.transactions.iter() {
                    if let Some(metadata) = tx.transaction.metadata.clone() {
                        let (account, amount) =
                            if metadata["from"].to_string().trim_start_matches("0x")
                                == wallet.account().address.trim_start_matches("0x")
                            {
                                (
                                    format!("{}", metadata["to"]),
                                    format!("-{}", metadata["amount"]),
                                )
                            } else {
                                (
                                    format!("{}", metadata["from"]),
                                    format!("{}", metadata["amount"]),
                                )
                            };
                        println!(
                            "{: <10} | {: <20} | {: <50}",
                            tx.block_identifier.index, amount, account
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
