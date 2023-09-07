use anyhow::Result;
use rosetta_config_astar::config as astar_config;
use rosetta_core::crypto::address::AddressFormat;
use rosetta_core::crypto::Algorithm;
use rosetta_core::{BlockchainConfig, NodeUri};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn config(network: &str) -> Result<BlockchainConfig> {
    let config = match network {
        "dev" | "mainnet" => BlockchainConfig {
            blockchain: "ethereum",
            network: if network == "dev" { "dev" } else { "mainnet" },
            algorithm: Algorithm::EcdsaRecoverableSecp256k1,
            address_format: AddressFormat::Eip55,
            coin: if network == "mainnet" { 60 } else { 1 },
            bip44: true,
            utxo: false,
            currency_unit: "wei",
            currency_symbol: "ETH",
            currency_decimals: 18,
            node_uri: NodeUri::parse("ws://127.0.0.1:8545/ws")?,
            node_image: "ethereum/client-go:v1.12.2",
            node_command: Arc::new(|network, port| {
                let mut params = if network == "dev" {
                    vec![
                        "--dev".into(),
                        "--dev.period=1".into(),
                        "--ipcdisable".into(),
                    ]
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
            testnet: network == "dev",
        },
        // Try to load the network config from astar
        "astar-local" => astar_config("dev")?,
        network => astar_config(network)?,
    };
    Ok(config)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthereumMetadataParams {
    pub destination: Vec<u8>,
    pub amount: [u64; 4],
    pub data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EthereumMetadata {
    pub chain_id: u64,
    pub nonce: u64,
    pub max_priority_fee_per_gas: [u64; 4],
    pub max_fee_per_gas: [u64; 4],
    pub gas_limit: [u64; 4],
}

// pub trait TransactionT<T: BlockchainConfigT> {
//     fn hash(&self) -> Multihash<T::HASH_LEN>;
//     fn fee(&self) -> u128;
//     fn is_coinbase(&self) -> bool;
// }
//
// pub trait BlockT<T: BlockchainConfigT> {
//     fn hash(&self) -> Multihash<T::HASH_LEN>;
// }
//
// pub trait AddressT<T: BlockchainConfigT> {
//     fn hash(&self) -> Multihash<T::HASH_LEN>;
// }
//
// pub trait BlockchainConfigT {
//     const HASH_LEN: usize;
//
//     type Address: AddressT<Self>;
//     type Transaction: TransactionT<Self>;
//     type Block: BlockT<Self>;
// }
//
// pub enum TransactionStatus {
//     /// The transaction is part of the “future” queue.
//     Future,
//     /// The transaction is part of the “ready” queue.
//     Ready,
//     /// The transaction has been broadcast to the given peers.
//     Broadcast(Vec<String>),
//     /// The transaction has been included in a block with given hash.
//     InBlock(TxInBlock<T, C>),
//     /// The block this transaction was included in has been retracted, probably because it did not make it onto the blocks which were finalized.
//     Retracted(T::Hash),
//     /// A block containing the transaction did not reach finality within 512 blocks, and so the subscription has ended.
//     FinalityTimeout(T::Hash),
//     /// The transaction has been finalized by a finality-gadget, e.g GRANDPA.
//     Finalized(TxInBlock<T, C>),
//     /// The transaction has been replaced in the pool by another transaction that provides the same tags. (e.g. same (sender, nonce)).
//     Usurped(T::Hash),
//     /// The transaction has been dropped from the pool because of the limit.
//     Dropped,
//     /// The transaction is no longer valid in the current state.
//     Invalid,
// }
//
// pub trait TransactionStream: futures_util::stream::Stream<Item = ()> {}
//
// pub enum ClientError<CustomError> {
//     /// The request was invalid.
//     InvalidRequest(String),
//
//     /// The request was valid but the server is currently unable to handle it.
//     ServerError(String),
//
//     /// The request was valid but the server is rate-limiting the client.
//     RateLimit(String),
//
//     /// The request was valid but there is not enough balance in the account.
//     InsufficientBalance(String),
//
//     /// The request was valid but the account is not ready to send funds.
//     NotReady(String),
//
//     /// The request was valid but the account is not ready to send funds.
//     Stale(String),
//
//     /// The request was valid but the account is not ready to send funds.
//     TemporaryFailure(String),
//
//     /// The request was valid but the account is not ready to send funds.
//     Unknown(String),
//
//     /// The request was valid but the account is not ready to send funds.
//     Custom(CustomError),
// }
