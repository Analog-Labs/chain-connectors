use anyhow::Result;
use rosetta_core::crypto::address::AddressFormat;
use rosetta_core::crypto::Algorithm;
use rosetta_core::BlockchainConfig;
use std::sync::Arc;

pub fn config(network: &str) -> Result<BlockchainConfig> {
    anyhow::ensure!(network == "regtest");
    Ok(BlockchainConfig {
        blockchain: "bitcoin",
        network: "regtest",
        algorithm: Algorithm::EcdsaSecp256k1,
        address_format: AddressFormat::Bech32("bcrt"),
        coin: 1,
        bip44: true,
        utxo: true,
        currency_unit: "satoshi",
        currency_symbol: "tBTC",
        currency_decimals: 8,
        node_port: 18443,
        node_image: "ruimarinho/bitcoin-core:23",
        node_command: Arc::new(|_network, port| {
            vec![
                "-regtest=1".into(),
                "-rpcbind=0.0.0.0".into(),
                format!("-rpcport={port}"),
                "-rpcallowip=0.0.0.0/0".into(),
                "-rpcuser=rosetta".into(),
                "-rpcpassword=rosetta".into(),
            ]
        }),
        node_additional_ports: &[],
        connector_port: 8080,
        testnet: network == "regtest",
    })
}
