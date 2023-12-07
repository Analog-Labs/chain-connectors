mod node_uri;

use crate::{
    crypto::{
        address::{Address, AddressFormat},
        Algorithm, PublicKey, SecretKey,
    },
    types::{
        Block, BlockIdentifier, CallRequest, Coin, Currency, CurveType, NetworkIdentifier,
        PartialBlockIdentifier, SignatureType, Transaction, TransactionIdentifier,
    },
};
use anyhow::Result;
use async_trait::async_trait;
pub use futures_util::{future, stream};
use serde::{de::DeserializeOwned, Serialize};
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
    #[must_use]
    pub fn network(&self) -> NetworkIdentifier {
        NetworkIdentifier {
            blockchain: self.blockchain.into(),
            network: self.network.into(),
            sub_network_identifier: None,
        }
    }

    #[must_use]
    pub fn currency(&self) -> Currency {
        Currency {
            symbol: self.currency_symbol.into(),
            decimals: self.currency_decimals,
            metadata: None,
        }
    }

    #[must_use]
    pub fn node_url(&self) -> String {
        self.node_uri.with_host("rosetta.analog.one").to_string()
    }

    #[must_use]
    pub fn connector_url(&self) -> String {
        format!("http://rosetta.analog.one:{}", self.connector_port)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockOrIdentifier {
    Identifier(BlockIdentifier),
    Block(Block),
}

impl From<BlockIdentifier> for BlockOrIdentifier {
    fn from(identifier: BlockIdentifier) -> Self {
        Self::Identifier(identifier)
    }
}

impl From<Block> for BlockOrIdentifier {
    fn from(block: Block) -> Self {
        Self::Block(block)
    }
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
    type MetadataParams: DeserializeOwned + Serialize + Send + Sync + 'static;
    type Metadata: DeserializeOwned + Serialize + Send + Sync + 'static;
    type EventStream<'a>: stream::Stream<Item = ClientEvent> + Send + Unpin + 'a;

    fn config(&self) -> &BlockchainConfig;
    fn genesis_block(&self) -> &BlockIdentifier;
    async fn node_version(&self) -> Result<String>;
    async fn current_block(&self) -> Result<BlockIdentifier>;
    async fn finalized_block(&self) -> Result<BlockIdentifier>;
    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128>;
    async fn coins(&self, address: &Address, block: &BlockIdentifier) -> Result<Vec<Coin>>;
    async fn faucet(
        &self,
        address: &Address,
        param: u128,
        private_key: Option<&str>,
    ) -> Result<Vec<u8>>;
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

#[async_trait]
impl<T> BlockchainClient for Arc<T>
where
    T: BlockchainClient,
{
    type MetadataParams = <T as BlockchainClient>::MetadataParams;
    type Metadata = <T as BlockchainClient>::Metadata;
    type EventStream<'a> = <T as BlockchainClient>::EventStream<'a>;

    fn config(&self) -> &BlockchainConfig {
        BlockchainClient::config(Self::as_ref(self))
    }
    fn genesis_block(&self) -> &BlockIdentifier {
        BlockchainClient::genesis_block(Self::as_ref(self))
    }
    async fn node_version(&self) -> Result<String> {
        BlockchainClient::node_version(Self::as_ref(self)).await
    }
    async fn current_block(&self) -> Result<BlockIdentifier> {
        BlockchainClient::current_block(Self::as_ref(self)).await
    }
    async fn finalized_block(&self) -> Result<BlockIdentifier> {
        BlockchainClient::finalized_block(Self::as_ref(self)).await
    }
    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128> {
        BlockchainClient::balance(Self::as_ref(self), address, block).await
    }
    async fn coins(&self, address: &Address, block: &BlockIdentifier) -> Result<Vec<Coin>> {
        BlockchainClient::coins(Self::as_ref(self), address, block).await
    }
    async fn faucet(
        &self,
        address: &Address,
        param: u128,
        private_key: Option<&str>,
    ) -> Result<Vec<u8>> {
        BlockchainClient::faucet(Self::as_ref(self), address, param, private_key).await
    }
    async fn metadata(
        &self,
        public_key: &PublicKey,
        params: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        BlockchainClient::metadata(Self::as_ref(self), public_key, params).await
    }
    async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        BlockchainClient::submit(Self::as_ref(self), transaction).await
    }
    async fn block(&self, block: &PartialBlockIdentifier) -> Result<Block> {
        BlockchainClient::block(Self::as_ref(self), block).await
    }
    async fn block_transaction(
        &self,
        block: &BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        BlockchainClient::block_transaction(Self::as_ref(self), block, tx).await
    }
    async fn call(&self, req: &CallRequest) -> Result<Value> {
        BlockchainClient::call(Self::as_ref(self), req).await
    }

    /// Return a stream of events, return None if the blockchain doesn't support events.
    async fn listen<'a>(&'a self) -> Result<Option<Self::EventStream<'a>>> {
        BlockchainClient::listen(Self::as_ref(self)).await
    }
}

pub trait RosettaAlgorithm {
    fn to_signature_type(self) -> SignatureType;
    fn to_curve_type(self) -> CurveType;
}

impl RosettaAlgorithm for Algorithm {
    fn to_signature_type(self) -> SignatureType {
        match self {
            Self::EcdsaSecp256k1 | Self::EcdsaSecp256r1 => SignatureType::Ecdsa,
            Self::EcdsaRecoverableSecp256k1 => SignatureType::EcdsaRecovery,
            Self::Ed25519 => SignatureType::Ed25519,
            Self::Sr25519 => SignatureType::Sr25519,
        }
    }

    fn to_curve_type(self) -> CurveType {
        match self {
            Self::EcdsaSecp256k1 | Self::EcdsaRecoverableSecp256k1 => CurveType::Secp256k1,
            Self::EcdsaSecp256r1 => CurveType::Secp256r1,
            Self::Ed25519 => CurveType::Edwards25519,
            Self::Sr25519 => CurveType::Schnorrkel,
        }
    }
}

pub trait TransactionBuilder: Default + Sized {
    type MetadataParams: Serialize + Clone;
    type Metadata: DeserializeOwned + Sized + Send + Sync + 'static;

    /// Returns the transfer metadata parameters
    ///
    /// # Errors
    /// Returns `Err` if for some reason it cannot construct the metadata parameters.
    fn transfer(&self, address: &Address, amount: u128) -> Result<Self::MetadataParams>;

    /// Returns the call metadata parameters
    ///
    /// # Errors
    /// Returns `Err` if for some reason it cannot construct the metadata parameters.
    fn method_call(
        &self,
        contract: &str,
        method: &str,
        values: &[String],
        amount: u128,
    ) -> Result<Self::MetadataParams>;

    /// Retrieve the metadata parameters for deploying a smart-contract
    ///
    /// # Errors
    /// Returns `Err` if for some reason it cannot construct the metadata parameters.
    fn deploy_contract(&self, contract_binary: Vec<u8>) -> Result<Self::MetadataParams>;

    fn create_and_sign(
        &self,
        config: &BlockchainConfig,
        metadata_params: &Self::MetadataParams,
        metdata: &Self::Metadata,
        secret_key: &SecretKey,
    ) -> Vec<u8>;
}
