#![allow(missing_docs)]
use crate::{
    crypto::{address::Address, PublicKey},
    types::CallRequest,
    Blockchain, BlockchainConfig,
};
use anyhow::Result;
use derive_more::From;
use futures::Stream;
use futures_util::StreamExt;
use rosetta_core::{
    types::{BlockIdentifier, PartialBlockIdentifier},
    BlockchainClient, ClientEvent,
};
use rosetta_server_astar::{AstarClient, AstarMetadata, AstarMetadataParams};
use rosetta_server_ethereum::{
    config::{
        CallResult, Query as EthQuery, QueryResult as EthQueryResult, TransactionReceipt, H256,
    },
    EthereumMetadata, EthereumMetadataParams, MaybeWsEthereumClient as EthereumClient,
    SubmitResult,
};
use rosetta_server_polkadot::{PolkadotClient, PolkadotMetadata, PolkadotMetadataParams};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{pin::Pin, str::FromStr, task::Poll};

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
            Blockchain::Binance => {
                let client = EthereumClient::new("binance", network, url, private_key).await?;
                Self::Ethereum(client)
            },
            Blockchain::Avalanche => {
                let client = EthereumClient::new("avalanche", network, url, private_key).await?;
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
            Blockchain::Ethereum |
            Blockchain::Polygon |
            Blockchain::Arbitrum |
            Blockchain::Binance | Blockchain::Avalanche => {
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
pub enum GenericTransaction {
    Ethereum(<EthereumClient as BlockchainClient>::Transaction),
    Polkadot(<PolkadotClient as BlockchainClient>::Transaction),
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
    type EventStream<'a> = GenericClientStream<'a>;
    type Call = GenericCall;
    type CallResult = GenericCallResult;

    type AtBlock = PartialBlockIdentifier;
    type BlockIdentifier = BlockIdentifier;

    type Query = ();
    type Transaction = GenericTransaction;
    type Subscription = GenericClientSubscription;
    type Event = GenericClientEvent;
    type SubmitResult = rosetta_server_ethereum::SubmitResult;

    async fn query(
        &self,
        _query: Self::Query,
    ) -> Result<<Self::Query as rosetta_core::traits::Query>::Result> {
        anyhow::bail!("unsupported query");
    }

    fn config(&self) -> &BlockchainConfig {
        dispatch!(self.config())
    }

    fn genesis_block(&self) -> Self::BlockIdentifier {
        // dispatch!(self.genesis_block())
        match self {
            Self::Ethereum(client) => client.genesis_block(),
            Self::Astar(client) => client.genesis_block(),
            Self::Polkadot(client) => client.genesis_block(),
        }
    }

    async fn current_block(&self) -> Result<Self::BlockIdentifier> {
        // dispatch!(self.current_block().await)
        match self {
            Self::Ethereum(client) => client.current_block().await,
            Self::Astar(client) => client.current_block().await,
            Self::Polkadot(client) => client.current_block().await,
        }
    }

    async fn finalized_block(&self) -> Result<Self::BlockIdentifier> {
        // dispatch!(self.finalized_block().await)
        match self {
            Self::Ethereum(client) => client.finalized_block().await,
            Self::Astar(client) => client.finalized_block().await,
            Self::Polkadot(client) => client.finalized_block().await,
        }
    }

    async fn balance(&self, address: &Address, block: &Self::AtBlock) -> Result<u128> {
        match self {
            Self::Ethereum(client) => client.balance(address, block).await,
            Self::Astar(client) => client.balance(address, block).await,
            Self::Polkadot(client) => client.balance(address, block).await,
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

    async fn submit(&self, transaction: &[u8]) -> Result<SubmitResult> {
        match self {
            Self::Ethereum(client) => client.submit(transaction).await,
            Self::Astar(client) => client.submit(transaction).await,
            Self::Polkadot(client) => {
                // TODO: implement a custom receipt for Polkadot
                let result = client.submit(transaction).await?;
                let tx_hash = H256::from_slice(result.as_slice());
                Ok(SubmitResult::Executed {
                    tx_hash,
                    result: CallResult::Success(Vec::new()),
                    // TODO: Refactor this to use a custom receipt for Polkadot
                    // Did this to avoid wrapping the result into another enum, currently we only
                    // care about ethereum chains.
                    receipt: TransactionReceipt::default(),
                })
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
        match self {
            Self::Ethereum(client) => {
                let Some(stream) = client.listen().await? else {
                    return Ok(None);
                };
                Ok(Some(GenericClientStream::Ethereum(stream)))
            },
            Self::Astar(client) => {
                let Some(stream) = client.listen().await? else {
                    return Ok(None);
                };
                Ok(Some(GenericClientStream::Astar(stream)))
            },
            Self::Polkadot(client) => {
                let Some(stream) = client.listen().await? else {
                    return Ok(None);
                };
                Ok(Some(GenericClientStream::Polkadot(stream)))
            },
        }
    }

    async fn subscribe(&self, sub: &Self::Subscription) -> Result<u32> {
        match self {
            Self::Ethereum(client) => match sub {
                GenericClientSubscription::Ethereum(sub) => client.subscribe(sub).await,
                _ => anyhow::bail!("invalid subscription"),
            },
            Self::Astar(client) => match sub {
                GenericClientSubscription::Astar(sub) => client.subscribe(sub).await,
                _ => anyhow::bail!("invalid subscription"),
            },
            Self::Polkadot(client) => match sub {
                GenericClientSubscription::Polkadot(sub) => client.subscribe(sub).await,
                _ => anyhow::bail!("invalid subscription"),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenericClientSubscription {
    Ethereum(<EthereumClient as BlockchainClient>::Subscription),
    Astar(<AstarClient as BlockchainClient>::Subscription),
    Polkadot(<PolkadotClient as BlockchainClient>::Subscription),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenericClientEvent {
    Ethereum(<EthereumClient as BlockchainClient>::Event),
    Astar(<AstarClient as BlockchainClient>::Event),
    Polkadot(<PolkadotClient as BlockchainClient>::Event),
}

pub enum GenericClientStream<'a> {
    Ethereum(<EthereumClient as BlockchainClient>::EventStream<'a>),
    Astar(<AstarClient as BlockchainClient>::EventStream<'a>),
    Polkadot(<PolkadotClient as BlockchainClient>::EventStream<'a>),
}

impl<'a> Stream for GenericClientStream<'a> {
    type Item = ClientEvent<BlockIdentifier, GenericClientEvent>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = &mut *self;
        match this {
            Self::Ethereum(stream) => stream
                .poll_next_unpin(cx)
                .map(|opt| opt.map(|event| event.map_event(GenericClientEvent::Ethereum))),
            Self::Astar(stream) => stream
                .poll_next_unpin(cx)
                .map(|opt| opt.map(|event| event.map_event(GenericClientEvent::Astar))),
            Self::Polkadot(stream) => stream
                .poll_next_unpin(cx)
                .map(|opt| opt.map(|event| event.map_event(GenericClientEvent::Polkadot))),
        }
    }
}
