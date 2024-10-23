use anyhow::Result;
use rosetta_core::{
    crypto::{address::AddressFormat, Algorithm},
    BlockchainConfig, NodeUri,
};
use std::sync::Arc;

// Generate an interface that we can use from the node's metadata.
#[cfg(feature = "humanode-metadata")]
pub mod metadata {
    #[subxt::subxt(runtime_metadata_path = "res/humanode.scale")]
    pub mod dev {}
}

/// Retrieve the [`BlockchainConfig`] from the provided `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn config(network: &str) -> Result<BlockchainConfig> {
    // All available networks are listed here:
    let (network, symbol) = match network {
        "dev" => ("dev", "LOC"),

        "humanode" => ("humanode", "hmnd"),

        "shasta" => ("shasta", "hmnd"),

        "Nile" => ("Nile", "hmnd"),

        "Tronex" => ("Tronex", "hmnd"),

        _ => anyhow::bail!("unsupported network: {}", network),
    };

    Ok(BlockchainConfig {
        blockchain: "humanode",
        network,
        algorithm: Algorithm::EcdsaRecoverableSecp256k1,
        address_format: AddressFormat::Eip55,
        coin: if network == "astar" { 592 } else { 5234 },
        bip44: true,
        utxo: false,
        currency_unit: "planck",
        currency_symbol: symbol,
        currency_decimals: 18,
        node_uri: NodeUri::parse("ws://127.0.0.1:9944")?,
        node_image: "staketechnologies/astar-collator:v5.28.0-rerun",
        node_command: Arc::new(|network, port| {
            let mut params = vec![
                format!("--chain={network}"),
                "--rpc-cors=all".into(),
                "--rpc-external".into(),
                format!("--rpc-port={port}"),
            ];
            if network.ends_with("dev") {
                params.extend_from_slice(&["--alice".into(), "--tmp".into()]);
            }
            params
        }),
        node_additional_ports: &[],
        connector_port: 8083,
        testnet: network != "humanode",
    })
}
