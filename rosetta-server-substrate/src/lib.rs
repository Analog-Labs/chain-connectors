use anyhow::Result;
use chains::substrate::api;
use chains::substrate::api::system::constants;
use rosetta_types::{
    AccountBalanceRequest, AccountBalanceResponse, AccountIdentifier, Amount, Block,
    BlockIdentifier, BlockRequest, BlockResponse, BlockTransactionRequest,
    ConstructionCombineRequest, ConstructionCombineResponse, ConstructionDeriveRequest,
    ConstructionDeriveResponse, ConstructionHashRequest, ConstructionMetadataRequest,
    ConstructionMetadataResponse, ConstructionPayloadsRequest, ConstructionPayloadsResponse,
    ConstructionPreprocessRequest, ConstructionPreprocessResponse, ConstructionSubmitRequest,
    Currency, CurveType, MetadataRequest, NetworkIdentifier, NetworkListResponse, NetworkRequest,
    NetworkStatusResponse, Operation, SignatureType, SigningPayload, TransactionIdentifier,
    TransactionIdentifierResponse,
};
use ss58_registry::{Ss58AddressFormat, Ss58AddressFormatRegistry};
use std::str::FromStr;
use std::time::Duration;
use subxt::ext::sp_core::blake2_256;
use subxt::ext::sp_core::{crypto::AccountId32, H256};
use subxt::ext::sp_runtime::MultiAddress;
use subxt::tx::{AssetTip, SubstrateExtrinsicParamsBuilder as Params};
use subxt::tx::{Era, PlainTip, TxPayload};
use subxt::{OnlineClient, SubstrateConfig};
use tide::prelude::json;
use tide::{Body, Request, Response};
use utils::{
    encode_call_data, get_block_transactions, get_transaction_detail, resolve_block, Error,
};

