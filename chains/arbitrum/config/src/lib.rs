use std::sync::Arc;

use anyhow::Result;
use rosetta_core::{
    crypto::{address::AddressFormat, Algorithm},
    BlockchainConfig, NodeUri,
};

/// Retrieve the [`BlockchainConfig`] from the provided `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn config(network: &str) -> Result<BlockchainConfig> {
    // All available networks are listed here:
    let (network, bip44_id, is_dev) = match network {
        "dev" => ("dev", 1, true),
        "goerli" => ("goerli", 1, true),
        "mainnet" => ("mainnet", 9001, false),
        _ => anyhow::bail!("unsupported network: {}", network),
    };

    Ok(BlockchainConfig {
        blockchain: "arbitrum",
        network,
        algorithm: Algorithm::EcdsaRecoverableSecp256k1,
        address_format: AddressFormat::Eip55,
        coin: bip44_id,
        bip44: true,
        utxo: false,
        currency_unit: "Wei",
        currency_symbol: "ARB",
        currency_decimals: 18,
        node_uri: NodeUri::parse("ws://127.0.0.1:8547")?,
        node_image: "offchainlabs/arb-node:v1.0.0-2b628f8",
        node_command: Arc::new(|network, port| {
            let mut params = if network == "dev" {
                vec!["--dev".into(), "--dev.period=1".into(), "--ipcdisable".into()]
            } else {
                vec!["--syncmode=full".into()]
            };
            params.extend_from_slice(&[
                "--http".into(),
                "--http.addr=0.0.0.0".into(),
                format!("--http.port={port}"),
                "--http.vhosts=*".into(),
                "--http.corsdomain=*".into(),
                "--http.api=eth,debug,admin,txpool,web3".into(),
                "--ws".into(),
                "--ws.addr=0.0.0.0".into(),
                format!("--ws.port={port}"),
                "--ws.origins=*".into(),
                "--ws.api=eth,debug,admin,txpool,web3".into(),
                "--ws.rpcprefix=/ws".into(),
            ]);
            params
        }),
        node_additional_ports: &[],
        connector_port: 8084,
        testnet: is_dev,
    })
}
