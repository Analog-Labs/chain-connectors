use crate::crypto::Algorithm;
use crate::crypto::address::AddressFormat;
use crate::types::{BlockIdentifier, NetworkIdentifier};
use anyhow::Result;

pub use rosetta_crypto as crypto;
pub use rosetta_types as types;

#[derive(Clone, Debug, Eq, PartialEq)]
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
    pub node_additional_ports: &'static [u16],
    pub connector_port: u16,
}

#[async_trait::async_trait]
pub trait BlockchainClient: Sized + Send + Sync + 'static {
    async fn new(network: &str, addr: &str) -> Result<Self>;
    fn network(&self) -> &NetworkIdentifier;
    fn node_version(&self) -> &str;
    fn genesis_block(&self) -> &BlockIdentifier;
    async fn current_block(&self) -> Result<BlockIdentifier>;
}

pub trait Blockchain: Sized {
    type Client: BlockchainClient;
    fn new(network: &str) -> Self;
    fn config(&self) -> &BlockchainConfig;
    fn node_command(&self) -> Vec<String>;
}
