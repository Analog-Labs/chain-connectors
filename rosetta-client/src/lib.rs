//! Rosetta client.
use crate::crypto::bip39::{Language, Mnemonic};
use crate::types::Amount;
use anyhow::Result;
use fraction::{BigDecimal, BigUint};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

pub use crate::client::Client;
pub use crate::signer::Signer;
pub use crate::tx::TransactionBuilder;
pub use crate::wallet::Wallet;
pub use rosetta_core::{crypto, types, BlockchainConfig};

mod client;
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
    Ok(dirs_next::config_dir()
        .ok_or_else(|| anyhow::anyhow!("no config dir found"))?
        .join("rosetta-wallet")
        .join("mnemonic"))
}

pub fn generate_mnemonic() -> Result<Mnemonic> {
    let mut entropy = [0; 32];
    getrandom::getrandom(&mut entropy)?;
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
    Ok(mnemonic)
}

pub fn create_keyfile(path: &Path, mnemonic: &Mnemonic) -> Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    let mut opts = OpenOptions::new();
    opts.create(true).write(true).truncate(true);
    #[cfg(unix)]
    opts.mode(0o600);
    let mut f = opts.open(path)?;
    f.write_all(mnemonic.to_string().as_bytes())?;
    Ok(())
}

pub fn open_keyfile(path: &Path) -> Result<Mnemonic> {
    let mnemonic = std::fs::read_to_string(path)?;
    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)?;
    Ok(mnemonic)
}

pub fn open_or_create_keyfile(path: Option<&Path>) -> Result<Mnemonic> {
    let path = if let Some(path) = path {
        path.to_path_buf()
    } else {
        default_keyfile()?
    };
    if !path.exists() {
        let mnemonic = generate_mnemonic()?;
        create_keyfile(&path, &mnemonic)?;
    }
    open_keyfile(&path)
}

#[cfg(target_family = "wasm")]
pub fn get_or_set_mnemonic() -> Result<Mnemonic> {
    use wasm_bindgen::{JsCast, UnwrapThrowExt};
    let local_storage = web_sys::window()
        .expect_throw("no window")
        .local_storage()
        .expect_throw("failed to get local_storage")
        .expect_throw("no local storage");
    let item = local_storage
        .get_item("mnemonic")
        .expect_throw("unreachable: get_item does not throw an exception");
    let mnemonic = if let Some(mnemonic) = item {
        Mnemonic::parse_in(Language::English, &mnemonic)?
    } else {
        let mnemonic = generate_mnemonic()?;
        local_storage
            .set_item("mnemonic", &mnemonic.to_string())
            .map_err(|value| {
                anyhow::anyhow!(String::from(
                    value.dyn_into::<js_sys::Error>().unwrap().to_string()
                ))
            })?;
        mnemonic
    };
    Ok(mnemonic)
}

pub fn create_signer(_keyfile: Option<&Path>) -> Result<Signer> {
    #[cfg(not(target_family = "wasm"))]
    let mnemonic = open_or_create_keyfile(_keyfile)?;
    #[cfg(target_family = "wasm")]
    let mnemonic = get_or_set_mnemonic()?;
    Signer::new(&mnemonic, "")
}

pub fn create_config(blockchain: &str, network: &str) -> Result<BlockchainConfig> {
    match blockchain {
        "bitcoin" => rosetta_config_bitcoin::config(network),
        "ethereum" => rosetta_config_ethereum::config(network),
        "polkadot" => rosetta_config_polkadot::config(network),
        _ => anyhow::bail!("unsupported blockchain"),
    }
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
