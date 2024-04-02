#![cfg_attr(not(feature = "std"), no_std)]

mod types;
mod util;

use rosetta_config_astar::config as astar_config;
use rosetta_core::{
    crypto::{address::AddressFormat, Algorithm},
    BlockchainConfig, NodeUri,
};
use rosetta_ethereum_types::TxHash;
pub use types::{
    Address, AtBlock, BlockFull, Bloom, CallContract, CallResult, EIP1186ProofResponse,
    EthereumMetadata, EthereumMetadataParams, GetBalance, GetProof, GetStorageAt,
    GetTransactionReceipt, Header, Log, PartialBlock, Query, QueryItem, QueryResult, SealedHeader,
    SignedTransaction, StorageProof, TransactionReceipt, H256,
};

pub mod query {
    pub use crate::types::{
        CallContract, GetBalance, GetBlockByHash, GetLogs, GetProof, GetStorageAt,
        GetTransactionReceipt, Query, QueryItem, QueryResult,
    };
}

#[cfg(not(feature = "std"))]
#[cfg_attr(test, macro_use)]
extern crate alloc;

#[cfg(feature = "std")]
pub(crate) mod rstd {
    pub use std::{convert, fmt, ops, option, result, slice, str, sync, vec};
}

#[cfg(not(feature = "std"))]
pub(crate) mod rstd {
    pub use alloc::{sync, vec};
    pub use core::{convert, fmt, ops, option, result, slice, str};
}

/// Re-export external crates that are made use of in the client API.
pub mod ext {
    pub use rosetta_ethereum_types as types;

    #[cfg(feature = "scale-info")]
    pub use scale_info;

    #[cfg(feature = "scale-codec")]
    pub use parity_scale_codec;

    #[cfg(feature = "serde")]
    pub use serde;
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SubmitResult {
    /// The transaction was submitted and included in the block
    Executed { tx_hash: TxHash, result: CallResult, receipt: TransactionReceipt },
    /// The transaction was submitted but not included in the block within the timeout
    Timeout { tx_hash: TxHash },
}

impl SubmitResult {
    #[must_use]
    pub const fn tx_hash(&self) -> TxHash {
        match self {
            Self::Executed { tx_hash, .. } | Self::Timeout { tx_hash } => *tx_hash,
        }
    }

