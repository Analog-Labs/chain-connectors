use crate::indexer::Indexer;
use anyhow::Result;
use clap::Parser;
use rosetta_core::crypto::address::Address;
use rosetta_core::crypto::PublicKey;
use rosetta_core::types::{
    AccountBalanceRequest, AccountBalanceResponse, AccountCoinsRequest, AccountCoinsResponse,
    AccountFaucetRequest, Amount, BlockRequest, BlockResponse, BlockTransactionRequest,
    BlockTransactionResponse, CallRequest, CallResponse, ConstructionMetadataRequest,
    ConstructionMetadataResponse, ConstructionSubmitRequest, MetadataRequest, NetworkIdentifier,
    NetworkListResponse, NetworkOptionsResponse, NetworkRequest, NetworkStatusResponse,
    SearchTransactionsRequest, TransactionIdentifier, TransactionIdentifierResponse, Version,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tide::http::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};
use tide::{Body, Request, Response};

pub use rosetta_core::*;

mod indexer;

#[derive(Parser)]
struct Opts {
    #[clap(long)]
    network: String,
    #[clap(long)]
    addr: SocketAddr,
    #[clap(long)]
    node_addr: String,
    #[clap(long)]
    path: PathBuf,
}

pub async fn main<T: BlockchainClient>() -> Result<()> {
    femme::start();
    let opts = Opts::parse();

    log::info!("connecting to {}", &opts.node_addr);
    let config = T::create_config(&opts.network)?;
    let client = T::new(config, &opts.node_addr).await?;
    let indexer = Arc::new(Indexer::new(&opts.path, client)?);

    let cors = CorsMiddleware::new()
        .allow_methods("POST".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);
    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.with(cors);
    app.at("/").nest(server(indexer.clone()));

    tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            if let Err(err) = indexer.sync().await {
                log::error!("{}", err);
            }
        }
    });

    log::info!("listening on {}", &opts.addr);
    app.listen(opts.addr).await?;

    Ok(())
}

type State<T> = Arc<Indexer<T>>;

fn server<T: BlockchainClient>(client: State<T>) -> tide::Server<State<T>> {
    let config = client.config();
    let utxo = config.utxo;
    let testnet = config.testnet;
    let mut app = tide::with_state(client);
    app.at("/account/balance").post(account_balance);
    if utxo {
        app.at("/account/coins").post(account_coins);
    }
    if testnet {
        app.at("/account/faucet").post(account_faucet);
    }
    app.at("/block").post(block);
    app.at("/block/transaction").post(block_transaction);
    app.at("/call").post(call);
    app.at("/construction/metadata").post(construction_metadata);
    app.at("/construction/submit").post(construction_submit);
    app.at("/network/list").post(network_list);
    app.at("/network/options").post(network_options);
    app.at("/network/status").post(network_status);
    app.at("/search/transactions").post(search_transactions);
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

async fn network_list<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
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

async fn network_options<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
    let request: NetworkRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let node_version = match req.state().node_version().await {
        Ok(node_version) => node_version,
        Err(err) => return Error::RpcError(err).to_result(),
    };
    let response = NetworkOptionsResponse {
        version: Version {
            rosetta_version: "1.4.13".into(),
            node_version,
            middleware_version: Some(env!("VERGEN_GIT_DESCRIBE").into()),
            metadata: None,
        },
        allow: None,
    };
    ok(&response)
}

async fn network_status<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
    let request: NetworkRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let current_block_identifier = match req.state().current_block().await {
        Ok(current_block_identifier) => current_block_identifier,
        Err(err) => return Error::RpcError(err).to_result(),
    };
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

async fn account_balance<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
    let request: AccountBalanceRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let block_identifier = match req.state().current_block().await {
        Ok(block_identifier) => block_identifier,
        Err(err) => return Error::RpcError(err).to_result(),
    };
    let address = Address::new(config.address_format, request.account_identifier.address);
    let value = match req.state().balance(&address, &block_identifier).await {
        Ok(value) => value,
        Err(err) => return Error::RpcError(err).to_result(),
    };
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

