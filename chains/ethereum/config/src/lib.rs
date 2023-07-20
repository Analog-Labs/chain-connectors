use anyhow::Result;
use rosetta_core::crypto::address::AddressFormat;
use rosetta_core::crypto::Algorithm;
use rosetta_core::{BlockchainConfig, NodeUri};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn config(network: &str) -> Result<BlockchainConfig> {
    anyhow::ensure!(network == "dev");
    Ok(BlockchainConfig {
        blockchain: "ethereum",
        network: "dev",
        algorithm: Algorithm::EcdsaRecoverableSecp256k1,
        address_format: AddressFormat::Eip55,
        coin: 1,
        bip44: true,
        utxo: false,
        currency_unit: "wei",
        currency_symbol: "ETH",
        currency_decimals: 18,
        node_uri: NodeUri::parse("http://127.0.0.1:8545")?,
        node_image: "ethereum/client-go:v1.10.26",
        node_command: Arc::new(|_network, port| {
            vec![
                "--dev".into(),
                "--ipcdisable".into(),
                "--http".into(),
                "--http.addr=0.0.0.0".into(),
                format!("--http.port={port}"),
                "--http.vhosts=*".into(),
                "--http.api=eth,debug,admin,txpool,web3".into(),
            ]
        }),
        node_additional_ports: &[],
        connector_port: 8081,
        testnet: network == "dev",
    })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthereumMetadataParams {
    pub destination: Vec<u8>,
    pub amount: [u64; 4],
    pub data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EthereumMetadata {
    pub chain_id: u64,
    pub nonce: u64,
    pub max_priority_fee_per_gas: [u64; 4],
    pub max_fee_per_gas: [u64; 4],
    pub gas_limit: [u64; 4],
}
