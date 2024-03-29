mod node_uri;
pub mod traits;
pub mod types;

use crate::{
    crypto::{
        address::{Address, AddressFormat},
        Algorithm, PublicKey, SecretKey,
    },
    types::{Block, CurveType, SignatureType},
};
use anyhow::Result;
use async_trait::async_trait;
pub use futures_util::{future, stream};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

use futures_util::stream::Empty;
pub use node_uri::{NodeUri, NodeUriError};
pub use rosetta_crypto as crypto;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockOrIdentifier<ID> {
    Identifier(ID),
    Block(Block),
}

impl<ID> BlockOrIdentifier<ID> {
    pub const fn from_identifier(identifier: ID) -> Self {
        Self::Identifier(identifier)
    }

    #[must_use]
    pub fn map_identifier<T, FN: FnOnce(ID) -> T>(self, map: FN) -> BlockOrIdentifier<T> {
        match self {
            Self::Identifier(id) => BlockOrIdentifier::<T>::Identifier(map(id)),
            Self::Block(block) => BlockOrIdentifier::<T>::Block(block),
        }
    }
}

impl<ID> From<ID> for BlockOrIdentifier<ID> {
    fn from(identifier: ID) -> Self {
        Self::Identifier(identifier)
    }
}

impl<T: BlockchainClient> From<Block> for BlockOrIdentifier<T> {
    fn from(block: Block) -> Self {
        Self::Block(block)
    }
}

/// Event produced by a handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientEvent<BID, EV> {
    /// New header was appended to the chain, or a chain reorganization occur.
    NewHead(BlockOrIdentifier<BID>),

    /// A new block was finalized.
    NewFinalized(BlockOrIdentifier<BID>),

    /// Blockchain specific event.
    Event(EV),

    /// Close the connection for the given reason.
    Close(String),
}

impl<BID, EV> ClientEvent<BID, EV> {
    #[must_use]
    pub fn map_block_identifier<T, FN: FnOnce(BID) -> T>(self, map: FN) -> ClientEvent<T, EV> {
        match self {
            Self::NewHead(block) => ClientEvent::NewHead(block.map_identifier(map)),
            Self::NewFinalized(block) => ClientEvent::NewFinalized(block.map_identifier(map)),
            Self::Event(event) => ClientEvent::Event(event),
            Self::Close(reason) => ClientEvent::Close(reason),
        }
    }

    #[must_use]
    pub fn map_event<T, FN: FnOnce(EV) -> T>(self, map: FN) -> ClientEvent<BID, T> {
        match self {
            Self::NewHead(block) => ClientEvent::NewHead(block),
            Self::NewFinalized(block) => ClientEvent::NewFinalized(block),
            Self::Event(event) => ClientEvent::Event(map(event)),
            Self::Close(reason) => ClientEvent::Close(reason),
        }
    }
}

/// An empty event stream. Use this if the blockchain doesn't support events.
pub type EmptyEventStream<BID, EV> = Empty<ClientEvent<BID, EV>>;

#[async_trait]
pub trait BlockchainClient: Sized + Send + Sync + 'static {
    type MetadataParams: DeserializeOwned + Serialize + Send + Sync + 'static;
    type Metadata: DeserializeOwned + Serialize + Send + Sync + 'static;
    type EventStream<'a>: stream::Stream<Item = ClientEvent<Self::BlockIdentifier, Self::Event>>
        + Send
        + Unpin
        + 'a;
    type Call: Send + Sync + Sized + 'static;
    type CallResult: Send + Sync + Sized + 'static;

    type AtBlock: Clone + Send + Sync + Sized + Eq + From<Self::BlockIdentifier> + 'static;
    type BlockIdentifier: Clone + Send + Sync + Sized + Eq + 'static;

    type Query: traits::Query;
    type Transaction: Clone + Send + Sync + Sized + Eq + 'static;
    type Subscription: Clone + Send + Sync + Sized + Eq + 'static;
    type Event: Clone + Send + Sync + Sized + Eq + 'static;
    type SubmitResult: Clone + Send + Sync + Sized + 'static;

    async fn query(&self, query: Self::Query) -> Result<<Self::Query as traits::Query>::Result>;

    fn config(&self) -> &BlockchainConfig;
    fn genesis_block(&self) -> Self::BlockIdentifier;
    async fn current_block(&self) -> Result<Self::BlockIdentifier>;
    async fn finalized_block(&self) -> Result<Self::BlockIdentifier>;
    async fn balance(&self, address: &Address, block: &Self::AtBlock) -> Result<u128>;
    async fn faucet(&self, address: &Address, param: u128) -> Result<Vec<u8>>;
    async fn metadata(
        &self,
        public_key: &PublicKey,
        params: &Self::MetadataParams,
    ) -> Result<Self::Metadata>;
    async fn submit(&self, transaction: &[u8]) -> Result<Self::SubmitResult>;
    async fn call(&self, req: &Self::Call) -> Result<Self::CallResult>;

    #[allow(clippy::missing_errors_doc)]
    async fn subscribe(&self, sub: &Self::Subscription) -> Result<u32>;

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
    type Call = <T as BlockchainClient>::Call;
    type CallResult = <T as BlockchainClient>::CallResult;

    type AtBlock = <T as BlockchainClient>::AtBlock;
    type BlockIdentifier = <T as BlockchainClient>::BlockIdentifier;

    type Query = <T as BlockchainClient>::Query;
    type Transaction = <T as BlockchainClient>::Transaction;
    type Subscription = <T as BlockchainClient>::Subscription;
    type Event = <T as BlockchainClient>::Event;
    type SubmitResult = <T as BlockchainClient>::SubmitResult;

    async fn query(&self, query: Self::Query) -> Result<<Self::Query as traits::Query>::Result> {
        BlockchainClient::query(Self::as_ref(self), query).await
    }

    fn config(&self) -> &BlockchainConfig {
        BlockchainClient::config(Self::as_ref(self))
    }

    fn genesis_block(&self) -> Self::BlockIdentifier {
        BlockchainClient::genesis_block(Self::as_ref(self))
    }

    async fn current_block(&self) -> Result<Self::BlockIdentifier> {
        BlockchainClient::current_block(Self::as_ref(self)).await
    }

    async fn finalized_block(&self) -> Result<Self::BlockIdentifier> {
        BlockchainClient::finalized_block(Self::as_ref(self)).await
    }

    async fn balance(&self, address: &Address, block: &Self::AtBlock) -> Result<u128> {
        BlockchainClient::balance(Self::as_ref(self), address, block).await
    }

    async fn faucet(&self, address: &Address, param: u128) -> Result<Vec<u8>> {
        BlockchainClient::faucet(Self::as_ref(self), address, param).await
    }

    async fn metadata(
        &self,
        public_key: &PublicKey,
        params: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        BlockchainClient::metadata(Self::as_ref(self), public_key, params).await
    }
    async fn submit(&self, transaction: &[u8]) -> Result<Self::SubmitResult> {
        BlockchainClient::submit(Self::as_ref(self), transaction).await
    }
    async fn call(&self, req: &Self::Call) -> Result<Self::CallResult> {
        BlockchainClient::call(Self::as_ref(self), req).await
    }
    /// Return a stream of events, return None if the blockchain doesn't support events.
    async fn listen<'a>(&'a self) -> Result<Option<Self::EventStream<'a>>> {
        BlockchainClient::listen(Self::as_ref(self)).await
    }

    async fn subscribe(&self, sub: &Self::Subscription) -> Result<u32> {
        BlockchainClient::subscribe(Self::as_ref(self), sub).await
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
        contract: &[u8; 20],
        data: &[u8],
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
