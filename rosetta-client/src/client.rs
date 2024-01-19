#![allow(missing_docs)]
use crate::{
    crypto::{address::Address, PublicKey},
    types::{Block, CallRequest, Transaction, TransactionIdentifier},
    Blockchain, BlockchainConfig,
};
use anyhow::Result;
use derive_more::From;
use futures::Stream;
use rosetta_core::{BlockchainClient, ClientEvent};
use rosetta_server_astar::{AstarClient, AstarMetadata, AstarMetadataParams};
use rosetta_server_ethereum::{
    config::{Query as EthQuery, QueryResult as EthQueryResult},
    EthereumMetadata, EthereumMetadataParams, MaybeWsEthereumClient as EthereumClient,
};
use rosetta_server_polkadot::{PolkadotClient, PolkadotMetadata, PolkadotMetadataParams};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{pin::Pin, str::FromStr};

/// Generic Client
#[allow(clippy::large_enum_variant)]
pub enum GenericClient {
    Ethereum(EthereumClient),
    Astar(AstarClient),
    Polkadot(PolkadotClient),
}

#[allow(clippy::missing_errors_doc)]
impl GenericClient {
    pub async fn new(
        blockchain: Blockchain,
        network: &str,
        url: &str,
        private_key: Option<[u8; 32]>,
    ) -> Result<Self> {
        Ok(match blockchain {
            Blockchain::Ethereum => {
                let client = EthereumClient::new("ethereum", network, url, private_key).await?;
                Self::Ethereum(client)
            },
            Blockchain::Polygon => {
                let client = EthereumClient::new("polygon", network, url, private_key).await?;
                Self::Ethereum(client)
            },
            Blockchain::Arbitrum => {
                let client = EthereumClient::new("arbitrum", network, url, private_key).await?;
                Self::Ethereum(client)
            },
            Blockchain::Astar => {
                let client = AstarClient::new(network, url).await?;
                Self::Astar(client)
            },
            Blockchain::Polkadot | Blockchain::Rococo | Blockchain::Westend => {
                let client = PolkadotClient::new(network, url).await?;
                Self::Polkadot(client)
            },
            Blockchain::Kusama | Blockchain::Wococo => {
                anyhow::bail!("unsupported blockchain: {blockchain:?}")
            },
        })
    }

    pub async fn from_config(
        config: BlockchainConfig,
        url: &str,
        private_key: Option<[u8; 32]>,
    ) -> Result<Self> {
        let blockchain = Blockchain::from_str(config.blockchain)?;
        Ok(match blockchain {
            Blockchain::Ethereum | Blockchain::Polygon | Blockchain::Arbitrum => {
                let client = EthereumClient::from_config(config, url, private_key).await?;
                Self::Ethereum(client)
            },
            Blockchain::Astar => {
                let client = AstarClient::from_config(config, url).await?;
                Self::Astar(client)
            },
            Blockchain::Polkadot | Blockchain::Rococo | Blockchain::Westend => {
                let client = PolkadotClient::from_config(config, url).await?;
                Self::Polkadot(client)
            },
            Blockchain::Kusama | Blockchain::Wococo => {
                anyhow::bail!("unsupported blockchain: {blockchain:?}")
            },
        })
    }
}

/// Generic Blockchain Params
#[derive(Deserialize, Serialize, From)]
pub enum GenericMetadataParams {
    Ethereum(EthereumMetadataParams),
    Astar(AstarMetadataParams),
    Polkadot(PolkadotMetadataParams),
}

/// Generic Blockchain Metadata
#[derive(Deserialize, Serialize, From)]
pub enum GenericMetadata {
    Ethereum(EthereumMetadata),
    Astar(AstarMetadata),
    Polkadot(PolkadotMetadata),
}

pub enum GenericCall {
    Ethereum(EthQuery),
    Polkadot(CallRequest),
}

