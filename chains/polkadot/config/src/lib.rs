use anyhow::Result;
use rosetta_core::crypto::address::{AddressFormat, Ss58AddressFormatRegistry};
use rosetta_core::crypto::Algorithm;
use rosetta_core::BlockchainConfig;
use std::sync::Arc;

pub fn config(network: &str) -> Result<BlockchainConfig> {
    let (network, kusama) = match network {
        "dev" => ("dev", false),
        "kusama" => ("kusama", true),
        "polkadot" => ("polkadot", false),
        _ => anyhow::bail!("unsupported network"),
    };
    Ok(BlockchainConfig {
        blockchain: "polkadot",
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
        currency_symbol: if kusama { "KSM" } else { "DOT" },
        currency_decimals: if kusama { 12 } else { 10 },
        node_port: 9944,
        node_image: "parity/polkadot",
        node_command: Arc::new(|network, port| {
            vec![
                format!("--chain={}", network),
                "--rpc-cors=all".into(),
                "--ws-external".into(),
                format!("--ws-port={}", port),
                "--alice".into(),
                "--tmp".into(),
            ]
        }),
        node_additional_ports: &[],
        connector_port: 8082,
    })
}