mod chains;
mod ss58;
mod utils;

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
    let current_block = match req.state().client.rpc().block(None).await {
        Ok(block) => match block {
            Some(block) => block,
            None => return Error::BlockNotFound.to_response(),
        },
        Err(_) => return Error::BlockNotFound.to_response(),
    };

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
        .fetch_or_default(&current_block_timestamp, None)
        .await?;

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
        .fetch_or_default(&timestamp, Some(block_hash))
        .await?;

    let timestamp_nanos = Duration::from_millis(unix_timestamp_millis).as_nanos() as u64;

    let events_storage = api::storage().system().events();
    let events = req
        .state()
        .client
        .storage()
        .fetch_or_default(&events_storage, Some(block_hash))
        .await?;

    let parent_hash = block.block.header.parent_hash.to_string();

    let transactions = match get_block_transactions(req.state(), block, &events) {
        Ok(ok) => ok,
        Err(e) => return e.to_response(),
    };

    let block = Block {
        block_identifier: BlockIdentifier {
            index,
            hash: block_hash.to_string(),
        },
        parent_block_identifier: BlockIdentifier {
            index: index - 1,
            hash: parent_hash,
        },
        timestamp: timestamp_nanos as i64,
        transactions,
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
    let block_endcoded_hash = match H256::from_str(&block_hash) {
        Ok(ok) => ok,
        Err(_) => return Error::InvalidBlockHash.to_response(),
    };

    let transaction_identifier = request.transaction_identifier;
    let events_storage = api::storage().system().events();
    let events = req
        .state()
        .client
        .storage()
        .fetch(&events_storage, Some(block_endcoded_hash))
        .await?
        .unwrap_or_default();

    let block = req
        .state()
        .client
        .rpc()
        .block(Some(block_endcoded_hash))
        .await?;

    let block = match block {
        Some(block) => block,
        None => {
            return Error::BlockNotFound.to_response();
        }
    };

    let transaction =
        match get_transaction_detail(transaction_identifier.hash, req.state(), block, &events) {
            Ok(transaction) => match transaction {
                Some(transaction_inner) => transaction_inner,
                None => {
                    return Error::TransactionNotFound.to_response();
                }
            },
            Err(e) => return e.to_response(),
        };

    Ok(Response::builder(200)
        .body(Body::from_json(&transaction)?)
        .build())
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

async fn construction_hash(mut req: Request<State>) -> tide::Result {
    let request: ConstructionHashRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }
    let received_hex = request.signed_transaction.trim_start_matches("0x");
    let transaction = match hex::decode(received_hex) {
        Ok(transaction) => transaction,
        Err(_) => return Error::InvalidHex.to_response(),
    };
    let hash = blake2_256(&transaction);
    let response = TransactionIdentifierResponse {
        transaction_identifier: TransactionIdentifier {
            hash: format!("{}{}", "0x", hex::encode(hash)),
        },
        metadata: None,
    };
    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

async fn construction_preprocess(mut req: Request<State>) -> tide::Result {
    let request: ConstructionPreprocessRequest = req.body_json().await?;

    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }

    let operations = request.operations;

    let mut required_tx = vec![];

    for operation in operations.iter() {
        let acc_address = match operation.account.clone() {
            Some(account) => account.address,
            None => return Error::InvalidParams.to_response(),
        };

        let acc = AccountIdentifier {
            address: acc_address,
            sub_account: None,
            metadata: None,
        };

        required_tx.push(acc);
    }

    let sender_address = operations
        .iter()
        .filter(|op| {
            op.amount
                .as_ref()
                .map(|amount| amount.value.parse::<i32>().unwrap() < 0)
                .unwrap_or_default()
        })
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

async fn construction_metadata(mut req: Request<State>) -> tide::Result {
    let request: ConstructionMetadataRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }

    let options = match request.options {
        Some(options) => options,
        None => return Error::InvalidParams.to_response(),
    };

    let received_account_from = match options["from"].as_str() {
        Some(received_account_from) => received_account_from,
        None => return Error::InvalidParams.to_response(),
    };

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
    let entry = req
        .state()
        .client
        .storage()
        .fetch_or_default(&nonce_addr, None)
        .await?;

    let nonce = entry.nonce;
    let prefix = api::constants().system().ss58_prefix();
    let ss8_prefix = match req.state().client.constants().at(&prefix) {
        Ok(ss8_prefix) => ss8_prefix,
        Err(_) => return Error::InvalidParams.to_response(),
    };

    let current_block = match req.state().client.rpc().block(None).await {
        Ok(block) => match block {
            Some(block) => block,
            None => return Error::BlockNotFound.to_response(),
        },
        Err(_) => return Error::BlockNotFound.to_response(),
    };

    // let b_weights = api::constants().system().block_weights();
    // let block_weights = match req.state().client.constants().at(&b_weights){
    //     Ok(ss8_prefix) => ss8_prefix,
    //     Err(_) => return Error::InvalidParams.to_response(),
    // };

    let _response = ConstructionMetadataResponse {
        metadata: serde_json::json!({
            "nonce": nonce,
            "account_sequence": ss8_prefix,
            "recent_block_hash": current_block.block.header.hash(),
        }),
        suggested_fee: None,
    };

    Ok(Response::builder(200)
        .body(Body::from_json(&_response)?)
        .build())
}

async fn construction_parse(mut _req: Request<State>) -> tide::Result {
    todo!()
}

