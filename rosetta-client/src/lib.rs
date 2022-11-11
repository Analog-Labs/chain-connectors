//! Rosetta client.
use crate::crypto::bip39::{Language, Mnemonic};
use crate::types::Amount;
use anyhow::Result;
use fraction::{BigDecimal, BigUint};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

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
