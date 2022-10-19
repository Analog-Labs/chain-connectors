use crate::chains::substrate::api;
use anyhow::Result;
use rosetta_types::{
    AccountBalanceRequest, AccountBalanceResponse, AccountIdentifier, Amount, BlockIdentifier,
    ConstructionDeriveRequest, ConstructionDeriveResponse, Currency, CurveType, MetadataRequest,
    NetworkIdentifier, NetworkListResponse, PartialBlockIdentifier,
};
use ss58_registry::{Ss58AddressFormat, Ss58AddressFormatRegistry};
use subxt::ext::sp_core::{crypto::AccountId32, H256};
use subxt::{rpc::BlockNumber, OnlineClient, SubstrateConfig};
use tide::{Body, Request, Response};

mod chains;
mod ss58;

pub struct Config {
    pub url: &'static str,
    pub network: NetworkIdentifier,
    pub currency: Currency,
    pub ss58_address_format: Ss58AddressFormat,
}

impl Config {
    pub fn dev() -> Self {
        Self {
            url: "http://127.0.0.1:8082",
            network: NetworkIdentifier {
                blockchain: "Polkadot".into(),
                network: "Dev".into(),
                sub_network_identifier: None,
            },
            currency: Currency {
                decimals: 10,
                symbol: "DOT".into(),
                metadata: None,
            },
            ss58_address_format: Ss58AddressFormatRegistry::PolkadotAccount.into(),
        }
    }
}

#[derive(Clone)]
pub struct State {
    network: NetworkIdentifier,
    currency: Currency,
    ss58_address_format: Ss58AddressFormat,
    client: OnlineClient<SubstrateConfig>,
}

impl State {
    pub async fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            network: config.network.clone(),
            currency: config.currency.clone(),
            ss58_address_format: config.ss58_address_format,
            client: OnlineClient::new().await?,
        })
    }
}

pub async fn server(config: &Config) -> Result<tide::Server<State>> {
    let state = State::new(config).await?;
    let mut app = tide::with_state(state);
    app.at("/network/list").post(network_list);
    // app.at("/network/options").post(network_options);
    // app.at("/network/status").post(network_status);
    app.at("/account/balance").post(account_balance);
    // app.at("/account/coins").post(account_coins);
    // app.at("/block").post(block);
    // app.at("/block/transaction").post(block_transaction);
    // app.at("/construction/combine").post(construction_combine);
    app.at("/construction/derive").post(construction_derive);
    // app.at("/construction/hash").post(construction_hash);
    // app.at("/construction/metadata").post(construction_metadata);
    // app.at("/construction/parse").post(construction_parse);
    // app.at("/construction/payloads").post(construction_payloads);
    // app.at("/construction/preprocess").post(construction_preprocess);
    // app.at("/construction/submit").post(construction_submit);
    // app.at("/events/blocks").post(events_blocks);
    // app.at("/search/transactions").post(search_transactions);
    // app.at("/mempool").post(mempool);
    // app.at("/mempool/transaction").post(mempool_transaction);

    Ok(app)
}

