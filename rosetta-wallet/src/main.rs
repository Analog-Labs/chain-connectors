use anyhow::Result;
use clap::Parser;
use rosetta_client::types::AccountIdentifier;
use rosetta_client::{BlockchainConfig, Wallet};
use sp_keyring::AccountKeyring;
use std::path::PathBuf;
use std::str::FromStr;
use subxt::ext::sp_core::{Decode, Encode};
use subxt::ext::sp_runtime::{AccountId32, MultiAddress};
use subxt::tx::{PairSigner, StaticTxPayload};
use subxt::{OnlineClient, SubstrateConfig};

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
    Dot,
}

impl FromStr for Chain {
    type Err = anyhow::Error;

    fn from_str(chain: &str) -> Result<Self> {
        Ok(match chain {
            "btc" => Chain::Btc,
            "eth" => Chain::Eth,
            "dot" => Chain::Dot,
            _ => anyhow::bail!("unsupported chain {}", chain),
        })
    }
}

impl From<Chain> for BlockchainConfig {
    fn from(chain: Chain) -> Self {
        match chain {
            Chain::Btc => Self::bitcoin_regtest(),
            Chain::Eth => Self::ethereum_dev(),
            Chain::Dot => Self::polkadot_dev(),
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
            Chain::Dot => {
                faucet_substrate(&wallet.account().address, amount).await;
            }
        },
    }
    Ok(())
}

async fn faucet_substrate(address: &str, amount: u128) {
    #[derive(Decode, Encode, Debug)]
    pub struct Transfer {
        pub dest: MultiAddress<AccountId32, core::primitive::u32>,
        #[codec(compact)]
        pub value: ::core::primitive::u128,
    }

    let api = OnlineClient::<SubstrateConfig>::new().await.unwrap();
    let signer = PairSigner::<SubstrateConfig, _>::new(AccountKeyring::Alice.pair());

    let receiver_account: AccountId32 = address.parse().unwrap();
    let receiver_multiaddr: MultiAddress<AccountId32, u32> = MultiAddress::Id(receiver_account);

    let call_data = StaticTxPayload::new(
        "Balances",
        "transfer",
        Transfer {
            dest: receiver_multiaddr,
            value: amount,
        },
        [
            255u8, 181u8, 144u8, 248u8, 64u8, 167u8, 5u8, 134u8, 208u8, 20u8, 223u8, 103u8, 235u8,
            35u8, 66u8, 184u8, 27u8, 94u8, 176u8, 60u8, 233u8, 236u8, 145u8, 218u8, 44u8, 138u8,
            240u8, 224u8, 16u8, 193u8, 220u8, 95u8,
        ],
    );

    let hash = api
        .tx()
        .sign_and_submit_default(&call_data, &signer)
        .await
        .unwrap();
    println!("hash: {:?}", hash);
}
