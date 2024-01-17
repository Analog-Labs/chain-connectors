use anyhow::Result;
use rosetta_core::{
    crypto::{address::AddressFormat, Algorithm},
    BlockchainConfig, NodeUri,
};
use std::sync::Arc;

// Generate an interface that we can use from the node's metadata.
#[cfg(feature = "astar-metadata")]
pub mod metadata {
    #[subxt::subxt(runtime_metadata_path = "res/astar-dev.scale")]
    pub mod dev {}
}

/// Retrieve the [`BlockchainConfig`] from the provided `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn config(network: &str) -> Result<BlockchainConfig> {
    // All available networks are listed here:
    // https://github.com/AstarNetwork/Astar/blob/v5.15.0/bin/collator/src/command.rs#L88-L100
    // TODO: refactor, this is ugly i know, but necessary because the string must be &'static
    let (network, symbol) = match network {
        // ref: https://github.com/AstarNetwork/Astar/blob/v5.15.0/bin/collator/src/local/chain_spec.rs#L61-L63
        "dev" => ("dev", "LOC"),

        // ref: https://github.com/AstarNetwork/Astar/blob/v5.15.0/bin/collator/src/parachain/chain_spec/astar.rs#L55-L57
        "astar" => ("astar", "ASTR"),
        "astar-dev" => ("astar-dev", "ASTR"),

        // ref: https://github.com/AstarNetwork/Astar/blob/v5.15.0/bin/collator/src/parachain/chain_spec/shibuya.rs#L59-L61
        "shibuya" => ("shibuya", "SBY"),
        "shibuya-dev" => ("shibuya-dev", "SBY"),

        // ref: https://github.com/AstarNetwork/Astar/blob/v5.15.0/bin/collator/src/parachain/chain_spec/shiden.rs#L56-L58
        "shiden" => ("shiden", "SDN"),
        "shiden-dev" => ("shiden-dev", "SDN"),

        _ => anyhow::bail!("unsupported network: {}", network),
    };

    Ok(BlockchainConfig {
        blockchain: "astar",
        network,
        algorithm: Algorithm::EcdsaRecoverableSecp256k1,
        address_format: AddressFormat::Eip55,
        coin: if network == "astar" { 810 } else { 1 },
        bip44: true,
        utxo: false,
        currency_unit: "planck",
        currency_symbol: symbol,
        currency_decimals: 18,
        node_uri: NodeUri::parse("ws://127.0.0.1:9945")?,
        node_image: "staketechnologies/astar-collator:v5.28.0-rerun",
        node_command: Arc::new(|network, port| {
            let mut params = vec![
                "astar-collator".into(),
                format!("--chain={network}"),
                "--rpc-cors=all".into(),
                "--rpc-external".into(),
                format!("--rpc-port={port}"),
                "--enable-evm-rpc".into(),
            ];
            if network.ends_with("dev") {
                params.extend_from_slice(&["--alice".into(), "--tmp".into()]);
            }
            params
        }),
        node_additional_ports: &[],
        connector_port: 8083,
        testnet: network != "astar",
    })
}
