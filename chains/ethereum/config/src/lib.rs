#![cfg_attr(not(feature = "std"), no_std)]

mod types;
mod util;

use rosetta_config_astar::config as astar_config;
use rosetta_core::{
    crypto::{address::AddressFormat, Algorithm},
    BlockchainConfig, NodeUri,
};
#[allow(deprecated)]
pub use types::{
    AtBlock, BlockFull, BlockRef, Bloom, CallContract, CallResult, EIP1186ProofResponse,
    EthereumMetadata, EthereumMetadataParams, GetBalance, GetProof, GetStorageAt,
    GetTransactionReceipt, Header, Query, QueryResult, SignedTransaction, StorageProof,
    TransactionReceipt, H256,
};

#[cfg(not(feature = "std"))]
#[cfg_attr(test, macro_use)]
extern crate alloc;

#[cfg(feature = "std")]
pub(crate) mod rstd {
    pub use std::{convert, fmt, mem, option, result, slice, str, sync, vec};
}

#[cfg(not(feature = "std"))]
pub(crate) mod rstd {
    pub use alloc::{sync, vec};
    pub use core::{convert, fmt, mem, option, result, slice, str};
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

impl rosetta_core::traits::Header for Header {
    type Hash = BlockHash;

    fn number(&self) -> rosetta_core::traits::BlockNumber {
        self.0.number
    }

    fn hash(&self) -> Self::Hash {
        // TODO: compute header hash
        BlockHash(H256::zero())
    }
}

const _: () = {
    use rstd::mem::{align_of, size_of};
    type BlockTx = <BlockFull as rosetta_core::traits::Block>::Transaction;
    type RegularTx = types::SignedTransactionInner;
    assert!(
        !(size_of::<BlockTx>() != size_of::<RegularTx>()),
        "BlockFull and BlockFullInner must have the same memory size"
    );
    assert!(
        !(align_of::<BlockTx>() != align_of::<RegularTx>()),
        "BlockFull and BlockFullInner must have the same memory alignment"
    );
};

impl rosetta_core::traits::Block for BlockFull {
    type Transaction = SignedTransaction;
    type Header = Header;
    type Hash = BlockHash;

    fn header(&self) -> &Self::Header {
        (&self.0.header).into()
    }

    fn transactions(&self) -> &[Self::Transaction] {
        // Safety: `Self::Transaction` and  block transactions have the same memory layout
        unsafe {
            rstd::slice::from_raw_parts(
                self.0.transactions.as_ptr().cast(),
                self.0.transactions.len(),
            )
        }
    }

    fn hash(&self) -> Self::Hash {
        BlockHash(self.0.hash)
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
    Ok(evm_config("polygon", network, "MATIC", bip44_id, is_dev))
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
            NodeUri::parse("ws://127.0.0.1:8545/ws").expect("uri is valid; qed")
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
                "--http.api=eth,debug,admin,txpool,web3".into(),
                "--ws".into(),
                "--ws.addr=0.0.0.0".into(),
                format!("--ws.port={port}"),
                "--ws.origins=*".into(),
                "--ws.api=eth,debug,admin,txpool,web3".into(),
                "--ws.rpcprefix=/ws".into(),
            ]);
            params
        }),
        node_additional_ports: &[],
        connector_port: 8081,
        testnet: is_dev,
    }
}
