use anyhow::Result;
use clap::Parser;
use rosetta_client::types::AccountIdentifier;
use rosetta_client::{BlockchainConfig, Wallet};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser)]
pub struct Opts {
    #[clap(long)]
    pub keyfile: Option<PathBuf>,
    #[clap(long)]
    pub chain: Chain,
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Parser)]
pub enum Command {
    Pubkey,
    Account,
    Balance,
    Transfer(TransferOpts),
    Faucet(FaucetOpts),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Chain {
    Btc,
    Eth,
}

impl FromStr for Chain {
    type Err = anyhow::Error;

    fn from_str(chain: &str) -> Result<Self> {
        Ok(match chain {
            "btc" => Chain::Btc,
            "eth" => Chain::Eth,
            _ => anyhow::bail!("unsupported chain {}", chain),
        })
    }
}

impl From<Chain> for BlockchainConfig {
    fn from(chain: Chain) -> Self {
        match chain {
            Chain::Btc => Self::bitcoin_regtest(),
            Chain::Eth => Self::ethereum_dev(),
        }
    }
}

#[async_std::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opts = Opts::parse();
    let config = BlockchainConfig::from(opts.chain);
    let keyfile = if let Some(keyfile) = opts.keyfile {
        keyfile
    } else {
        rosetta_client::default_keyfile()?
    };
    let signer = rosetta_client::open_or_create_keyfile(&keyfile)?;
    let wallet = Wallet::new(config, &signer).await?;

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
        Command::Faucet(FaucetOpts { amount }) => match opts.chain {
            Chain::Btc => {
                use std::process::Command;
                let status = Command::new("bitcoin-cli")
                    .arg("-regtest")
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
            Chain::Eth => {
                use std::process::Command;
                let status = Command::new("geth")
                    .arg("attach")
                    .arg("--exec")
                    .arg(format!(
                        "eth.sendTransaction({{from: eth.coinbase, to: '{}', value: {}}})",
                        &wallet.account().address,
                        amount,
                    ))
                    .arg("http://127.0.0.1:8545")
                    .status()?;
                if !status.success() {
                    anyhow::bail!("cmd failed");
                }
            }
        },
    }
    Ok(())
}
