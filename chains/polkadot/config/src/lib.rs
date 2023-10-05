use anyhow::Result;
use rosetta_core::{
    crypto::{
        address::{AddressFormat, Ss58AddressFormatRegistry},
        Algorithm,
    },
    BlockchainConfig, NodeUri,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use subxt::ext::sp_core::crypto::Ss58AddressFormat;

// Generate an interface that we can use from the node's metadata.
pub mod metadata {
    #[subxt::subxt(runtime_metadata_path = "res/polkadot-v9430.scale")]
    pub mod dev {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolkadotNetworkProperties {
    blockchain: &'static str,
    network: &'static str,
    symbol: &'static str,
    bip44_id: u32,
    decimals: u32,
    ss58_format: Ss58AddressFormat,
}

impl TryFrom<&str> for PolkadotNetworkProperties {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        // To see all available blockchains and networks, see:
        // https://github.com/paritytech/polkadot/blob/v1.0.0/cli/src/command.rs#L93-L145
        // All blockchains in polkadot have "dev", "local" and "staging" variants

        // "dev" and "polkadot-dev" are the same
        let chain = if value == "dev" { "polkadot-dev" } else { value };

        // Split blockchain and network
        let (blockchain, network) = chain.split_once('-').unwrap_or((chain, ""));

        // Convert blockchain to &'static str
        let blockchain = match blockchain {
            "polkadot" => "polkadot",
            "kusama" => "kusama",
            "rococo" => "rococo",
            "westend" => "westend",
            "wococo" => "wococo",
            "versi" => "versi",
            _ => anyhow::bail!("unsupported blockchain: {}", blockchain),
        };

        // Convert network to &'static str
        let network = match network {
            "" => "mainnet",
            "dev" => "dev",
            "local" => "local",
            "staging" => "staging",
            _ => anyhow::bail!("unsupported network: {blockchain}-{network}"),
        };

        // Get blockchain parameters
        let (symbol, bip44_id, decimals, ss58_format) = match (blockchain, network) {
            // Polkadot mainnet and dev networks
            ("polkadot", "mainnet") => ("DOT", 354, 10, Ss58AddressFormatRegistry::PolkadotAccount),
            ("polkadot", _) => ("DOT", 1, 10, Ss58AddressFormatRegistry::PolkadotAccount),

            // Kusama mainnet and dev networks
            ("kusama", "mainnet") => ("KSM", 434, 12, Ss58AddressFormatRegistry::KusamaAccount),
            ("kusama", _) => ("KSM", 1, 12, Ss58AddressFormatRegistry::KusamaAccount),

            // Rococo
            ("rococo", _) => ("ROC", 1, 12, Ss58AddressFormatRegistry::SubstrateAccount),

            // Westend
            ("westend", _) => ("WND", 1, 12, Ss58AddressFormatRegistry::SubstrateAccount),

            // Wococo
            ("wococo", _) => ("WOCO", 1, 12, Ss58AddressFormatRegistry::SubstrateAccount),

            // Versi
            ("versi", _) => ("VRS", 1, 12, Ss58AddressFormatRegistry::SubstrateAccount),

            _ => anyhow::bail!("unsupported network: {network}"),
        };

        Ok(Self {
            blockchain,
            network,
            symbol,
            bip44_id,
            decimals,
            ss58_format: ss58_format.into(),
        })
    }
}

impl PolkadotNetworkProperties {
    // TODO: What is considered testnet? only local chains, or public testnets as well?
    #[must_use]
    pub fn is_testnet(&self) -> bool {
        self.network != "mainnet"
    }

    #[must_use]
    pub fn is_live(&self) -> bool {
        matches!(self.network, "mainnet" | "staging")
    }
}

/// Retrieve the [`BlockchainConfig`] from the provided `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn config(network: &str) -> Result<BlockchainConfig> {
    let properties = PolkadotNetworkProperties::try_from(network)?;

    let blockchain = properties.blockchain;
    Ok(BlockchainConfig {
        blockchain: properties.blockchain,
        network: properties.network,
        algorithm: Algorithm::Sr25519,
        address_format: AddressFormat::Ss58(properties.ss58_format),
        coin: properties.bip44_id,
        bip44: false,
        utxo: false,
        currency_unit: "planck",
        currency_symbol: properties.symbol,
        currency_decimals: properties.decimals,
        node_uri: NodeUri::parse("ws://127.0.0.1:9944")?,
        node_image: "parity/polkadot:v1.0.0",
        node_command: Arc::new(move |network, port| {
            let chain = if network == "mainnet" {
                blockchain.to_string()
            } else {
                format!("{blockchain}-{network}")
            };
            match network {
                "dev" | "local" => vec![
                    format!("--chain={chain}"),
                    format!("--rpc-port={port}"),
                    "--rpc-external".into(),
                    "--force-authoring".into(),
                    "--rpc-cors=all".into(),
                    "--alice".into(),
                    "--tmp".into(),
                    "--allow-private-ip".into(),
                    "--no-mdns".into(),
                ],
                _ => vec![
                    format!("--chain={chain}"),
                    format!("--rpc-port={port}"),
                    "--rpc-external".into(),
                    "--rpc-cors=all".into(),
                ],
            }
        }),
        node_additional_ports: &[],
        connector_port: 8082,
        testnet: properties.is_testnet(),
    })
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PolkadotMetadataParams {
    pub pallet_name: String,
    pub call_name: String,
    pub call_args: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
pub struct PolkadotMetadata {
    pub nonce: u32,
    pub spec_version: u32,
    pub transaction_version: u32,
    pub genesis_hash: [u8; 32],
    pub pallet_index: u8,
    pub call_index: u8,
    pub call_hash: [u8; 32],
}
