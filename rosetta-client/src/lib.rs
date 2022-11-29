//! Rosetta client.
use crate::crypto::bip39::{Language, Mnemonic};
use crate::types::Amount;
use anyhow::Result;
use fraction::{BigDecimal, BigUint};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub use crate::client::Client;
pub use crate::config::BlockchainConfig;
pub use crate::signer::Signer;
pub use crate::tx::TransactionBuilder;
pub use crate::wallet::Wallet;
pub use rosetta_crypto as crypto;
pub use rosetta_types as types;

mod client;
mod config;
pub mod signer;
mod tx;
mod wallet;

pub fn amount_to_string(amount: &Amount) -> Result<String> {
    let value = BigUint::parse_bytes(amount.value.as_bytes(), 10)
        .ok_or_else(|| anyhow::anyhow!("invalid amount {:?}", amount))?;
    let decimals = BigUint::pow(&10u32.into(), amount.currency.decimals);
    let value = BigDecimal::from(value) / BigDecimal::from(decimals);
    Ok(format!("{:.256} {}", value, amount.currency.symbol))
}

pub fn default_keyfile() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("no config dir found"))?
        .join("rosetta-wallet")
        .join("mnemonic"))
}

pub fn open_or_create_keyfile(path: &Path) -> Result<Signer> {
    if !path.exists() {
        std::fs::create_dir_all(path.parent().unwrap())?;
        let mut entropy = [0; 32];
        getrandom::getrandom(&mut entropy)?;
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
        #[cfg(unix)]
        use std::os::unix::fs::OpenOptionsExt;
        let mut opts = OpenOptions::new();
        opts.create(true).write(true).truncate(true);
        #[cfg(unix)]
        opts.mode(0o600);
        let mut f = opts.open(path)?;
        f.write_all(mnemonic.to_string().as_bytes())?;
    }
    let mnemonic = std::fs::read_to_string(path)?;
    let mnemonic = Mnemonic::parse_in(Language::English, &mnemonic)?;
    let signer = Signer::new(&mnemonic, "")?;
    Ok(signer)
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

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}

impl Chain {
    pub fn id(self) -> &'static str {
        match self {
            Chain::Btc => "btc",
            Chain::Eth => "eth",
            Chain::Dot => "dot",
        }
    }

    pub fn url(self) -> &'static str {
        match self {
            Chain::Btc => "http://rosetta.analog.one:8080",
            Chain::Eth => "http://rosetta.analog.one:8081",
            Chain::Dot => "http://rosetta.analog.one:8082",
        }
    }

    pub fn config(self) -> BlockchainConfig {
        match self {
            Chain::Btc => BlockchainConfig::bitcoin_regtest(),
            Chain::Eth => BlockchainConfig::ethereum_dev(),
            Chain::Dot => BlockchainConfig::polkadot_dev(),
        }
    }
}

pub async fn create_wallet(
    chain: Chain,
    url: Option<&str>,
    keyfile: Option<&Path>,
) -> Result<Wallet> {
    let keyfile = if let Some(keyfile) = keyfile {
        keyfile.to_path_buf()
    } else {
        default_keyfile()?
    };
    let url = if let Some(url) = url {
        url
    } else {
        chain.url()
    };
    let signer = open_or_create_keyfile(&keyfile)?;
    let wallet = Wallet::new(url, chain.config(), &signer).await?;
    Ok(wallet)
}