async fn construction_payloads(mut req: Request<State>) -> tide::Result {
    let request: ConstructionPayloadsRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }

    let metadata = match request.metadata {
        Some(options) => options,
        None => return Error::InvalidParams.to_response(),
    };

    let nonce = match metadata["nonce"].as_u64() {
        Some(nonce) => nonce,
        None => return Error::InvalidParams.to_response(),
    };

    let account_sequence = match metadata["account_sequence"].as_u64() {
        Some(account_sequence) => account_sequence,
        None => return Error::InvalidParams.to_response(),
    };

    let recent_block_hash = match metadata["recent_block_hash"].as_str() {
        Some(recent_block_hash) => recent_block_hash,
        None => return Error::InvalidParams.to_response(),
    };

    let operations = request.operations;

    //for transafer check
    if operations.len() != 2 {
        return Error::InvalidParams.to_response();
    }

    let sender_operations = operations
        .iter()
        .filter(|op| {
            op.amount
                .as_ref()
                .map(|amount| amount.value.parse::<i32>().unwrap() < 0)
                .unwrap_or_default()
        })
        .collect::<Vec<&Operation>>();

    let receiver_operations = operations
        .iter()
        .filter(|op| {
            op.amount
                .as_ref()
                .map(|amount| amount.value.parse::<i32>().unwrap() > 0)
                .unwrap_or_default()
        })
        .collect::<Vec<&Operation>>();

    if sender_operations.len() != 1 || receiver_operations.len() != 1 {
        return Error::InvalidParams.to_response();
    }

    let sender_address = match sender_operations[0].account.clone() {
        Some(account) => account.address,
        None => return Error::InvalidParams.to_response(),
    };

    let receiver_address = match receiver_operations[0].account.clone() {
        Some(account) => account.address,
        None => return Error::InvalidParams.to_response(),
    };

    let amount = receiver_operations[0]
        .amount
        .as_ref()
        .map(|amount| amount.value.parse::<u128>().unwrap())
        .unwrap_or_default();

    let receiver_account: Result<AccountId32, Error> =
        receiver_address.parse().map_err(|_| Error::InvalidAddress);
    let receiver_account = match receiver_account {
        Ok(account) => account,
        Err(error) => {
            return error.to_response();
        }
    };

    let tx_params = Params::new()
        .tip(AssetTip::new(0))
        .era(Era::Immortal, req.state().client.genesis_hash());

    let payload = api::tx()
        .balances()
        .transfer(MultiAddress::Id(receiver_account), amount);
    let encoded_tx = match encode_call_data(&payload, &req.state().client, nonce as u32, tx_params){
        Ok(encoded_tx) => encoded_tx,
        Err(_) => return Error::InvalidParams.to_response(),
    };

    let tx_hex = hex::encode(&encoded_tx);

    let signing_payload = SigningPayload {
        address: Some(sender_address.clone()),
        account_identifier: Some(AccountIdentifier {
            address: sender_address,
            sub_account: None,
            metadata: None,
        }),
        hex_bytes: tx_hex.clone(),
        signature_type: Some(SignatureType::Ed25519),
    };

    let response = ConstructionPayloadsResponse {
        unsigned_transaction: format!("0x{}", tx_hex),
        payloads: vec![signing_payload],
        // metadata: None,
    };

    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

async fn construction_combine(mut req: Request<State>) -> tide::Result {
    let request: ConstructionCombineRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }

    let unsinged_transaction = request.unsigned_transaction;
    let signatures = request.signatures;

    let response = ConstructionCombineResponse {
        signed_transaction: "".into(),
    };

    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

async fn construction_submit(mut req: Request<State>) -> tide::Result {
    let request: ConstructionSubmitRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }

    // let signed_transaction = request.signed_transaction;

    // let response = TransactionIdentifierResponse {
    //     transaction_identifier: TransactionIdentifier {
    //         hash: signed_transaction,
    //     },
    //     metadata: None,
    // };

    Ok(Response::builder(200).body(Body::from_json(&"")?).build())
}

async fn events_blocks(mut _req: Request<State>) -> tide::Result {
    todo!()
}

async fn search_transactions(mut _req: Request<State>) -> tide::Result {
    todo!()
}

async fn mempool(_req: Request<State>) -> tide::Result {
    Error::NotImplemented.to_response()
}

async fn mempool_transaction(_req: Request<State>) -> tide::Result {
    Error::NotImplemented.to_response()
}
