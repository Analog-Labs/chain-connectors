use anyhow::Result;
use rosetta_core::crypto::address::AddressFormat;
use rosetta_core::crypto::Algorithm;
use rosetta_core::{BlockchainConfig, NodeUri};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn config(network: &str) -> Result<BlockchainConfig> {
    anyhow::ensure!(network == "dev");
    let config = match network {
        "dev" => BlockchainConfig {
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
                    "--dev.period=1".into(),
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
        },
        "mainnet" => BlockchainConfig {
            blockchain: "ethereum",
            network: "mainnet",
            algorithm: Algorithm::EcdsaRecoverableSecp256k1,
            address_format: AddressFormat::Eip55,
            coin: 60,
            bip44: true,
            utxo: false,
            currency_unit: "wei",
            currency_symbol: "ETH",
            currency_decimals: 18,
            node_uri: NodeUri::parse("http://127.0.0.1:8545")?,
            node_image: "ethereum/client-go:v1.10.26",
            node_command: Arc::new(|_network, port| {
                vec![
                    "--syncmode=full".into(),
                    "--http".into(),
                    "--http.addr=0.0.0.0".into(),
                    format!("--http.port={port}"),
                    "--http.vhosts=*".into(),
                    "--http.api=eth,debug,admin,txpool,web3".into(),
                ]
            }),
            node_additional_ports: &[],
            connector_port: 8081,
            testnet: false,
        },
        "astar" => BlockchainConfig {
            blockchain: "astar",
            // Astar networks are listed here:
            // https://github.com/AstarNetwork/Astar/blob/v5.15.0/bin/collator/src/command.rs#L56-L69
            network: "astar",
            algorithm: Algorithm::EcdsaRecoverableSecp256k1,
            address_format: AddressFormat::Eip55,
            coin: 810,
            bip44: true,
            utxo: false,
            currency_unit: "planck",
            currency_symbol: "ASTR",
            currency_decimals: 18,
            // The default RPC port is 9945
            // https://github.com/AstarNetwork/Astar/blob/v5.15.0/bin/collator/src/command.rs#L965-L967
            node_uri: NodeUri::parse("http://127.0.0.1:9945")?,
            node_image: "staketechnologies/astar-collator:v5.15.0",
            node_command: Arc::new(|network, port| {
                vec![
                    "astar-collator".into(),
                    format!("--chain=astar"),
                    "--rpc-cors=all".into(),
                    "--rpc-external".into(),
                    format!("--rpc-port={port}"),
                    "--alice".into(),
                    "--enable-evm-rpc".into(),
                ]
            }),
            node_additional_ports: &[],
            connector_port: 8083,
            testnet: false,
        },
        _ => anyhow::bail!("unsupported network: {}", network),
    };
    Ok(config)
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
