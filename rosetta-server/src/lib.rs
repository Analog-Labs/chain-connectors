use anyhow::Result;
use clap::Parser;
use rosetta_core::crypto::address::Address;
use rosetta_core::types::{
    AccountBalanceRequest, AccountBalanceResponse, AccountCoinsRequest, AccountCoinsResponse,
    AccountFaucetRequest, Amount, MetadataRequest, NetworkIdentifier, NetworkListResponse,
    NetworkOptionsResponse, NetworkRequest, NetworkStatusResponse, TransactionIdentifier,
    TransactionIdentifierResponse, Version,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tide::http::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};
use tide::{Body, Request, Response};

pub use rosetta_core::*;

#[derive(Parser)]
struct Opts {
    #[clap(long)]
    network: String,
    #[clap(long)]
    addr: SocketAddr,
    #[clap(long)]
    node_addr: String,
}

pub async fn main<T: BlockchainClient>() -> Result<()> {
    femme::start();
    let opts = Opts::parse();

    log::info!("connecting to {}", &opts.node_addr);
    let client = T::new(&opts.network, &opts.node_addr).await?;

    let cors = CorsMiddleware::new()
        .allow_methods("POST".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.with(cors);
    app.at("/").nest(server(client));

    log::info!("listening on {}", &opts.addr);
    app.listen(opts.addr).await?;

    Ok(())
}

fn server<T: BlockchainClient>(client: T) -> tide::Server<Arc<T>> {
    let mut app = tide::with_state(Arc::new(client));
    app.at("/account/balance").post(account_balance);
    app.at("/account/coins").post(account_coins);
    app.at("/account/faucet").post(account_faucet);
    app.at("/network/list").post(network_list);
    app.at("/network/options").post(network_options);
    app.at("/network/status").post(network_status);
    // unimplemented
    app.at("/block").post(unimplemented);
    app.at("/block/transaction").post(unimplemented);
    app.at("/construction/metadata").post(unimplemented);
    app.at("/construction/submit").post(unimplemented);
    app.at("/search/transactions").post(unimplemented);
    // unsupported
    app.at("/mempool").post(unsupported);
    app.at("/mempool/transaction").post(unsupported);
    app.at("/construction/combine").post(unsupported);
    app.at("/construction/derive").post(unsupported);
    app.at("/construction/hash").post(unsupported);
    app.at("/construction/parse").post(unsupported);
    app.at("/construction/payloads").post(unsupported);
    app.at("/construction/preprocess").post(unsupported);
    app.at("/events/blocks").post(unsupported);
    app
}

fn ok<T: serde::Serialize>(t: &T) -> tide::Result {
    let r = Response::builder(200)
        .body(Body::from_json(t).unwrap())
        .build();
    Ok(r)
}

fn is_network_supported(network_identifier: &NetworkIdentifier, config: &BlockchainConfig) -> bool {
    network_identifier.blockchain == config.blockchain
        && network_identifier.network == config.network
        && network_identifier.sub_network_identifier.is_none()
}

async fn network_list<T: BlockchainClient>(mut req: Request<Arc<T>>) -> tide::Result {
    let _request: MetadataRequest = req.body_json().await?;
    let config = req.state().config();
    let response = NetworkListResponse {
        network_identifiers: vec![NetworkIdentifier {
            blockchain: config.blockchain.into(),
            network: config.network.into(),
            sub_network_identifier: None,
        }],
    };
    ok(&response)
}

async fn network_options<T: BlockchainClient>(mut req: Request<Arc<T>>) -> tide::Result {
    let request: NetworkRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let node_version = req.state().node_version().await?;
    let response = NetworkOptionsResponse {
        version: Version {
            rosetta_version: "1.4.13".into(),
            node_version,
            middleware_version: None,
            metadata: None,
        },
        allow: None,
    };
    ok(&response)
}

async fn network_status<T: BlockchainClient>(mut req: Request<Arc<T>>) -> tide::Result {
    let request: NetworkRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let current_block_identifier = req.state().current_block().await?;
    let response = NetworkStatusResponse {
        current_block_identifier,
        current_block_timestamp: 0,
        genesis_block_identifier: Some(req.state().genesis_block().clone()),
        peers: None,
        oldest_block_identifier: None,
        sync_status: None,
    };
    ok(&response)
}

async fn account_balance<T: BlockchainClient>(mut req: Request<Arc<T>>) -> tide::Result {
    let request: AccountBalanceRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let block_identifier = req.state().current_block().await?;
    let address = Address::new(config.address_format, request.account_identifier.address);
    let value = req.state().balance(&address, &block_identifier).await?;
    let response = AccountBalanceResponse {
        balances: vec![Amount {
            value: value.to_string(),
            currency: config.currency(),
            metadata: None,
        }],
        block_identifier,
        metadata: None,
    };
    ok(&response)
}

async fn account_coins<T: BlockchainClient>(mut req: Request<Arc<T>>) -> tide::Result {
    let request: AccountCoinsRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let block_identifier = req.state().current_block().await?;
    let address = Address::new(config.address_format, request.account_identifier.address);
    let coins = req.state().coins(&address, &block_identifier).await?;
    let response = AccountCoinsResponse {
        coins,
        block_identifier,
        metadata: None,
    };
    ok(&response)
}

async fn account_faucet<T: BlockchainClient>(mut req: Request<Arc<T>>) -> tide::Result {
    let request: AccountFaucetRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let address = Address::new(config.address_format, request.account_identifier.address);
    let hash = req
        .state()
        .faucet(&address, request.faucet_parameter)
        .await?;
    let response = TransactionIdentifierResponse {
        transaction_identifier: TransactionIdentifier {
            hash: hex::encode(&hash),
        },
        metadata: None,
    };
    ok(&response)
}

async fn unimplemented<T>(_: Request<T>) -> tide::Result {
    Error::Unimplemented.to_result()
}

async fn unsupported<T>(_: Request<T>) -> tide::Result {
    Error::Unsupported.to_result()
}

#[derive(Debug)]
pub enum Error {
    Unimplemented,
    Unsupported,
    UnsupportedNetwork,
    RpcError(anyhow::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            Self::Unimplemented => "unimplemented",
            Self::Unsupported => "unsupported",
            Self::UnsupportedNetwork => "unsupported network",
            Self::RpcError(_) => "rpc error",
        };
        f.write_str(msg)
    }
}

