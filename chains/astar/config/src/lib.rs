use anyhow::Result;
use rosetta_core::{
    crypto::{address::AddressFormat, Algorithm},
    BlockchainConfig, NodeUri,
};
use std::sync::Arc;

pub fn config(network: &str) -> Result<BlockchainConfig> {
    anyhow::ensure!(network == "dev");
    Ok(BlockchainConfig {
        blockchain: "astar",
        network: "dev",
        algorithm: Algorithm::EcdsaRecoverableSecp256k1,
        address_format: AddressFormat::Eip55,
        coin: 1,
        bip44: true,
        utxo: false,
        currency_unit: "planck",
        currency_symbol: "ASTR",
        currency_decimals: 18,
        node_uri: NodeUri::parse("ws://127.0.0.1:9944")?,
        node_image: "staketechnologies/astar-collator:latest",
        node_command: Arc::new(|network, port| {
            vec![
                "astar-collator".into(),
                format!("--chain={network}"),
                "--rpc-cors=all".into(),
                "--ws-external".into(),
                format!("--ws-port={port}"),
                "--alice".into(),
                "--tmp".into(),
                "--enable-evm-rpc".into(),
            ]
        }),
        node_additional_ports: &[],
        connector_port: 8083,
        testnet: network == "dev",
    })
}
