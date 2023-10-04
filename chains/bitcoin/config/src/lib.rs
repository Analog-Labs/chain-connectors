use anyhow::Result;
use rosetta_core::{
    crypto::{address::AddressFormat, Algorithm},
    BlockchainConfig, NodeUri,
};
use std::sync::Arc;

/// Retrieve the [`BlockchainConfig`] from the provided `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn config(network: &str) -> Result<BlockchainConfig> {
    let (network, symbol, bip44_id) = match network {
        "regtest" => ("regtest", "tBTC", 1),
        "mainnet" => ("mainnet", "BTC", 0),
        _ => anyhow::bail!("unsupported network: {}", network),
    };
    Ok(BlockchainConfig {
        blockchain: "bitcoin",
        network,
        algorithm: Algorithm::EcdsaSecp256k1,
        address_format: AddressFormat::Bech32("bcrt"),
        coin: bip44_id,
        bip44: true,
        utxo: true,
        currency_unit: "satoshi",
        currency_symbol: symbol,
        currency_decimals: 8,
        node_uri: NodeUri::parse("http://127.0.0.1:18443")?,
        node_image: "ruimarinho/bitcoin-core:23",
        node_command: Arc::new(|network, port| {
            let mut params: Vec<String> = vec![
                "-rpcbind=0.0.0.0".into(),
                format!("-rpcport={port}"),
                "-rpcallowip=0.0.0.0/0".into(),
                "-rpcuser=rosetta".into(),
                "-rpcpassword=rosetta".into(),
            ];
            if network == "regtest" {
                params.push("-regtest=1".into());
            }
            params
        }),
        node_additional_ports: &[],
        connector_port: 8080,
        testnet: network == "regtest",
    })
}
