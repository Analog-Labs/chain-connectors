//! Rosetta client.
#![deny(missing_docs)]
use crate::types::Amount;
use anyhow::{Context, Result};
use fraction::{BigDecimal, BigInt};
use std::path::Path;

pub use crate::client::Client;
pub use crate::mnemonic::{generate_mnemonic, MnemonicStore};
pub use crate::signer::{RosettaAccount, RosettaPublicKey, Signer};
pub use crate::wallet::EthereumExt;
pub use crate::wallet::Wallet;
use rosetta_core::BlockchainClient;
pub use rosetta_core::{crypto, types, BlockchainConfig, TransactionBuilder};

mod client;
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

/// Returns a blockchain config for a given blockchain and network.
pub fn create_config(blockchain: &str, network: &str) -> Result<BlockchainConfig> {
    match blockchain {
        "bitcoin" => rosetta_config_bitcoin::config(network),
        "ethereum" => rosetta_config_ethereum::config(network),
        "astar" => rosetta_config_astar::config(network),
        "polkadot" => rosetta_config_polkadot::config(network),
        _ => anyhow::bail!("unsupported blockchain"),
    }
}

/// Returns a signer for a given keyfile.
pub fn create_signer(keyfile: Option<&Path>) -> Result<Signer> {
    let store = MnemonicStore::new(keyfile)?;
    let mnemonic = store.get_or_generate_mnemonic()?;
    Signer::new(&mnemonic, "")
}

/// Returns a wallet instance.
/// Parameters:
/// - `blockchain`: blockchain name e.g. "bitcoin", "ethereum".
/// - `network`: network name e.g. "dev".
/// - `url`: rosetta server url.
/// - `keyfile`: path to a keyfile.
pub fn create_wallet<T: BlockchainClient>(client: T, keyfile: Option<&Path>) -> Result<Wallet<T>> {
    let signer = create_signer(keyfile)?;
    Wallet::new(client, &signer)
}