async fn account_coins<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
    let request: AccountCoinsRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let block_identifier = match req.state().current_block().await {
        Ok(block_identifier) => block_identifier,
        Err(err) => return Error::RpcError(err).to_result(),
    };
    let address = Address::new(config.address_format, request.account_identifier.address);
    let coins = match req.state().coins(&address, &block_identifier).await {
        Ok(coins) => coins,
        Err(err) => return Error::RpcError(err).to_result(),
    };
    let response = AccountCoinsResponse {
        coins,
        block_identifier,
        metadata: None,
    };
    ok(&response)
}

async fn account_faucet<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
    let request: AccountFaucetRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let address = Address::new(config.address_format, request.account_identifier.address);
    let hash = match req.state().faucet(&address, request.faucet_parameter).await {
        Ok(hash) => hash,
        Err(err) => return Error::RpcError(err).to_result(),
    };
    let response = TransactionIdentifierResponse {
        transaction_identifier: TransactionIdentifier {
            hash: hex::encode(hash),
        },
        metadata: None,
    };
    ok(&response)
}

async fn construction_metadata<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
    let request: ConstructionMetadataRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let options: T::MetadataParams = if let Some(options) = request.options {
        serde_json::from_value(options)?
    } else {
        return Error::UnsupportedOption.to_result();
    };
    if request.public_keys.len() != 1 {
        return Error::MissingPublicKey.to_result();
    }
    let public_key = &request.public_keys[0];
    if public_key.curve_type != config.algorithm.to_curve_type() {
        return Error::UnsupportedCurveType.to_result();
    }
    let public_key_bytes = hex::decode(&public_key.hex_bytes)?;
    let public_key = PublicKey::from_bytes(config.algorithm, &public_key_bytes)?;
    let metadata = match req.state().metadata(&public_key, &options).await {
        Ok(metadata) => metadata,
        Err(err) => return Error::RpcError(err).to_result(),
    };
    let response = ConstructionMetadataResponse {
        metadata: serde_json::to_value(&metadata)?,
        suggested_fee: None,
    };
    ok(&response)
}

async fn construction_submit<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
    let request: ConstructionSubmitRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let transaction = hex::decode(&request.signed_transaction)?;
    let hash = match req.state().submit(&transaction).await {
        Ok(hash) => hash,
        Err(err) => return Error::RpcError(err).to_result(),
    };
    let response = TransactionIdentifierResponse {
        transaction_identifier: TransactionIdentifier {
            hash: hex::encode(hash),
        },
        metadata: None,
    };
    ok(&response)
}

async fn block<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
    let request: BlockRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let block = match req.state().block(&request.block_identifier).await {
        Ok(block) => block,
        Err(err) => return Error::RpcError(err).to_result(),
    };
    let response = BlockResponse {
        block: Some(block),
        other_transactions: None,
    };
    ok(&response)
}

async fn block_transaction<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
    let request: BlockTransactionRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let transaction = match req
        .state()
        .block_transaction(&request.block_identifier, &request.transaction_identifier)
        .await
    {
        Ok(transaction) => transaction,
        Err(err) => return Error::RpcError(err).to_result(),
    };
    let response = BlockTransactionResponse { transaction };
    ok(&response)
}

async fn search_transactions<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
    let request: SearchTransactionsRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let response = match req.state().search(&request).await {
        Ok(response) => response,
        Err(err) => return Error::RpcError(err).to_result(),
    };
    ok(&response)
}

async fn call<T: BlockchainClient>(mut req: Request<State<T>>) -> tide::Result {
    let request: CallRequest = req.body_json().await?;
    let config = req.state().config();
    if !is_network_supported(&request.network_identifier, config) {
        return Error::UnsupportedNetwork.to_result();
    }
    let call_result = match req.state().call(&request).await {
        Ok(call_result) => call_result,
        Err(err) => return Error::RpcError(err).to_result(),
    };
    let response = CallResponse {
        result: call_result,
        idempotent: false,
    };
    ok(&response)
}

async fn unsupported<T>(_: Request<T>) -> tide::Result {
    Error::Unsupported.to_result()
}

