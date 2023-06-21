use anyhow::Result;
use rosetta_core::crypto::address::{AddressFormat, Ss58AddressFormatRegistry};
use rosetta_core::crypto::Algorithm;
use rosetta_core::BlockchainConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn config(network: &str) -> Result<BlockchainConfig> {
    let (network, kusama) = match network {
        "dev" => ("dev", false),
        _ => anyhow::bail!("unsupported network"),
    };
    Ok(BlockchainConfig {
        blockchain: "astar",
        network,
        algorithm: Algorithm::Sr25519,
        address_format: AddressFormat::Ss58(
            if kusama {
                Ss58AddressFormatRegistry::PolkadotAccount
            } else {
                Ss58AddressFormatRegistry::KusamaAccount
            }
            .into(),
        ),
        coin: 1,
        bip44: false,
        utxo: false,
        currency_unit: "planck",
        currency_symbol: "ASTR",
        currency_decimals: 18,
        node_port: 9944,
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
            ]
        }),
        node_additional_ports: &[],
        connector_port: 8083,
        testnet: network == "dev",
    })
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AstarMetadataParams {
    pub pallet_name: String,
    pub call_name: String,
    pub call_args: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
pub struct AstarMetadata {
    pub nonce: u32,
    pub spec_version: u32,
    pub transaction_version: u32,
    pub genesis_hash: [u8; 32],
    pub pallet_index: u8,
    pub call_index: u8,
    pub call_hash: [u8; 32],
}
