use crate::crypto::address::{Address, AddressFormat};
use crate::crypto::{Algorithm, PublicKey, SecretKey};
use crate::types::{
    Block, BlockIdentifier, CallRequest, Coin, Currency, CurveType, NetworkIdentifier,
    PartialBlockIdentifier, SignatureType, Transaction, TransactionIdentifier,
};
use anyhow::Result;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

pub use rosetta_crypto as crypto;
pub use rosetta_types as types;

type NodeCommand = Arc<dyn Fn(&str, u16) -> Vec<String> + Send + Sync + 'static>;

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
    pub node_command: NodeCommand,
    pub node_additional_ports: &'static [u16],
    pub connector_port: u16,
    pub testnet: bool,
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

#[async_trait]
pub trait BlockchainClient: Sized + Send + Sync + 'static {
    type MetadataParams: DeserializeOwned + Send + Sync + 'static;
    type Metadata: Serialize;
    async fn new(network: &str, addr: &str) -> Result<Self>;
    fn config(&self) -> &BlockchainConfig;
    fn genesis_block(&self) -> &BlockIdentifier;
    async fn node_version(&self) -> Result<String>;
    async fn current_block(&self) -> Result<BlockIdentifier>;
    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128>;
    async fn coins(&self, address: &Address, block: &BlockIdentifier) -> Result<Vec<Coin>>;
    async fn faucet(&self, address: &Address, param: u128) -> Result<Vec<u8>>;
    async fn metadata(
        &self,
        public_key: &PublicKey,
        params: &Self::MetadataParams,
    ) -> Result<Self::Metadata>;
    async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>>;
    async fn block(&self, block: &PartialBlockIdentifier) -> Result<Block>;
    async fn block_transaction(
        &self,
        block: &BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction>;
    async fn call(&self, req: &CallRequest) -> Result<Value>;
}

pub trait RosettaAlgorithm {
    fn to_signature_type(self) -> SignatureType;
    fn to_curve_type(self) -> CurveType;
}

impl RosettaAlgorithm for Algorithm {
    fn to_signature_type(self) -> SignatureType {
        match self {
            Algorithm::EcdsaSecp256k1 => SignatureType::Ecdsa,
            Algorithm::EcdsaRecoverableSecp256k1 => SignatureType::EcdsaRecovery,
            Algorithm::EcdsaSecp256r1 => SignatureType::Ecdsa,
            Algorithm::Ed25519 => SignatureType::Ed25519,
            Algorithm::Sr25519 => SignatureType::Sr25519,
        }
    }

    fn to_curve_type(self) -> CurveType {
        match self {
            Algorithm::EcdsaSecp256k1 => CurveType::Secp256k1,
            Algorithm::EcdsaRecoverableSecp256k1 => CurveType::Secp256k1,
            Algorithm::EcdsaSecp256r1 => CurveType::Secp256r1,
            Algorithm::Ed25519 => CurveType::Edwards25519,
            Algorithm::Sr25519 => CurveType::Schnorrkel,
        }
    }
}

pub trait TransactionBuilder: Default + Sized {
    type MetadataParams: Serialize + Clone;
    type Metadata: DeserializeOwned + Sized + Send + Sync + 'static;

    fn transfer(&self, address: &Address, amount: u128) -> Result<Self::MetadataParams>;

    fn create_and_sign(
        &self,
        config: &BlockchainConfig,
        metadata_params: &Self::MetadataParams,
        metdata: &Self::Metadata,
        secret_key: &SecretKey,
    ) -> Vec<u8>;
}