#[derive(Debug)]
pub enum Error {
    Unimplemented,
    Unsupported,
    UnsupportedNetwork,
    UnsupportedOption,
    MissingPublicKey,
    UnsupportedCurveType,
    MoreThanOneSignature,
    InvalidSignatureType,
    RpcError(anyhow::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            Self::Unimplemented => "unimplemented",
            Self::Unsupported => "unsupported",
            Self::UnsupportedNetwork => "unsupported network",
            Self::UnsupportedOption => "unsupported option",
            Self::MissingPublicKey => "missing public key",
            Self::UnsupportedCurveType => "unsupported curve type",
            Self::MoreThanOneSignature => "expected one signature",
            Self::InvalidSignatureType => "invalid signature type",
            Self::RpcError(error) => return write!(f, "rpc error: {error}",),
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
    use futures::stream::StreamExt;
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
        let version = client.node_version().await?;

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

    pub async fn account(config: BlockchainConfig) -> Result<()> {
        let env = Env::new("account", config.clone()).await?;

        let value = 100 * u128::pow(10, config.currency_decimals);
        let wallet = env.ephemeral_wallet()?;
        wallet.faucet(value).await?;
        let amount = wallet.balance().await?;
        assert_eq!(amount.value, value.to_string());
        assert_eq!(amount.currency, config.currency());
        assert!(amount.metadata.is_none());

        env.shutdown().await?;
        Ok(())
    }

    pub async fn construction(config: BlockchainConfig) -> Result<()> {
        let env = Env::new("construction", config.clone()).await?;

        let faucet = 100 * u128::pow(10, config.currency_decimals);
        let value = u128::pow(10, config.currency_decimals);
        let alice = env.ephemeral_wallet()?;
        alice.faucet(faucet).await?;

        let bob = env.ephemeral_wallet()?;
        alice.transfer(bob.account(), value).await?;
        let amount = bob.balance().await?;
        assert_eq!(amount.value, value.to_string());

        env.shutdown().await?;
        Ok(())
    }

    pub async fn find_transaction(config: BlockchainConfig) -> Result<()> {
        let env = Env::new("find-transaction", config.clone()).await?;

        let faucet = 100 * u128::pow(10, config.currency_decimals);
        let value = u128::pow(10, config.currency_decimals);
        let alice = env.ephemeral_wallet()?;
        alice.faucet(faucet).await?;

        let bob = env.ephemeral_wallet()?;
        let tx_id = alice.transfer(bob.account(), value).await?;

        let tx = alice.transaction(tx_id.clone()).await?;
        assert_eq!(tx.transaction.transaction_identifier, tx_id);

        env.shutdown().await?;
        Ok(())
    }

    pub async fn list_transactions(config: BlockchainConfig) -> Result<()> {
        let env = Env::new("list-transactions", config.clone()).await?;

        let faucet = 100 * u128::pow(10, config.currency_decimals);
        let value = u128::pow(10, config.currency_decimals);
        let alice = env.ephemeral_wallet()?;
        alice.faucet(faucet).await?;

        let bob = env.ephemeral_wallet()?;
        alice.transfer(bob.account(), value).await?;
        alice.transfer(bob.account(), value).await?;
        alice.transfer(bob.account(), value).await?;

        tokio::time::sleep(Duration::from_secs(1)).await;

        let mut stream = bob.transactions(1);
        let mut count = 0;
        while let Some(res) = stream.next().await {
            let transactions = res?;
            assert_eq!(transactions.len(), 1);
            assert_eq!(stream.total_count(), Some(3));
            count += 1;
            assert!(count <= 3);
        }
        assert_eq!(count, 3);

        let mut stream = bob.transactions(10);
        let mut count = 0;
        while let Some(res) = stream.next().await {
            let transactions = res?;
            assert_eq!(transactions.len(), 3);
            assert_eq!(stream.total_count(), Some(3));
            count += 1;
            assert!(count <= 1);
        }
        assert_eq!(count, 1);

        env.shutdown().await?;
        Ok(())
    }
}
