//! Rosetta client.
#![deny(missing_docs)]
use crate::types::Amount;
use anyhow::{Context, Result};
use fraction::{BigDecimal, BigInt};

pub use crate::wallet::Wallet;
pub use rosetta_core::{crypto, types, BlockchainConfig};

mod client;
mod mnemonic;
mod signer;
mod tx_builder;
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

/// Supported chains.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blockchain {
    /// Bitcoin
    Bitcoin,
    /// Ethereum
    Ethereum,
    /// Astar
    Astar,
    /// Polkadot
    Polkadot,
}

impl std::str::FromStr for Blockchain {
    type Err = anyhow::Error;

    fn from_str(blockchain: &str) -> Result<Self> {
        Ok(match blockchain {
            "bitcoin" => Self::Bitcoin,
            "ethereum" => Self::Ethereum,
            "astar" => Self::Astar,
            "polkadot" => Self::Polkadot,
            _ => anyhow::bail!("unsupported blockchain {}", blockchain),
        })
    }
}