//list of methods implementation for substrate chain
async fn network_list(mut req: Request<State>) -> tide::Result {
    let _request: MetadataRequest = req.body_json().await?;
    let response = NetworkListResponse {
        network_identifiers: vec![req.state().network.clone()],
    };
    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

async fn network_options(mut req: Request<State>) -> tide::Result{todo!()}

async fn network_status(mut req: Request<State>) -> tide::Result{todo!()}

async fn account_balance(mut req: Request<State>) -> tide::Result {
    let request: AccountBalanceRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }
    let (hash, index) =
        resolve_block(&req.state().client, request.block_identifier.as_ref()).await?;
    let address = &request.account_identifier.address;
    let account: Result<AccountId32, Error> = address.parse().map_err(|_| Error::InvalidAddress);
    let account = match account {
        Ok(account) => account,
        Err(error) => {
            return error.to_response();
        }
    };
    let account_key = api::storage().balances().account(&account);
    let account_data = req
        .state()
        .client
        .storage()
        .fetch_or_default(&account_key, Some(hash))
        .await?;
    let response = AccountBalanceResponse {
        balances: vec![Amount {
            value: account_data.free.to_string(),
            currency: req.state().currency.clone(),
            metadata: None,
        }],
        block_identifier: BlockIdentifier {
            index,
            hash: hash.to_string(),
        },
        metadata: None,
    };
    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

async fn account_coins(mut req: Request<State>) -> tide::Result{todo!()}

async fn block(mut req: Request<State>) -> tide::Result{todo!()}

async fn block_transaction(mut req: Request<State>) -> tide::Result{todo!()}

async fn construction_combine(mut req: Request<State>) -> tide::Result{todo!()}

async fn construction_derive(mut req: Request<State>) -> tide::Result {
    let request: ConstructionDeriveRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }
    if request.public_key.curve_type != CurveType::Schnorrkel {
        return Error::UnsupportedCurveType.to_response();
    }
    let public_key = match hex::decode(&request.public_key.hex_bytes) {
        Ok(public_key) => public_key,
        Err(_) => return Error::InvalidHex.to_response(),
    };
    let address = ss58::ss58_encode(req.state().ss58_address_format, &public_key);
    let response = ConstructionDeriveResponse {
        account_identifier: Some(AccountIdentifier {
            address: address.clone(),
            sub_account: None,
            metadata: None,
        }),
        address: Some(address),
        metadata: None,
    };
    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

async fn construction_hash(mut req: Request<State>) -> tide::Result{todo!()}

async fn construction_metadata(mut req: Request<State>) -> tide::Result{todo!()}

async fn construction_parse(mut req: Request<State>) -> tide::Result{todo!()}

async fn construction_payloads(mut req: Request<State>) -> tide::Result{todo!()}

async fn construction_preprocess(mut req: Request<State>) -> tide::Result{todo!()}

async fn construction_submit(mut req: Request<State>) -> tide::Result{todo!()}

async fn events_blocks(mut req: Request<State>) -> tide::Result{todo!()}

async fn search_transactions(mut req: Request<State>) -> tide::Result{todo!()}

async fn mempool(mut req: Request<State>) -> tide::Result{todo!()}

async fn mempool_transaction(mut req: Request<State>) -> tide::Result{todo!()}


//utils for methods
enum Error {
    UnsupportedNetwork,
    UnsupportedCurveType,
    InvalidHex,
    InvalidAddress,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UnsupportedNetwork => write!(f, "unsupported network"),
            Self::UnsupportedCurveType => write!(f, "unsupported curve type"),
            Self::InvalidHex => write!(f, "invalid hex"),
            Self::InvalidAddress => write!(f, "invalid address"),
        }
    }
}

impl Error {
    fn to_response(&self) -> tide::Result {
        let error = rosetta_types::Error {
            code: 500,
            message: format!("{}", self),
            description: None,
            retriable: false,
            details: None,
        };
        Ok(Response::builder(500)
            .body(Body::from_json(&error)?)
            .build())
    }
}

async fn resolve_block(
    subxt: &OnlineClient<SubstrateConfig>,
    partial: Option<&PartialBlockIdentifier>,
) -> Result<(H256, u64)> {
    let mindex = if let Some(PartialBlockIdentifier {
        index: Some(index), ..
    }) = partial
    {
        Some(*index)
    } else {
        None
    };
    let hash = if let Some(PartialBlockIdentifier {
        hash: Some(hash), ..
    }) = partial
    {
        hash.parse()?
    } else if let Some(hash) = subxt
        .rpc()
        .block_hash(mindex.map(BlockNumber::from))
        .await?
    {
        hash
    } else {
        anyhow::bail!("invalid hash");
    };
    let index = if let Some(header) = subxt.rpc().header(Some(hash)).await? {
        header.number as _
    } else {
        anyhow::bail!("invalid hash");
    };
    if let Some(mindex) = mindex {
        anyhow::ensure!(index == mindex);
    }
    Ok((hash, index))
}
