//! Rosetta client.
#![deny(missing_docs)]
use anyhow::Result;

pub use crate::wallet::Wallet;
pub use rosetta_core::{crypto, types, BlockchainConfig};

/// Clients that communicates to different blockchains
pub mod client;
mod mnemonic;
mod signer;
mod tx_builder;
mod wallet;

pub use signer::Signer;

/// Re-exports libraries to not require any additional
/// dependencies to be explicitly added on the client side.
#[doc(hidden)]
pub mod ext {
    pub use anyhow;
    pub use fraction;
    pub use rosetta_core as core;
    pub use rosetta_server_astar as astar;
    pub use rosetta_server_ethereum as ethereum;
    pub use rosetta_server_polkadot as polkadot;
}

/// Supported chains.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blockchain {
    /// Ethereum
    Ethereum,
    /// Astar
    Astar,
    /// Polkadot
    Polkadot,
    /// Kusama
    Kusama,
    /// Rococo
    Rococo,
    /// Westend
    Westend,
    /// Wococo
    Wococo,
    /// Polygon
    Polygon,
    /// Arbitrum
    Arbitrum,
    /// Humanode
    Humanode,
}

impl std::str::FromStr for Blockchain {
    type Err = anyhow::Error;

    fn from_str(blockchain: &str) -> Result<Self> {
        Ok(match blockchain {
            "ethereum" => Self::Ethereum,
            "astar" => Self::Astar,
            "polkadot" => Self::Polkadot,
            "kusama" => Self::Kusama,
            "rococo" => Self::Rococo,
            "westend" => Self::Westend,
            "wococo" => Self::Wococo,
            "polygon" => Self::Polygon,
            "arbitrum" => Self::Arbitrum,
            "humanode" => Self::Humanode,
            _ => anyhow::bail!("unsupported blockchain {}", blockchain),
        })
    }
}
