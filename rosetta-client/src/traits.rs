#![allow(dead_code)]

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::Future;

pub trait Query {
    type Params: Serialize + Clone;
    type Response: DeserializeOwned + Sized + Send + Sync + 'static;
    type Error: DeserializeOwned + Sized + Send + Sync + 'static;

    fn query(&self, request: &Self::Params) -> Result<Self::Response, Self::Error>;
}

pub trait DefaultQuery {
    type Balance: Query;
}

/// Base Ledger Config
pub trait BaseConfig {
    type Address;
    type Transaction;

    /// The native currency of the blockchain
    type MainCurrency: FungibleAssetConfig;
}

pub enum AtBlock<T: BlockchainConfig> {
    /// The latest block with at least 1 confirmation
    Latest,
    /// The earliest block
    Earliest,
    /// The pending block, may not yet included in the blockchain
    Pending,
    /// The block with the given height
    Number(T::BlockNumber),
    /// The block with the given unique identifier
    At(T::BlockIdentifier),
}

/// A blockchain have the concept of blocks
pub trait BlockchainConfig: BaseConfig {
    type BlockIdentifier;
    type BlockNumber: num_traits::Unsigned + num_traits::Bounded;

    /// The genesis block identifier
    const GENESIS_BLOCK_IDENTIFIER: Self::BlockIdentifier;

    /// The forks of the blockchain, empty if there is no fork
    const FORKED_BLOCKS: [(Self::BlockNumber, Self::BlockIdentifier)];
}

/// The blockchain have a native currency
pub trait FungibleAssetConfig {
    const SYMBOL: &'static str;
    const DECIMALS: u8;
    type Balance: num_traits::Unsigned + num_traits::Bounded;
}

pub trait BlockchainClient<T: BlockchainConfig> {
    type Error;

    type InspectBalance: InspectBalance<T, Self::Error>;
}

/// Trait for providing balance-inspection access to a fungible asset.
pub trait InspectBalance<T: BlockchainConfig, ERR>: Sized {
    type Error: Into<ERR>;

    type Future: Future<Output = Result<<T::MainCurrency as FungibleAssetConfig>::Balance, Self::Error>>
        + Unpin;

    /// Returns the balance of the given account.
    fn balance_of(&self, account: T::Address, at: AtBlock<T>) -> Self::Future;
}
