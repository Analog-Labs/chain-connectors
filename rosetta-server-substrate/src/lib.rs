use anyhow::Result;
use api::runtime_types::frame_system;
use api::runtime_types::kitchensink_runtime::RuntimeEvent;
use chains::substrate::api;
use chains::substrate::api::runtime_types::frame_system::Phase;
use parity_scale_codec::Encode;
use rosetta_types::{
    AccountBalanceRequest, AccountBalanceResponse, AccountIdentifier, Amount, Block,
    BlockIdentifier, BlockRequest, BlockResponse, BlockTransactionRequest,
    ConstructionDeriveRequest, ConstructionDeriveResponse, ConstructionMetadataRequest,
    ConstructionPreprocessRequest, ConstructionPreprocessResponse, Currency, CurveType,
    MetadataRequest, NetworkIdentifier, NetworkListResponse, NetworkRequest, NetworkStatusResponse,
    Operation, OperationIdentifier, PartialBlockIdentifier, SubAccountIdentifier, Transaction,
    TransactionIdentifier,
};
use serde_json::Value;
use ss58_registry::{Ss58AddressFormat, Ss58AddressFormatRegistry};
use std::str::FromStr;
use std::time::Duration;
use subxt::ext::sp_core::{crypto::AccountId32, H256};
use subxt::ext::sp_runtime::generic::{Block as SPBlock, Header, SignedBlock};
use subxt::ext::sp_runtime::traits::BlakeTwo256;
use subxt::ext::sp_runtime::OpaqueExtrinsic;
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
    let events = req
        .state()
        .client
        .storage()
        .fetch(&events_storage, Some(block_hash))
        .await?
        .unwrap();

    //get transactions data
    let transactions = match get_transactions(req.state(), &block, &events) {
        Ok(ok) => ok,
        Err(e) => return e.to_response(),
    };

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
    let block_endcoded_hash = H256::from_str(&block_hash).unwrap();

    let transaction_identifier = request.transaction_identifier;
    let events_storage = api::storage().system().events();
    let events = req
        .state()
        .client
        .storage()
        .fetch(&events_storage, Some(block_endcoded_hash))
        .await?
        .unwrap();

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
        match get_transaction(transaction_identifier.hash, req.state(), &block, &events) {
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

async fn mempool(_req: Request<State>) -> tide::Result {
    Error::NotImplemented.to_response()
}

async fn mempool_transaction(_req: Request<State>) -> tide::Result {
    Error::NotImplemented.to_response()
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
    NotImplemented,
    OperationParse,
    TransactionNotFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UnsupportedNetwork => write!(f, "Unsupported network"),
            Self::UnsupportedCurveType => write!(f, "Unsupported curve type"),
            Self::InvalidHex => write!(f, "Invalid hex"),
            Self::InvalidAddress => write!(f, "Invalid address"),
            Self::BlockNotFound => write!(f, "Block not found"),
            Self::InvalidBlockIdentifier => write!(f, "Invalid block identifier"),
            Self::InvalidBlockHash => write!(f, "Invalid block hash"),
            Self::InvalidParams => write!(f, "Invalid params"),
            Self::NotImplemented => write!(f, "Not implemented"),
            Self::OperationParse => write!(f, "Operation parse error"),
            Self::TransactionNotFound => write!(f, "Transaction not found"),
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

//make transaction identifier here
fn get_transactions(
    state: &State,
    block: &SignedBlock<SPBlock<Header<u32, BlakeTwo256>, OpaqueExtrinsic>>,
    events: &[frame_system::EventRecord<RuntimeEvent, H256>],
) -> Result<Vec<Transaction>, Error> {
    let mut vec_of_extrinsics = vec![];

    let extrinsics = block.block.extrinsics.clone();
    let _block_number = block.block.header.number;

    for (ex_index, extrinsic) in extrinsics.iter().enumerate() {
        let encoded_item: &[u8] = &extrinsic.encode();
        let hex_val = hex::encode(encoded_item);
        let mut vec_of_operations = vec![];

        let transaction_identifier = TransactionIdentifier {
            hash: hex_val.clone(),
        };

        let events_for_current_extrinsic = events
            .iter()
            .filter(|e| e.phase == Phase::ApplyExtrinsic(ex_index as u32))
            .collect::<Vec<&frame_system::EventRecord<RuntimeEvent, H256>>>();

        for (event_index, event) in events_for_current_extrinsic.iter().enumerate() {
            let operation_identifier = OperationIdentifier {
                index: event_index as i64,
                network_index: None,
            };
            let json_string = serde_json::to_string(&event.event).unwrap();
            let json_event: Value = serde_json::from_str(&json_string).unwrap();
            let event_parsed_data = match get_operation_data(json_event.clone()) {
                Ok(data) => data,
                Err(e) => return Err(e),
            };

            let op_account: Option<AccountIdentifier> = match event_parsed_data.from {
                Some(from) => match event_parsed_data.to {
                    Some(to) => Some(AccountIdentifier {
                        address: from,
                        sub_account: Some(SubAccountIdentifier {
                            address: to,
                            metadata: None,
                        }),
                        metadata: None,
                    }),
                    None => Some(AccountIdentifier {
                        address: from,
                        sub_account: None,
                        metadata: None,
                    }),
                },
                None => None,
            };

            let op_amount: Option<Amount> = event_parsed_data.amount.map(|amount| Amount {
                value: amount,
                currency: state.currency.clone(),
                metadata: None,
            });

            let operation = Operation {
                operation_identifier,
                related_operations: None,
                r#type: event_parsed_data.event_type,
                status: None,
                account: op_account,
                amount: op_amount,
                coin_change: None,
                metadata: Some(json_event),
            };

            vec_of_operations.push(operation)
        }

        let transaction = Transaction {
            transaction_identifier,
            operations: vec_of_operations,
            related_transactions: None,
            metadata: None,
        };

        vec_of_extrinsics.push(transaction);
    }
    Ok(vec_of_extrinsics)
}

fn get_operation_data(data: Value) -> Result<TransactionOperationStatus, Error> {
    let root_object = match data.as_object() {
        Some(root_obj) => root_obj,
        None => return Err(Error::OperationParse),
    };
    let pallet_name = match root_object.keys().next() {
        Some(pallet_name) => pallet_name,
        None => return Err(Error::OperationParse),
    };

    let pallet_object = match root_object.get(pallet_name) {
        Some(pallet_obj) => match pallet_obj.as_object() {
            Some(pallet_obj_inner) => pallet_obj_inner,
            None => return Err(Error::OperationParse),
        },
        None => return Err(Error::OperationParse),
    };

    let event_name = match pallet_object.keys().next() {
        Some(event_name) => event_name,
        None => return Err(Error::OperationParse),
    };

    let event_object = match pallet_object.get(event_name) {
        Some(event_obj) => match event_obj.as_object() {
            Some(event_obj_inner) => event_obj_inner,
            None => return Err(Error::OperationParse),
        },
        None => return Err(Error::OperationParse),
    };

    let call_type = format!("{}.{}", pallet_name.clone(), event_name.clone());

    let amount: Option<String> = match event_object.get("amount") {
        Some(amount) => Some(amount.to_string()),
        None => event_object
            .get("actual_fee")
            .map(|actual_fee| actual_fee.to_string()),
    };

    let who: Option<String> = match event_object.get("who") {
        Some(who) => Some(who.as_str().unwrap().to_string()),
        None => match event_object.get("account") {
            Some(account) => Some(account.as_str().unwrap().to_string()),
            None => event_object
                .get("from")
                .map(|from| from.as_str().unwrap().to_string()),
        },
    };

    let to: Option<String> = event_object
        .get("to")
        .map(|to| to.as_str().unwrap().to_string());

    let transaction_operation_status = TransactionOperationStatus {
        event_type: call_type,
        amount,
        from: who,
        to,
    };

    Ok(transaction_operation_status)
}

fn get_transaction(
    transaction_hash: String,
    state: &State,
    block: &SignedBlock<SPBlock<Header<u32, BlakeTwo256>, OpaqueExtrinsic>>,
    events: &[frame_system::EventRecord<RuntimeEvent, H256>],
) -> Result<Option<Transaction>, Error> {
    let tx_hash = transaction_hash.trim_start_matches("0x");
    let extrinsics = block.block.extrinsics.clone();
    for (ex_index, extrinsic) in extrinsics.iter().enumerate() {
        let encoded_item: &[u8] = &extrinsic.encode();
        let hex_val = hex::encode(encoded_item);

        if hex_val.eq(&tx_hash) {
            let mut vec_of_operations = vec![];
            let transaction_identifier = TransactionIdentifier { hash: hex_val };

            let events_for_current_extrinsic = events
                .iter()
                .filter(|e| e.phase == Phase::ApplyExtrinsic(ex_index as u32))
                .collect::<Vec<&frame_system::EventRecord<RuntimeEvent, H256>>>();

            for (event_index, event) in events_for_current_extrinsic.iter().enumerate() {
                let operation_identifier = OperationIdentifier {
                    index: event_index as i64,
                    network_index: None,
                };
                let json_string = serde_json::to_string(&event.event).unwrap();
                let json_event: Value = serde_json::from_str(&json_string).unwrap();
                let event_parsed_data = match get_operation_data(json_event.clone()) {
                    Ok(event_parsed_data) => event_parsed_data,
                    Err(e) => return Err(e),
                };

                let op_account: Option<AccountIdentifier> = match event_parsed_data.from {
                    Some(from) => match event_parsed_data.to {
                        Some(to) => Some(AccountIdentifier {
                            address: from,
                            sub_account: Some(SubAccountIdentifier {
                                address: to,
                                metadata: None,
                            }),
                            metadata: None,
                        }),
                        None => Some(AccountIdentifier {
                            address: from,
                            sub_account: None,
                            metadata: None,
                        }),
                    },
                    None => None,
                };

                let op_amount: Option<Amount> = event_parsed_data.amount.map(|amount| Amount {
                    value: amount,
                    currency: state.currency.clone(),
                    metadata: None,
                });

                let operation = Operation {
                    operation_identifier,
                    related_operations: None,
                    r#type: event_parsed_data.event_type,
                    status: None,
                    account: op_account,
                    amount: op_amount,
                    coin_change: None,
                    metadata: Some(json_event),
                };

                vec_of_operations.push(operation)
            }

            let transaction = Transaction {
                transaction_identifier,
                operations: vec_of_operations,
                related_transactions: None,
                metadata: None,
            };
            return Ok(Some(transaction));
        }
    }
    Ok(None)
}

struct TransactionOperationStatus {
    event_type: String,
    from: Option<String>,
    to: Option<String>,
    amount: Option<String>,
}
