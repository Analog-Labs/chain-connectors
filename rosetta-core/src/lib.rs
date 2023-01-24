use crate::crypto::address::AddressFormat;
use crate::crypto::Algorithm;
use crate::types::{BlockIdentifier, Currency, NetworkIdentifier};
use anyhow::Result;
use std::sync::Arc;

pub use rosetta_crypto as crypto;
pub use rosetta_types as types;

#[derive(Clone)]
pub struct BlockchainConfig {
    pub blockchain: &'static str,
    pub network: &'static str,
    pub algorithm: Algorithm,
    pub address_format: AddressFormat,
    pub coin: u32,
    pub bip44: bool,
    pub utxo: bool,
    pub currency_unit: &'static str,
    pub currency_symbol: &'static str,
    pub currency_decimals: u32,
    pub node_port: u16,
    pub node_image: &'static str,
    pub node_command: Arc<dyn Fn(&str, u16) -> Vec<String> + Send + Sync + 'static>,
    pub node_additional_ports: &'static [u16],
    pub connector_port: u16,
}

impl BlockchainConfig {
    pub fn network(&self) -> NetworkIdentifier {
        NetworkIdentifier {
            blockchain: self.blockchain.into(),
            network: self.network.into(),
            sub_network_identifier: None,
        }
    }

    pub fn currency(&self) -> Currency {
        Currency {
            symbol: self.currency_symbol.into(),
            decimals: self.currency_decimals,
            metadata: None,
        }
    }

    pub fn node_url(&self) -> String {
        format!("http://rosetta.analog.one:{}", self.node_port)
    }

    pub fn connector_url(&self) -> String {
        format!("http://rosetta.analog.one:{}", self.connector_port)
    }
}

#[async_trait::async_trait]
pub trait BlockchainClient: Sized + Send + Sync + 'static {
    async fn new(network: &str, addr: &str) -> Result<Self>;
    fn config(&self) -> &BlockchainConfig;
    fn node_version(&self) -> &str;
    fn genesis_block(&self) -> &BlockIdentifier;
    async fn current_block(&self) -> Result<BlockIdentifier>;
}
