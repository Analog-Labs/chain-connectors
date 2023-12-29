mod types;

use anyhow::Result;
pub use ethereum_types;

use rosetta_config_astar::config as astar_config;
use rosetta_core::{
    crypto::{address::AddressFormat, Algorithm},
    BlockchainConfig, NodeUri,
};
use std::sync::Arc;
pub use types::*;

/// Retrieve the [`BlockchainConfig`] from the provided polygon `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn polygon_config(network: &str) -> Result<BlockchainConfig> {
    let (network, bip44_id, is_dev) = match network {
        "dev" => ("dev", 1, true),
        "mumbai" => ("mumbai", 1, true),
        "mainnet" => ("mainnet", 966, false),
        _ => anyhow::bail!("unsupported network: {}", network),
    };

    Ok(evm_config("polygon", network, "MATIC", bip44_id, is_dev))
}

/// Retrieve the [`BlockchainConfig`] from the provided arbitrum `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn arbitrum_config(network: &str) -> Result<BlockchainConfig> {
    let (network, bip44_id, is_dev) = match network {
        "dev" => ("dev", 1, true),
        "goerli" => ("goerli", 1, true),
        "mainnet" => ("mainnet", 42161, false),
        _ => anyhow::bail!("unsupported network: {}", network),
    };

    Ok(evm_config("arbitrum", network, "ARB", bip44_id, is_dev))
}

/// Retrieve the [`BlockchainConfig`] from the provided ethereum `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn config(network: &str) -> Result<BlockchainConfig> {
    let (network, symbol, bip44_id, is_dev) = match network {
        "dev" => ("dev", "ETH", 1, true),
        "mainnet" => ("mainnet", "ETH", 60, false),
        "goerli" => ("goerli", "TST", 1, true),
        "sepolia" => ("sepolia", "SepoliaETH", 1, true),

        // Polygon
        "polygon-local" => return polygon_config("dev"),
        "polygon" => return polygon_config("mainnet"),
        "mumbai" => return polygon_config("mumbai"),

        // Astar
        "astar-local" => return astar_config("dev"),

        // Arbitrum
        "arbitrum-local" => return arbitrum_config("dev"),
        "arbitrum" => return arbitrum_config("mainnet"),
        "arbitrum-goerli" => return arbitrum_config("goerli"),

        network => return astar_config(network),
    };

    Ok(evm_config("ethereum", network, symbol, bip44_id, is_dev))
}

fn evm_config(
    blockchain: &'static str,
    network: &'static str,
    symbol: &'static str,
    bip44_id: u32,
    is_dev: bool,
) -> BlockchainConfig {
    BlockchainConfig {
        blockchain,
        network,
        algorithm: Algorithm::EcdsaRecoverableSecp256k1,
        address_format: AddressFormat::Eip55,
        coin: bip44_id,
        bip44: true,
        utxo: false,
        currency_unit: "wei",
        currency_symbol: symbol,
        currency_decimals: 18,
        node_uri: {
            #[allow(clippy::expect_used)]
            NodeUri::parse("ws://127.0.0.1:8545/ws").expect("uri is valid; qed")
        },
        node_image: "ethereum/client-go:v1.12.2",
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
        connector_port: 8081,
        testnet: is_dev,
    }
}
