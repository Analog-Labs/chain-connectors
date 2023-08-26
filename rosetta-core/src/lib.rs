mod node_uri;

use crate::crypto::address::{Address, AddressFormat};
use crate::crypto::{Algorithm, PublicKey, SecretKey};
use crate::types::{
    Block, BlockIdentifier, CallRequest, Coin, Currency, CurveType, NetworkIdentifier,
    PartialBlockIdentifier, SignatureType, Transaction, TransactionIdentifier,
};
use anyhow::Result;
use async_trait::async_trait;
pub use futures_util::future;
pub use futures_util::stream;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

use futures_util::stream::Empty;

pub use node_uri::{NodeUri, NodeUriError};
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
    pub node_uri: NodeUri<'static>,
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
        self.node_uri.with_host("rosetta.analog.one").to_string()
    }

    pub fn connector_url(&self) -> String {
        format!("http://rosetta.analog.one:{}", self.connector_port)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockOrIdentifier {
    Identifier(BlockIdentifier),
    Block(Block),
}

/// Event produced by a handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientEvent {
    /// New header was appended to the chain, or a chain reorganization occur.
    NewHead(BlockOrIdentifier),

    /// A new block was finalized.
    NewFinalized(BlockOrIdentifier),

    /// Close the connection for the given reason.
    Close(String),
}

/// An empty event stream. Use this if the blockchain doesn't support events.
pub type EmptyEventStream = Empty<ClientEvent>;

#[async_trait]
pub trait BlockchainClient: Sized + Send + Sync + 'static {
    type MetadataParams: DeserializeOwned + Send + Sync + 'static;
    type Metadata: Serialize;
    type EventStream<'a>: stream::Stream<Item = ClientEvent> + Send + Unpin + 'a;

    fn create_config(network: &str) -> Result<BlockchainConfig>;
    async fn new(config: BlockchainConfig, addr: &str) -> Result<Self>;
    fn config(&self) -> &BlockchainConfig;
    fn genesis_block(&self) -> &BlockIdentifier;
    async fn node_version(&self) -> Result<String>;
    async fn current_block(&self) -> Result<BlockIdentifier>;
    async fn finalized_block(&self) -> Result<BlockIdentifier>;
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

    /// Return a stream of events, return None if the blockchain doesn't support events.
    async fn listen<'a>(&'a self) -> Result<Option<Self::EventStream<'a>>> {
        Ok(None)
    }
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

    fn method_call(
        &self,
        contract: &str,
        method: &str,
        values: &[String],
        amount: u128,
    ) -> Result<Self::MetadataParams>;

    fn deploy_contract(&self, contract_binary: Vec<u8>) -> Result<Self::MetadataParams>;

    fn create_and_sign(
        &self,
        config: &BlockchainConfig,
        metadata_params: &Self::MetadataParams,
        metdata: &Self::Metadata,
        secret_key: &SecretKey,
    ) -> Vec<u8>;
}
