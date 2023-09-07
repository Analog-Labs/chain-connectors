use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use ethers_solc::{artifacts::Source, CompilerInput, Solc};
use rosetta_client::types::AccountIdentifier;
use rosetta_client::{Blockchain, Wallet};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Parser)]
pub struct Opts {
    #[clap(long)]
    pub keyfile: Option<PathBuf>,
    #[clap(long)]
    pub url: String,
    #[clap(long)]
    pub blockchain: Blockchain,
    #[clap(long, default_value = "dev")]
    pub network: String,
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Parser)]
pub enum Command {
    Pubkey,
    Account,
    Balance,
    DeployContract(DeployContractOpts),
    Faucet(FaucetOpts),
    Transfer(TransferOpts),
    MethodCall(MethodCallOpts),
}

#[derive(Parser)]
pub struct TransferOpts {
    pub account: String,
    pub amount: String,
}

#[derive(Parser)]
pub struct FaucetOpts {
    pub amount: String,
}

#[derive(Parser)]
pub struct TransactionOpts {
    pub transaction: String,
}

#[derive(Parser)]
pub struct MethodCallOpts {
    pub chain: String,
    pub contract: String,
    pub method: String,
    #[clap(value_delimiter = ' ')]
    pub params: Vec<String>,
    #[clap(long, default_value = "0")]
    pub amount: u128,
}

#[derive(Parser)]
pub struct DeployContractOpts {
    pub contract_path: String,
}

#[async_std::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opts = Opts::parse();
    let wallet = Wallet::new(
        opts.blockchain,
        &opts.network,
        &opts.url,
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
        Command::DeployContract(DeployContractOpts { contract_path }) => {
            match wallet.config().blockchain {
                "astar" | "ethereum" => {
                    let bytes = compile_file(&contract_path)?;
                    let tx_hash = wallet.eth_deploy_contract(bytes).await?;
                    let tx_receipt = wallet.eth_transaction_receipt(&tx_hash).await?;
                    let contract_address = tx_receipt
                        .get("contractAddress")
                        .and_then(|v| v.as_str().map(str::to_string))
                        .ok_or(anyhow::anyhow!("Unable to get contract address"))?;
                    println!("Deploy contract address {:?}", contract_address);
                }
                _ => {
                    anyhow::bail!("Not implemented");
                }
            }
        }
        Command::Transfer(TransferOpts { account, amount }) => {
            let amount =
                rosetta_client::string_to_amount(&amount, wallet.config().currency_decimals)?;
            let account = AccountIdentifier {
                address: account,
                sub_account: None,
                metadata: None,
            };
            let tx_hash = wallet.transfer(&account, amount).await?;
            println!("success: {}", hex::encode(tx_hash));
        }
        Command::Faucet(FaucetOpts { amount }) => match wallet.config().blockchain {
            "bitcoin" => {
                let url_str = wallet.config().node_url();
                let url_obj = match Url::parse(&url_str) {
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
                    .arg(format!("-rpcconnect={url}"))
                    .arg("-rpcuser=rosetta")
                    .arg("-rpcpassword=rosetta")
                    .arg("generatetoaddress")
                    .arg(amount)
                    .arg(&wallet.account().address)
                    .status()?;
                if !status.success() {
                    anyhow::bail!("cmd failed");
                }
            }
            _ => {
                let amount =
                    rosetta_client::string_to_amount(&amount, wallet.config().currency_decimals)?;
                let tx_hash = wallet.faucet(amount).await?;
                println!("success: {}", hex::encode(tx_hash));
            }
        },
        Command::MethodCall(MethodCallOpts {
            contract,
            method,
            params,
            amount,
            ..
        }) => {
            let tx_hash = wallet
                .eth_send_call(&contract, &method, &params, amount)
                .await?;
            println!("Transaction hash: {}", hex::encode(tx_hash));
        }
    }
    Ok(())
}

fn compile_file(path: &str) -> Result<Vec<u8>> {
    let solc = Solc::default();
    let mut sources = BTreeMap::new();
    sources.insert(Path::new(path).into(), Source::read(path).unwrap());
    let input = &CompilerInput::with_sources(sources)[0];
    let output = solc.compile_exact(input)?;
    let file = output.contracts.get(path).unwrap();
    let (key, _) = file.first_key_value().unwrap();
    let contract = file.get(key).unwrap();
    let bytecode = contract
        .evm
        .as_ref()
        .context("evm not found")?
        .bytecode
        .as_ref()
        .context("bytecode not found")?
        .object
        .as_bytes()
        .context("could not convert to bytes")?
        .to_vec();
    Ok(bytecode)
}
