//! Rosetta client.
#![deny(missing_docs)]
use crate::types::Amount;
use anyhow::{Context, Result};
use fraction::{BigDecimal, BigInt};
use std::path::Path;

pub use crate::mnemonic::{generate_mnemonic, MnemonicStore};
pub use crate::signer::{RosettaAccount, RosettaPublicKey, Signer};
pub use crate::wallet::EthereumExt;
pub use crate::wallet::Wallet;
pub use rosetta_core::{crypto, types, BlockchainConfig, TransactionBuilder};

mod mnemonic;
mod signer;
mod wallet;

/// Converts an amount to a human readable string.
pub fn amount_to_string(amount: &Amount) -> Result<String> {
    let value = BigInt::parse_bytes(amount.value.as_bytes(), 10)
        .ok_or_else(|| anyhow::anyhow!("invalid amount {:?}", amount))?;
    let decimals = BigInt::pow(&10u32.into(), amount.currency.decimals);
    let value = BigDecimal::from(value) / BigDecimal::from(decimals);
    Ok(format!("{:.256} {}", value, amount.currency.symbol))
}

/// Parses a string into an amount using the equation `amount * 10 ** decimals`.
///
/// Example:
/// `string_to_amount("1.1", 10)` converts 1.1 dot into 11_000_000_000 planc.
pub fn string_to_amount(amount: &str, decimals: u32) -> Result<u128> {
    let (amount, decimals): (u128, u32) = if let Some((main, rest)) = amount.split_once('.') {
        let decimals = decimals
            .checked_sub(rest.chars().count() as _)
            .context("too many decimals")?;
        let mut amount = main.to_string();
        amount.push_str(rest);
        (amount.parse()?, decimals)
    } else {
        (amount.parse()?, decimals)
    };
    amount
        .checked_mul(u128::pow(10, decimals))
        .context("u128 overflow")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blockchain {
    Bitcoin,
    Ethereum,
    Astar,
    Polkadot,
}

impl std::str::FromStr for Blockchain {
    type Error = anyhow::Error;

    fn from_str(blockchain &str) -> Result<Self> {
        match blockchain {
            "bitcoin" => Self::Bitcoin,
            "ethereum" => Self::Ethereum,
            "astar" => Self::Astar,
            "polkadot" => Self::Polkadot,
        }
    }
}

pub enum MultiWallet {
    Ethereum(Wallet<MaybeWsEthereumClient>),
    Astar(Wallet<MaybeWsEthereumClient>),
}

impl MultiWallet {
    pub async fn new(blockchain: Blockchain, network: &str, url: Url, keyfile: Option<&Path>) -> Result<Self> {
        let store = MnemonicStore::new(keyfile)?;
        let mnemonic = store.get_or_generate_mnemonic()?;
        let signer = Signer::new(&mnemonic, "");
        match blockchain {
            Blockchain::Ethereum => {
                let config = rosetta_server_ethereum::MaybeWsEthereumClient::create_config(network)?;
                let client = rosetta_server_ethereum::MaybeWsEthereumClient::new(config, url).await?;
                Self::Ethereum(Wallet::new(client, &signer))
            }
            Blockchain::Astar => {
                let config = rosetta_server_astar::MaybeWsEthereumClient::create_config(network)?;
                let client = rosetta_server_astar::MaybeWsEthereumClient::new(config, url).await?;
                Self::Astar(Wallet::new(client, &signer))
            }
            _ => anyhow::bail!("unsupported blockchain"),
        }
    }
}
