use anyhow::Result;
use rosetta_core::{
    crypto::{address::AddressFormat, Algorithm},
    BlockchainConfig, NodeUri,
};
use std::sync::Arc;

// Generate an interface that we can use from the node's metadata.
#[cfg(feature = "humanode-metadata")]
pub mod metadata {
    #[subxt::subxt(runtime_metadata_path = "res/humanode-dev.scale")]
    pub mod dev {}
}

/// Retrieve the [`BlockchainConfig`] from the provided `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn config(network: &str) -> Result<BlockchainConfig> {
    // All available networks are listed here:
    // https://github.com/humanode-network/humanode/blob/6321838585fb3d3f41a0cef349fca67644e90077/crates/humanode-peer/src/cli/root.rs#L47-L49
    let (network, symbol) = match network {
        "dev" => ("dev", "HMND"),

        _ => anyhow::bail!("unsupported network: {}", network),
    };
    Ok(BlockchainConfig {
        blockchain: "humanode",
        network,
        algorithm: Algorithm::EcdsaRecoverableSecp256k1,
        address_format: AddressFormat::Eip55,
        coin: 1,
        bip44: true,
        utxo: false,
        currency_unit: "hmnd",
        currency_symbol: symbol,
        currency_decimals: 18,
        node_uri: NodeUri::parse("ws://127.0.0.1:9944")?,
        node_image: "humanode",
        node_command: Arc::new(|network, port| {
            let mut params = vec![
                "humanode-peer".into(),
                format!("--chain={network}"),
                "--force-authoring".into(),
                "--rpc-cors=all".into(),
                // format!("--rpc-port={port}"),
                format!("--ws-port={port}"),
                "--ws-external".into(),
                "--unsafe-ws-external".into(),
                "--unsafe-rpc-external".into(),
            ];
            if network.ends_with("dev") {
                params.extend_from_slice(&["--alice".into(), "--tmp".into()]);
            }
            params
        }),
        node_additional_ports: &[],
        connector_port: 8084,
        testnet: network != "humanode",
    })
}
