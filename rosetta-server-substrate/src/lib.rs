use std::str::FromStr;
use std::time::Duration;

use crate::chains::substrate::api;
use anyhow::Result;
use parity_scale_codec::Encode;
use rosetta_types::{
    AccountBalanceRequest, AccountBalanceResponse, AccountIdentifier, Amount, Block,
    BlockIdentifier, BlockRequest, BlockResponse, BlockTransactionRequest,
    ConstructionDeriveRequest, ConstructionDeriveResponse, ConstructionMetadataRequest,
    ConstructionPreprocessRequest, ConstructionPreprocessResponse, Currency, CurveType,
    MetadataRequest, NetworkIdentifier, NetworkListResponse, NetworkRequest, NetworkStatusResponse,
    PartialBlockIdentifier,
};

use ss58_registry::{Ss58AddressFormat, Ss58AddressFormatRegistry};
use subxt::ext::sp_core::{crypto::AccountId32, H256};
use subxt::ext::sp_runtime::generic::{Block as SPBlock, Header, SignedBlock};
use subxt::ext::sp_runtime::traits::BlakeTwo256;
use subxt::ext::sp_runtime::OpaqueExtrinsic;
use subxt::rpc_params;
use subxt::{rpc::BlockNumber, OnlineClient, SubstrateConfig};
use tide::prelude::json;
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
    app.at("/network/options").post(network_options);
    app.at("/network/status").post(network_status);
    app.at("/account/balance").post(account_balance);
    app.at("/account/coins").post(account_coins);
    app.at("/block").post(block);
    app.at("/block/transaction").post(block_transaction);
    app.at("/construction/combine").post(construction_combine);
    app.at("/construction/derive").post(construction_derive);
    app.at("/construction/hash").post(construction_hash);
    app.at("/construction/metadata").post(construction_metadata);
    app.at("/construction/parse").post(construction_parse);
    app.at("/construction/payloads").post(construction_payloads);
    app.at("/construction/preprocess")
        .post(construction_preprocess);
    app.at("/construction/submit").post(construction_submit);
    app.at("/events/blocks").post(events_blocks);
    app.at("/search/transactions").post(search_transactions);
    app.at("/mempool").post(mempool);
    app.at("/mempool/transaction").post(mempool_transaction);

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

async fn network_options(mut _req: Request<State>) -> tide::Result {
    todo!()
}

async fn network_status(mut req: Request<State>) -> tide::Result {
    let _request: NetworkRequest = req.body_json().await?;

    let current_block_timestamp = api::storage().timestamp().now();
    let genesis_block_hash = req.state().client.rpc().genesis_hash().await?;
    let current_block = req.state().client.rpc().block(None).await?.unwrap();

    let current_block_identifier = BlockIdentifier {
        index: current_block.block.header.number as u64,
        hash: current_block.block.header.hash().to_string(),
    };

    let genesis_block_identifier = BlockIdentifier {
        index: 0,
        hash: genesis_block_hash.to_string(),
    };

    let unix_timestamp_millis = req
        .state()
        .client
        .storage()
        .fetch(&current_block_timestamp, None)
        .await?
        .unwrap();

    let timestamp_nanos = Duration::from_millis(unix_timestamp_millis).as_nanos() as u64;

    let response = NetworkStatusResponse {
        current_block_identifier,
        current_block_timestamp: timestamp_nanos as i64,
        genesis_block_identifier,
        peers: None,
        oldest_block_identifier: None,
        sync_status: None,
    };

    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

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

async fn account_coins(mut _req: Request<State>) -> tide::Result {
    todo!()
}

//transactions are pending in this response
async fn block(mut req: Request<State>) -> tide::Result {
    let request: BlockRequest = match req.body_json().await {
        Ok(ok) => ok,
        Err(e) => {
            return Ok(Response::builder(400)
                .body(Body::from_json(&format!(
                    "error while parsing params {}",
                    e
                ))?)
                .build());
        }
    };
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }

    let block_hash: H256 = match request.block_identifier.hash {
        Some(hash) => match H256::from_str(&hash) {
            Ok(hash) => hash,
            Err(_) => return Error::InvalidBlockHash.to_response(),
        },
        None => return Error::InvalidBlockIdentifier.to_response(),
    };

    let index = match request.block_identifier.index {
        Some(index) => index,
        None => return Error::InvalidBlockIdentifier.to_response(),
    };

    let block = req.state().client.rpc().block(Some(block_hash)).await?;
    let block = match block {
        Some(block) => block,
        None => {
            return Error::BlockNotFound.to_response();
        }
    };

    let timestamp = api::storage().timestamp().now();
    let unix_timestamp_millis = req
        .state()
        .client
        .storage()
        .fetch(&timestamp, Some(block_hash))
        .await?
        .unwrap();

    let timestamp_nanos = Duration::from_millis(unix_timestamp_millis).as_nanos() as u64;

    let events_storage = api::storage().system().events();
    let _events = req
        .state()
        .client
        .storage()
        .fetch(&events_storage, Some(block_hash))
        .await?
        .unwrap();


    /////////////////////////
    // Getting transactions data
    let mut payment_infos = vec![];
    let extrinsincs = block.block.extrinsics.clone();
    for extrinsic in extrinsincs.iter() {
        //process extrinsics
        let extrincic_bytes = extrinsic.encode();
        let blockhash = block_hash;
        let params = rpc_params![extrincic_bytes, blockhash];
        payment_infos.push(
            req.state()
                .client
                .rpc()
                .request("payment_queryInfo", params)
                .await?,
        );
    }

    get_transactions(&req.state().client, &block).await;

    /////////////////////////

    let block = Block {
        block_identifier: BlockIdentifier {
            index,
            hash: block_hash.to_string(),
        },
        parent_block_identifier: BlockIdentifier {
            index: index - 1,
            hash: block.block.header.parent_hash.to_string(),
        },
        timestamp: timestamp_nanos as i64,
        transactions: vec![],
        metadata: None,
    };

    let response = BlockResponse {
        block: Some(block),
        other_transactions: None,
    };

    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

async fn block_transaction(mut req: Request<State>) -> tide::Result {
    let request: BlockTransactionRequest = match req.body_json().await {
        Ok(ok) => ok,
        Err(e) => {
            return Ok(Response::builder(400)
                .body(Body::from_json(&format!(
                    "error while parsing params {}",
                    e
                ))?)
                .build());
        }
    };

    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }

    let _block_index = request.block_identifier.index;
    let block_hash = request.block_identifier.hash;
    let block_endcoded_hash = H256::from_str(&block_hash).unwrap();

    let _transaction_identifier = request.transaction_identifier;
    let events_storage = api::storage().system().events();
    let _events = req
        .state()
        .client
        .storage()
        .fetch(&events_storage, Some(block_endcoded_hash))
        .await?
        .unwrap();

    Ok(Response::builder(200).body(Body::from_json(&"")?).build())
}

