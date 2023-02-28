//! Rosetta client.
use crate::types::Amount;
use anyhow::{Context, Result};
use fraction::{BigDecimal, BigUint};
use std::path::Path;

pub use crate::client::Client;
pub use crate::mnemonic::{generate_mnemonic, MnemonicStore};
pub use crate::signer::{RosettaAccount, RosettaPublicKey, Signer};
pub use crate::wallet::Wallet;
pub use rosetta_core::{crypto, types, BlockchainConfig, TransactionBuilder};

mod client;
mod mnemonic;
mod signer;
mod wallet;

pub fn amount_to_string(amount: &Amount) -> Result<String> {
    let value = BigUint::parse_bytes(amount.value.as_bytes(), 10)
        .ok_or_else(|| anyhow::anyhow!("invalid amount {:?}", amount))?;
    let decimals = BigUint::pow(&10u32.into(), amount.currency.decimals);
    let value = BigDecimal::from(value) / BigDecimal::from(decimals);
    Ok(format!("{:.256} {}", value, amount.currency.symbol))
}

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

pub fn create_config(blockchain: &str, network: &str) -> Result<BlockchainConfig> {
    match blockchain {
        "bitcoin" => rosetta_config_bitcoin::config(network),
        "ethereum" => rosetta_config_ethereum::config(network),
        "polkadot" => rosetta_config_polkadot::config(network),
        _ => anyhow::bail!("unsupported blockchain"),
    }
}

pub fn create_signer(_keyfile: Option<&Path>) -> Result<Signer> {
    let store = MnemonicStore::new(_keyfile)?;
    let mnemonic = store.get_or_generate_mnemonic()?;
    Signer::new(&mnemonic, "")
}

pub async fn create_client(
    blockchain: Option<String>,
    network: Option<String>,
    url: Option<String>,
) -> Result<(BlockchainConfig, Client)> {
    let (blockchain, network) = if let (Some(blockchain), Some(network)) = (blockchain, network) {
        (blockchain, network)
    } else if let Some(url) = url.as_ref() {
        let network = Client::new(url)?.network_list().await?[0].clone();
        (network.blockchain, network.network)
    } else {
        anyhow::bail!("requires url or blockchain argument");
    };
    let config = create_config(&blockchain, &network)?;
    let url = url.unwrap_or_else(|| config.connector_url());
    let client = Client::new(&url)?;
    Ok((config, client))
}

pub async fn create_wallet(
    blockchain: Option<String>,
    network: Option<String>,
    url: Option<String>,
    keyfile: Option<&Path>,
) -> Result<Wallet> {
    let (config, client) = create_client(blockchain, network, url).await?;
    let signer = create_signer(keyfile)?;
    Wallet::new(config, &signer, client)
}