#[allow(clippy::large_enum_variant)]
pub enum GenericCallResult {
    Ethereum(EthQueryResult),
    Polkadot(Value),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenericAtBlock {
    Ethereum(<EthereumClient as BlockchainClient>::AtBlock),
    Polkadot(<PolkadotClient as BlockchainClient>::AtBlock),
}

impl From<GenericBlockIdentifier> for GenericAtBlock {
    fn from(block: GenericBlockIdentifier) -> Self {
        match block {
            GenericBlockIdentifier::Ethereum(block) => Self::Ethereum(block.into()),
            GenericBlockIdentifier::Polkadot(block) => Self::Polkadot(block.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenericBlockIdentifier {
    Ethereum(<EthereumClient as BlockchainClient>::BlockIdentifier),
    Polkadot(<PolkadotClient as BlockchainClient>::BlockIdentifier),
}

macro_rules! dispatch {
    ($self:tt$($method:tt)+) => {
        match $self {
            Self::Ethereum(client) => client$($method)*,
            Self::Astar(client) => client$($method)*,
            Self::Polkadot(client) => client$($method)*,
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

    type AtBlock = GenericAtBlock;
    type BlockIdentifier = GenericBlockIdentifier;

    fn config(&self) -> &BlockchainConfig {
        dispatch!(self.config())
    }

    fn genesis_block(&self) -> Self::BlockIdentifier {
        // dispatch!(self.genesis_block())
        match self {
            Self::Ethereum(client) => GenericBlockIdentifier::Ethereum(client.genesis_block()),
            Self::Astar(client) => GenericBlockIdentifier::Ethereum(client.genesis_block()),
            Self::Polkadot(client) => GenericBlockIdentifier::Polkadot(client.genesis_block()),
        }
    }

    async fn node_version(&self) -> Result<String> {
        dispatch!(self.node_version().await)
    }

    async fn current_block(&self) -> Result<Self::BlockIdentifier> {
        // dispatch!(self.current_block().await)
        match self {
            Self::Ethereum(client) => {
                client.current_block().await.map(GenericBlockIdentifier::Ethereum)
            },
            Self::Astar(client) => {
                client.current_block().await.map(GenericBlockIdentifier::Ethereum)
            },
            Self::Polkadot(client) => {
                client.current_block().await.map(GenericBlockIdentifier::Polkadot)
            },
        }
    }

    async fn finalized_block(&self) -> Result<Self::BlockIdentifier> {
        // dispatch!(self.finalized_block().await)
        match self {
            Self::Ethereum(client) => {
                client.finalized_block().await.map(GenericBlockIdentifier::Ethereum)
            },
            Self::Astar(client) => {
                client.finalized_block().await.map(GenericBlockIdentifier::Ethereum)
            },
            Self::Polkadot(client) => {
                client.finalized_block().await.map(GenericBlockIdentifier::Polkadot)
            },
        }
    }

    async fn balance(&self, address: &Address, block: &Self::AtBlock) -> Result<u128> {
        match self {
            Self::Ethereum(client) => match block {
                GenericAtBlock::Ethereum(at_block) => client.balance(address, at_block).await,
                GenericAtBlock::Polkadot(_) => anyhow::bail!("invalid block identifier"),
            },
            Self::Astar(client) => match block {
                GenericAtBlock::Ethereum(at_block) => client.balance(address, at_block).await,
                GenericAtBlock::Polkadot(_) => anyhow::bail!("invalid block identifier"),
            },
            Self::Polkadot(client) => match block {
                GenericAtBlock::Polkadot(at_block) => client.balance(address, at_block).await,
                GenericAtBlock::Ethereum(_) => anyhow::bail!("invalid block identifier"),
            },
        }
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

    async fn block(&self, block: &GenericAtBlock) -> Result<Block> {
        match self {
            Self::Ethereum(client) => match block {
                GenericAtBlock::Ethereum(at_block) => client.block(at_block).await,
                GenericAtBlock::Polkadot(_) => anyhow::bail!("invalid block identifier"),
            },
            Self::Astar(client) => match block {
                GenericAtBlock::Ethereum(at_block) => client.block(at_block).await,
                GenericAtBlock::Polkadot(_) => anyhow::bail!("invalid block identifier"),
            },
            Self::Polkadot(client) => match block {
                GenericAtBlock::Polkadot(at_block) => client.block(at_block).await,
                GenericAtBlock::Ethereum(_) => anyhow::bail!("invalid block identifier"),
            },
        }
    }

    async fn block_transaction(
        &self,
        block: &Self::BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        match self {
            Self::Ethereum(client) => match block {
                Self::BlockIdentifier::Ethereum(block) => client.block_transaction(block, tx).await,
                Self::BlockIdentifier::Polkadot(_) => anyhow::bail!("invalid block identifier"),
            },
            Self::Astar(client) => match block {
                Self::BlockIdentifier::Ethereum(block) => client.block_transaction(block, tx).await,
                Self::BlockIdentifier::Polkadot(_) => anyhow::bail!("invalid block identifier"),
            },
            Self::Polkadot(client) => match block {
                Self::BlockIdentifier::Polkadot(block) => client.block_transaction(block, tx).await,
                Self::BlockIdentifier::Ethereum(_) => anyhow::bail!("invalid block identifier"),
            },
        }
    }

    async fn call(&self, req: &GenericCall) -> Result<GenericCallResult> {
        let result = match self {
            Self::Ethereum(client) => match req {
                GenericCall::Ethereum(args) => {
                    GenericCallResult::Ethereum(client.call(args).await?)
                },
                GenericCall::Polkadot(_) => anyhow::bail!("invalid call"),
            },
            Self::Astar(client) => match req {
                GenericCall::Ethereum(args) => {
                    GenericCallResult::Ethereum(client.call(args).await?)
                },
                GenericCall::Polkadot(_) => anyhow::bail!("invalid call"),
            },
            Self::Polkadot(client) => match req {
                GenericCall::Polkadot(args) => {
                    GenericCallResult::Polkadot(client.call(args).await?)
                },
                GenericCall::Ethereum(_) => anyhow::bail!("invalid call"),
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
