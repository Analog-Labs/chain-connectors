#![allow(missing_docs)]
use crate::{
    crypto::{address::Address, PublicKey},
    types::{
        Block, BlockIdentifier, CallRequest, Coin, PartialBlockIdentifier, Transaction,
        TransactionIdentifier,
    },
    Blockchain, BlockchainConfig,
};
use anyhow::Result;
use derive_more::From;
use futures::Stream;
use rosetta_core::{BlockchainClient, ClientEvent};
use rosetta_server_astar::{AstarClient, AstarMetadata, AstarMetadataParams};
use rosetta_server_bitcoin::{BitcoinClient, BitcoinMetadata, BitcoinMetadataParams};
use rosetta_server_ethereum::{
    config::{Query as EthQuery, QueryResult as EthQueryResult},
    EthereumMetadata, EthereumMetadataParams, MaybeWsEthereumClient as EthereumClient,
};
use rosetta_server_polkadot::{PolkadotClient, PolkadotMetadata, PolkadotMetadataParams};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{pin::Pin, str::FromStr};
use void::Void;

// TODO: Use
#[allow(clippy::large_enum_variant)]
/// Generic Client
pub enum GenericClient {
    Bitcoin(BitcoinClient),
    Ethereum(EthereumClient),
    Astar(AstarClient),
    Polkadot(PolkadotClient),
}

#[allow(clippy::missing_errors_doc)]
impl GenericClient {
    pub async fn new(blockchain: Blockchain, network: &str, url: &str) -> Result<Self> {
        Ok(match blockchain {
            Blockchain::Bitcoin => {
                let client = BitcoinClient::new(network, url).await?;
                Self::Bitcoin(client)
            },
            Blockchain::Ethereum => {
                let client = EthereumClient::new("ethereum", network, url).await?;
                Self::Ethereum(client)
            },
            Blockchain::Polygon => {
                let client = EthereumClient::new("polygon", network, url).await?;
                Self::Ethereum(client)
            },
            Blockchain::Arbitrum => {
                let client = EthereumClient::new("arbitrum", network, url).await?;
                Self::Ethereum(client)
            },
            Blockchain::Astar => {
                let client = AstarClient::new(network, url).await?;
                Self::Astar(client)
            },
            Blockchain::Humanode => {
                let client = AstarClient::new(network, url).await?;
                Self::Astar(client)
            },
            Blockchain::Polkadot => {
                let client = PolkadotClient::new(network, url).await?;
                Self::Polkadot(client)
            },
        })
    }

    pub async fn from_config(config: BlockchainConfig, url: &str) -> Result<Self> {
        let blockchain = Blockchain::from_str(config.blockchain)?;
        Ok(match blockchain {
            Blockchain::Bitcoin => {
                let client = BitcoinClient::from_config(config, url).await?;
                Self::Bitcoin(client)
            },
            Blockchain::Ethereum | Blockchain::Polygon | Blockchain::Arbitrum => {
                let client = EthereumClient::from_config(config, url).await?;
                Self::Ethereum(client)
            },
            Blockchain::Astar => {
                let client = AstarClient::from_config(config, url).await?;
                Self::Astar(client)
            },
            Blockchain::Humanode => {
                let client = AstarClient::from_config(config, url).await?;
                Self::Astar(client)
            },
            Blockchain::Polkadot => {
                let client = PolkadotClient::from_config(config, url).await?;
                Self::Polkadot(client)
            },
        })
    }
}

/// Generic Blockchain Params
#[derive(Deserialize, Serialize, From)]
pub enum GenericMetadataParams {
    Bitcoin(BitcoinMetadataParams),
    Ethereum(EthereumMetadataParams),
    Astar(AstarMetadataParams),
    // Humanode(AstarMetadataParams),
    Polkadot(PolkadotMetadataParams),
}

/// Generic Blockchain Metadata
#[derive(Deserialize, Serialize, From)]
pub enum GenericMetadata {
    Bitcoin(BitcoinMetadata),
    Ethereum(EthereumMetadata),
    Astar(AstarMetadata),
    // Humanode(AstarMetadata),
    Polkadot(PolkadotMetadata),
}

pub enum GenericCall {
    Bitcoin(Void),
    Ethereum(EthQuery),
    Polkadot(CallRequest),
}