    #[must_use]
    pub const fn receipt(&self) -> Option<&TransactionReceipt> {
        match self {
            Self::Executed { receipt, .. } => Some(receipt),
            Self::Timeout { .. } => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum Subscription {
    Logs { address: Address, topics: Vec<H256> },
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum Event {
    Logs(Vec<Log>),
}

impl rosetta_core::traits::Transaction for SignedTransaction {
    type Call = ();
    type SignaturePayload = ();
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct BlockHash(pub H256);

impl From<H256> for BlockHash {
    fn from(hash: H256) -> Self {
        Self(hash)
    }
}

impl From<BlockHash> for H256 {
    fn from(block_hash: BlockHash) -> Self {
        block_hash.0
    }
}

impl rstd::convert::AsMut<[u8]> for BlockHash {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_bytes_mut()
    }
}

impl rstd::convert::AsRef<[u8]> for BlockHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl rstd::str::FromStr for BlockHash {
    type Err = <H256 as rstd::str::FromStr>::Err;

    fn from_str(s: &str) -> rstd::result::Result<Self, Self::Err> {
        let hash = <H256 as rstd::str::FromStr>::from_str(s)?;
        Ok(Self(hash))
    }
}

impl rstd::fmt::Display for BlockHash {
    fn fmt(&self, f: &mut rstd::fmt::Formatter<'_>) -> rstd::fmt::Result {
        rstd::fmt::Display::fmt(&self.0, f)
    }
}

impl rosetta_core::traits::HashOutput for BlockHash {}

impl rosetta_core::traits::Header for SealedHeader {
    type Hash = BlockHash;

    fn number(&self) -> rosetta_core::traits::BlockNumber {
        self.0.header().number
    }

    fn hash(&self) -> Self::Hash {
        BlockHash(self.0.hash())
    }
}

// Make sure that `Transaction` has the same memory layout as `SignedTransactionInner`
static_assertions::assert_eq_size!(
    <BlockFull as rosetta_core::traits::Block>::Transaction,
    types::SignedTransactionInner
);
static_assertions::assert_eq_align!(
    <BlockFull as rosetta_core::traits::Block>::Transaction,
    types::SignedTransactionInner
);

impl rosetta_core::traits::Block for BlockFull {
    type Transaction = SignedTransaction;
    type Header = SealedHeader;
    type Hash = BlockHash;

    fn header(&self) -> &Self::Header {
        (self.0.header()).into()
    }

    fn transactions(&self) -> &[Self::Transaction] {
        // Safety: `Self::Transaction` and  block transactions have the same memory layout
        let transactions: &[types::SignedTransactionInner] = self.0.body().transactions.as_ref();
        unsafe { rstd::slice::from_raw_parts(transactions.as_ptr().cast(), transactions.len()) }
    }

    fn hash(&self) -> Self::Hash {
        BlockHash(self.0.header().hash())
    }
}

/// Retrieve the [`BlockchainConfig`] from the provided polygon `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn polygon_config(network: &str) -> anyhow::Result<BlockchainConfig> {
    let (network, bip44_id, is_dev) = match network {
        "dev" => ("dev", 1, true),
        "mumbai" => ("mumbai", 1, true),
        "mainnet" => ("mainnet", 966, false),
        _ => anyhow::bail!("unsupported network: {}", network),
    };
    Ok(poly_config("polygon", network, "MATIC", bip44_id, is_dev))
}

/// Retrieve the [`BlockchainConfig`] from the provided arbitrum `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn arbitrum_config(network: &str) -> anyhow::Result<BlockchainConfig> {
    // All available networks are listed here:
    let (network, bip44_id, is_dev) = match network {
        "dev" => ("dev", 1, true),
        "goerli" => ("goerli", 1, true),
        "mainnet" => ("mainnet", 42161, false),
        _ => anyhow::bail!("unsupported network: {}", network),
    };
    Ok(evm_config("arbitrum", network, "ARB", bip44_id, is_dev))
}

/// Retrieve the [`BlockchainConfig`] from the provided ethereum `network`
///
/// # Errors
/// Returns `Err` if the network is not supported
pub fn config(network: &str) -> anyhow::Result<BlockchainConfig> {
    let (network, symbol, bip44_id, is_dev) = match network {
        "dev" => ("dev", "ETH", 1, true),
        "mainnet" => ("mainnet", "ETH", 60, false),
        "goerli" => ("goerli", "TST", 1, true),
        "sepolia" => ("sepolia", "SepoliaETH", 1, true),

        // Polygon
        "polygon-local" => return polygon_config("dev"),
        "polygon" => return polygon_config("mainnet"),
        "mumbai" => return polygon_config("mumbai"),

        // Astar
        "astar-local" => return astar_config("dev"),

        // Arbitrum
        "arbitrum-local" => return arbitrum_config("dev"),
        "arbitrum" => return arbitrum_config("mainnet"),
        "arbitrum-goerli" => return arbitrum_config("goerli"),

        network => return astar_config(network),
    };

    Ok(evm_config("ethereum", network, symbol, bip44_id, is_dev))
}

fn evm_config(
    blockchain: &'static str,
    network: &'static str,
    symbol: &'static str,
    bip44_id: u32,
    is_dev: bool,
) -> BlockchainConfig {
    BlockchainConfig {
        blockchain,
        network,
        algorithm: Algorithm::EcdsaRecoverableSecp256k1,
        address_format: AddressFormat::Eip55,
        coin: bip44_id,
        bip44: true,
        utxo: false,
        currency_unit: "wei",
        currency_symbol: symbol,
        currency_decimals: 18,
        node_uri: {
            #[allow(clippy::expect_used)]
            NodeUri::parse("ws://127.0.0.1:8545").expect("uri is valid; qed")
        },
        node_image: "ethereum/client-go:v1.12.2",
        node_command: rstd::sync::Arc::new(|network, port| {
            let mut params = if network == "dev" {
                vec!["--dev".into(), "--dev.period=1".into(), "--ipcdisable".into()]
            } else {
                vec!["--syncmode=full".into()]
            };
            params.extend_from_slice(&[
                "--http".into(),
                "--http.addr=0.0.0.0".into(),
                format!("--http.port={port}"),
                "--http.vhosts=*".into(),
                "--http.corsdomain=*".into(),
                "--http.api=eth,debug,admin,txpool,web3,net".into(),
                "--ws".into(),
                "--ws.addr=0.0.0.0".into(),
                format!("--ws.port={port}"),
                "--ws.origins=*".into(),
                "--ws.api=eth,debug,admin,txpool,web3,net".into(),
                "--ws.rpcprefix=/".into(),
            ]);
            params
        }),
        node_additional_ports: &[],
        connector_port: 8081,
        testnet: is_dev,
    }
}



fn poly_config(
    blockchain: &'static str,
    network: &'static str,
    symbol: &'static str,
    bip44_id: u32,
    is_dev: bool,
) -> BlockchainConfig {
    BlockchainConfig {
        blockchain,
        network,
        algorithm: Algorithm::EcdsaRecoverableSecp256k1,
        address_format: AddressFormat::Eip55,
        coin: bip44_id,
        bip44: true,
        utxo: false,
        currency_unit: "matic",
        currency_symbol: symbol,
        currency_decimals: 18,
        node_uri: NodeUri::parse("ws://127.0.0.1:8546").expect("uri is valid; qed"),
        node_image: "local/bor",
        node_command: rstd::sync::Arc::new(|network, port| {
            let mut params = if network == "dev" {
                vec!["--dev".into(), "--dev.period=1".into(), "--ipcdisable".into()]
            } else {
                vec!["--syncmode=full".into()]
            };
            params.extend_from_slice(&[
                "--http".into(),
                "--http.addr=0.0.0.0".into(),
                format!("--http.port={port}"),
                "--http.vhosts=*".into(),
                "--http.corsdomain=*".into(),
                "--http.api=eth,debug,admin,txpool,web3,net".into(),
                "--ws".into(),
                "--ws.addr=0.0.0.0".into(),
                format!("--ws.port={port}"),
                "--ws.origins=*".into(),
                "--ws.api=eth,debug,admin,txpool,web3,net".into(),
                "--ws.rpcprefix=/".into(),
            ]);
            params
        }),
        node_additional_ports: &[],
        connector_port: 8084,
        testnet: is_dev,
    }
}