async fn construction_combine(mut _req: Request<State>) -> tide::Result {
    todo!()
}

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

async fn construction_hash(mut _req: Request<State>) -> tide::Result {
    todo!()
}

async fn construction_metadata(mut req: Request<State>) -> tide::Result {
    let request: ConstructionMetadataRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }

    let options = match request.options {
        Some(options) => options,
        None => return Error::InvalidParams.to_response(),
    };

    let received_account_from = options["from"].clone().to_string();
    let account: Result<AccountId32, Error> = received_account_from
        .parse()
        .map_err(|_| Error::InvalidAddress);
    let account = match account {
        Ok(account) => account,
        Err(error) => {
            return error.to_response();
        }
    };
    let nonce_addr = api::storage().system().account(account);
    let _entry = req
        .state()
        .client
        .storage()
        .fetch_or_default(&nonce_addr, None)
        .await?;


    Ok(Response::builder(200).body(Body::from_json(&"")?).build())
}

async fn construction_parse(mut _req: Request<State>) -> tide::Result {
    todo!()
}

async fn construction_payloads(mut _req: Request<State>) -> tide::Result {
    todo!()
}

async fn construction_preprocess(mut req: Request<State>) -> tide::Result {
    let request: ConstructionPreprocessRequest = req.body_json().await?;

    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }

    let operations = request.operations;

    let mut required_tx = vec![];
    for operation in operations.iter() {
        let acc = AccountIdentifier {
            address: operation.account.clone().unwrap().address,
            sub_account: None,
            metadata: None,
        };
        required_tx.push(acc);
    }

    let sender_address = operations
        .iter()
        .filter(|op| op.amount.clone().unwrap().value.parse::<i32>().unwrap() > 0)
        .map(|op| op.account.clone().unwrap().address)
        .collect::<Vec<String>>();

    let options_sender = sender_address[0].clone();
    let response = ConstructionPreprocessResponse {
        options: Some(json!({ "from": options_sender })),
        required_public_keys: Some(required_tx),
    };

    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

async fn construction_submit(mut _req: Request<State>) -> tide::Result {
    todo!()
}

async fn events_blocks(mut _req: Request<State>) -> tide::Result {
    todo!()
}

async fn search_transactions(mut _req: Request<State>) -> tide::Result {
    todo!()
}

async fn mempool(mut _req: Request<State>) -> tide::Result {
    todo!()
}

async fn mempool_transaction(mut _req: Request<State>) -> tide::Result {
    todo!()
}

//utils for methods
enum Error {
    UnsupportedNetwork,
    UnsupportedCurveType,
    InvalidHex,
    InvalidAddress,
    BlockNotFound,
    InvalidBlockIdentifier,
    InvalidBlockHash,
    InvalidParams,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UnsupportedNetwork => write!(f, "unsupported network"),
            Self::UnsupportedCurveType => write!(f, "unsupported curve type"),
            Self::InvalidHex => write!(f, "invalid hex"),
            Self::InvalidAddress => write!(f, "invalid address"),
            Self::BlockNotFound => write!(f, "block not found"),
            Self::InvalidBlockIdentifier => write!(f, "invalid block identifier"),
            Self::InvalidBlockHash => write!(f, "invalid block hash"),
            Self::InvalidParams => write!(f, "invalid params"),
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

//make transaction identifier here //WIP
async fn get_transactions(
    _subxt: &OnlineClient<SubstrateConfig>,
    block: &SignedBlock<SPBlock<Header<u32, BlakeTwo256>, OpaqueExtrinsic>>,
) {
    let extrinsics = block.block.extrinsics.clone();
    let _block_number = block.block.header.number;

    for (_item, _index) in extrinsics.iter().enumerate() {}
}