#[allow(clippy::large_enum_variant)]
pub enum GenericCallResult {
    Bitcoin(()),
    Ethereum(EthQueryResult),
    Polkadot(Value),
}

macro_rules! dispatch {
    ($self:tt$($method:tt)+) => {
        match $self {
            Self::Bitcoin(client) => client$($method)*,
            Self::Ethereum(client) => client$($method)*,
            Self::Astar(client) => client$($method)*,
            Self::Polkadot(client) => client$($method)*,
            // Self::Humanode(client) => client$($method)*,
        }
    };
}

#[async_trait::async_trait]
impl BlockchainClient for GenericClient {
    type MetadataParams = GenericMetadataParams;
    type Metadata = GenericMetadata;
    type EventStream<'a> = Pin<Box<dyn Stream<Item = ClientEvent> + Send + Unpin + 'a>>;
    type Call = GenericCall;
    type CallResult = GenericCallResult;

    fn config(&self) -> &BlockchainConfig {
        dispatch!(self.config())
    }

    fn genesis_block(&self) -> &BlockIdentifier {
        dispatch!(self.genesis_block())
    }

    async fn node_version(&self) -> Result<String> {
        dispatch!(self.node_version().await)
    }

    async fn current_block(&self) -> Result<BlockIdentifier> {
        dispatch!(self.current_block().await)
    }

    async fn finalized_block(&self) -> Result<BlockIdentifier> {
        dispatch!(self.finalized_block().await)
    }

    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128> {
        dispatch!(self.balance(address, block).await)
    }

    async fn coins(&self, address: &Address, block: &BlockIdentifier) -> Result<Vec<Coin>> {
        dispatch!(self.coins(address, block).await)
    }

    async fn faucet(&self, address: &Address, param: u128) -> Result<Vec<u8>> {
        dispatch!(self.faucet(address, param).await)
    }

    async fn metadata(
        &self,
        public_key: &PublicKey,
        params: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        Ok(match (self, params) {
            (Self::Bitcoin(client), GenericMetadataParams::Bitcoin(params)) => {
                client.metadata(public_key, params).await?.into()
            },
            (Self::Ethereum(client), GenericMetadataParams::Ethereum(params)) => {
                client.metadata(public_key, params).await?.into()
            },
            (Self::Astar(client), GenericMetadataParams::Astar(params)) => {
                client.metadata(public_key, params).await?.into()
            },
            (Self::Polkadot(client), GenericMetadataParams::Polkadot(params)) => {
                client.metadata(public_key, params).await?.into()
            },
            _ => anyhow::bail!("invalid params"),
        })
    }

    async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        dispatch!(self.submit(transaction).await)
    }

    async fn block(&self, block: &PartialBlockIdentifier) -> Result<Block> {
        dispatch!(self.block(block).await)
    }

    async fn block_transaction(
        &self,
        block: &BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        dispatch!(self.block_transaction(block, tx).await)
    }

    async fn call(&self, req: &GenericCall) -> Result<GenericCallResult> {
        let result = match self {
            Self::Bitcoin(client) => match req {
                GenericCall::Bitcoin(args) => GenericCallResult::Bitcoin(client.call(args).await?),
                _ => anyhow::bail!("invalid call"),
            },
            Self::Ethereum(client) => match req {
                GenericCall::Ethereum(args) => {
                    GenericCallResult::Ethereum(client.call(args).await?)
                },
                _ => anyhow::bail!("invalid call"),
            },
            Self::Astar(client) => match req {
                GenericCall::Ethereum(args) => {
                    GenericCallResult::Ethereum(client.call(args).await?)
                },
                _ => anyhow::bail!("invalid call"),
            },
            Self::Polkadot(client) => match req {
                GenericCall::Polkadot(args) => {
                    GenericCallResult::Polkadot(client.call(args).await?)
                },
                _ => anyhow::bail!("invalid call"),
            },
        };
        Ok(result)
    }

    /// Return a stream of events, return None if the blockchain doesn't support events.
    async fn listen<'a>(&'a self) -> Result<Option<Self::EventStream<'a>>> {
        Ok(dispatch!(self
            .listen()
            .await?
            .map(|s| Pin::new(Box::new(s) as Box<dyn Stream<Item = ClientEvent> + Send + Unpin>))))
    }
}