impl Error {
    pub fn error(&self) -> Option<&anyhow::Error> {
        let error = match self {
            Self::RpcError(error) => error,
            _ => return None,
        };
        Some(error)
    }

    pub fn description(&self) -> Option<String> {
        self.error().map(|error| error.to_string())
    }

    pub fn to_response(&self) -> Response {
        let error = rosetta_core::types::Error {
            code: 500,
            message: self.to_string(),
            description: self.description(),
            retriable: false,
            details: None,
        };
        Response::builder(500)
            .body(Body::from_json(&error).unwrap())
            .build()
    }

    pub fn to_result(&self) -> tide::Result {
        Ok(self.to_response())
    }
}

#[cfg(feature = "tests")]
pub mod tests {
    use super::*;
    use rosetta_docker::Env;

    pub async fn network_list(config: BlockchainConfig) -> Result<()> {
        let env = Env::new("network-list", config.clone()).await?;

        let client = env.connector()?;
        let networks = client.network_list().await?;
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].blockchain, config.blockchain);
        assert_eq!(networks[0].network, config.network);
        assert!(networks[0].sub_network_identifier.is_none());

        env.shutdown().await?;
        Ok(())
    }

    pub async fn network_options<T: BlockchainClient>(config: BlockchainConfig) -> Result<()> {
        let env = Env::new("network-options", config.clone()).await?;

        let client = env.node::<T>().await?;
        let version = client.node_version();

        let client = env.connector()?;
        let options = client.network_options(config.network()).await?;
        assert_eq!(options.version.node_version, version);

        env.shutdown().await?;
        Ok(())
    }

    pub async fn network_status<T: BlockchainClient>(config: BlockchainConfig) -> Result<()> {
        let env = Env::new("network-status", config.clone()).await?;

        let client = env.node::<T>().await?;
        let genesis = client.genesis_block().clone();
        let current = client.current_block().await?;

        let client = env.connector()?;
        let status = client.network_status(config.network()).await?;
        assert_eq!(status.genesis_block_identifier, Some(genesis));
        assert_eq!(status.current_block_identifier, current);

        env.shutdown().await?;
        Ok(())
    }
}
