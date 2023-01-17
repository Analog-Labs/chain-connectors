use anyhow::Result;
use parity_scale_codec::{Compact, Encode};
use rosetta_crypto::address::{Address, AddressFormat};
use rosetta_types::{
    AccountBalanceRequest, AccountBalanceResponse, AccountFaucetRequest, AccountIdentifier, Allow,
    Amount, Block, BlockIdentifier, BlockRequest, BlockResponse, BlockTransactionRequest,
    CallRequest, CallResponse, ConstructionCombineRequest, ConstructionCombineResponse,
    ConstructionDeriveRequest, ConstructionDeriveResponse, ConstructionHashRequest,
    ConstructionMetadataRequest, ConstructionMetadataResponse, ConstructionPayloadsRequest,
    ConstructionPayloadsResponse, ConstructionPreprocessRequest, ConstructionPreprocessResponse,
    ConstructionSubmitRequest, Currency, CurveType, MetadataRequest, NetworkIdentifier,
    NetworkListResponse, NetworkOptionsResponse, NetworkRequest, NetworkStatusResponse, Operation,
    SignatureType, SigningPayload, TransactionIdentifier, TransactionIdentifierResponse, Version,
};
use std::str::FromStr;
use std::time::Duration;
use subxt::ext::sp_core::blake2_256;
use subxt::ext::sp_core::sr25519::Signature;
use subxt::ext::sp_core::{crypto::AccountId32, H256};
use subxt::ext::sp_runtime::{MultiAddress, MultiSignature};
use subxt::tx::SubmittableExtrinsic;
use subxt::{OnlineClient, PolkadotConfig as GenericConfig};
use tide::prelude::json;
use tide::{Body, Request, Response};
use utils::{
    dynamic_constant_req, dynamic_storage_req, faucet_substrate, get_account_storage,
    get_block_events, get_block_transactions, get_call_data, get_runtime_call_data,
    get_runtime_error, get_transaction_detail, get_transfer_payload, get_unix_timestamp,
    resolve_block, string_to_err_response, Error, UnsignedTransactionData,
};

mod type_helper;
mod utils;

pub use rosetta_crypto::address::Ss58AddressFormatRegistry;

pub struct Config {
    pub rpc_url: String,
    pub network: NetworkIdentifier,
    pub currency: Currency,
    pub address_format: AddressFormat,
    pub testnet: bool,
}

impl Config {
    pub fn new(
        rpc_url: &str,
        blockchain: &str,
        network: &str,
        symbol: &str,
        decimals: u32,
        format: Ss58AddressFormatRegistry,
        testnet: bool,
    ) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            network: NetworkIdentifier {
                blockchain: blockchain.into(),
                network: network.into(),
                sub_network_identifier: None,
            },
            currency: Currency {
                decimals,
                symbol: symbol.into(),
                metadata: None,
            },
            address_format: format.into(),
            testnet,
        }
    }
}

#[derive(Clone)]
pub struct State {
    network: NetworkIdentifier,
    currency: Currency,
    address_format: AddressFormat,
    client: OnlineClient<GenericConfig>,
}

impl State {
    pub async fn new(config: &Config) -> Result<Self> {
        let client = OnlineClient::from_url(&config.rpc_url).await?;
        Ok(Self {
            network: config.network.clone(),
            currency: config.currency.clone(),
            address_format: config.address_format,
            client,
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
    if config.testnet {
        app.at("/account/faucet").post(account_faucet);
    }
    app.at("/block").post(block);
    app.at("/block/transaction").post(block_transaction);
    app.at("/call").post(call);
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

async fn network_options(mut req: Request<State>) -> tide::Result {
    let request: NetworkRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }
    let response = NetworkOptionsResponse {
        version: Version {
            rosetta_version: "1.0".into(),
            node_version: "1.0".into(),
            middleware_version: Some("1.0".into()),
            metadata: None,
        },
        allow: None,
    };
    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

async fn network_status(mut req: Request<State>) -> tide::Result {
    let request: NetworkRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }

    let unix_timestamp_millis = match get_unix_timestamp(&req.state().client).await {
        Ok(timestamp) => timestamp,
        Err(e) => return e.to_response(),
    };

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
        hash: format!("{:?}", current_block.block.header.hash()),
    };

    let genesis_block_identifier = Some(BlockIdentifier {
        index: 0,
        hash: format!("{:?}", genesis_block_hash),
    });

    let timestamp_nanos = Duration::from_millis(unix_timestamp_millis).as_nanos() as u64;

    let response = NetworkStatusResponse {
        current_block_identifier,
        current_block_timestamp: timestamp_nanos as i64,
        genesis_block_identifier,
        peers: Some(vec![]),
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

    let account_data = match get_account_storage(&req.state().client, &account).await {
        Ok(account_data) => account_data,
        Err(e) => return e.to_response(),
    };

    let response = AccountBalanceResponse {
        balances: vec![Amount {
            value: account_data.data.free.to_string(),
            currency: req.state().currency.clone(),
            metadata: None,
        }],
        block_identifier: BlockIdentifier {
            index,
            hash: format!("{:?}", hash),
        },
        metadata: None,
    };
    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

async fn account_coins(mut _req: Request<State>) -> tide::Result {
    Error::NotImplemented.to_response()
}

async fn account_faucet(mut req: Request<State>) -> tide::Result {
    let request: AccountFaucetRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }
    let data = match faucet_substrate(
        &req.state().client,
        &request.account_identifier.address,
        request.faucet_parameter,
    )
    .await
    {
        Ok(data) => data,
        Err(e) => return string_to_err_response(e),
    };

    let response = TransactionIdentifierResponse {
        transaction_identifier: TransactionIdentifier {
            hash: format!("{:?}", data),
        },
        metadata: None,
    };
    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
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

    let block_identifier = Some(request.block_identifier);
    let (block_hash, index) = resolve_block(&req.state().client, block_identifier.as_ref()).await?;
    let block = req.state().client.rpc().block(Some(block_hash)).await?;
    let block = match block {
        Some(block) => block,
        None => {
            return Error::BlockNotFound.to_response();
        }
    };

    let unix_timestamp_millis = match get_unix_timestamp(&req.state().client).await {
        Ok(timestamp) => timestamp,
        Err(e) => return e.to_response(),
    };

    let timestamp_nanos = Duration::from_millis(unix_timestamp_millis).as_nanos() as u64;

    let events = match get_block_events(&req.state().client, block_hash).await {
        Ok(events) => events,
        Err(e) => return e.to_response(),
    };

    let parent_hash = format!("{:?}", block.block.header.parent_hash);

    let transactions = match get_block_transactions(req.state(), &block, &events) {
        Ok(ok) => ok,
        Err(e) => return e.to_response(),
    };

    let block = Block {
        block_identifier: BlockIdentifier {
            index,
            hash: format!("{:?}", block_hash),
        },
        parent_block_identifier: BlockIdentifier {
            index: index.saturating_sub(1),
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
    let block_hash_str = request.block_identifier.hash;
    let block_hash = match H256::from_str(&block_hash_str) {
        Ok(ok) => ok,
        Err(_) => return Error::InvalidBlockHash.to_response(),
    };

    let transaction_identifier = request.transaction_identifier;

    let events = match get_block_events(&req.state().client, block_hash).await {
        Ok(events) => events,
        Err(e) => return e.to_response(),
    };

    let block = req.state().client.rpc().block(Some(block_hash)).await?;

    let block = match block {
        Some(block) => block,
        None => {
            return Error::BlockNotFound.to_response();
        }
    };

    let transaction =
        match get_transaction_detail(transaction_identifier.hash, req.state(), &block, &events) {
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

async fn call(mut req: Request<State>) -> tide::Result {
    let request: CallRequest = req.body_json().await?;
    if request.network_identifier != req.state().network {
        return Error::UnsupportedNetwork.to_response();
    }
    let method_name = request.method;

    let call_details = method_name.split(',').collect::<Vec<&str>>();
    if call_details.len() != 3 {
        return Error::InvalidParams.to_response();
    }

    let pallet_name = call_details[0];
    let call_name = call_details[1];
    let query_type = call_details[2];

    let val = match query_type.to_lowercase().as_str() {
        "constant" => match dynamic_constant_req(&req.state().client, pallet_name, call_name) {
            Ok(ok) => ok,
            Err(e) => return e.to_response(),
        },
        "storage" => {
            match dynamic_storage_req(
                &req.state().client,
                pallet_name,
                call_name,
                request.parameters,
            )
            .await
            {
                Ok(ok) => ok,
                Err(e) => return e.to_response(),
            }
        }
        _ => {
            return Error::InvalidParams.to_response();
        }
    };

    let call_response = CallResponse {
        result: val,
        idempotent: false,
    };

    Ok(Response::builder(200)
        .body(Body::from_json(&call_response)?)
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
    let address = String::from(Address::from_public_key_bytes(
        req.state().address_format,
        &public_key,
    ));
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
            hash: format!("0x{}", hex::encode(hash)),
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

    let sender_addresses = operations
        .iter()
        .filter(|op| {
            op.amount
                .as_ref()
                .map(|amount| amount.value.parse::<i128>().unwrap_or_default() < 0)
                .unwrap_or_default()
        })
        .map(|op| op.account.clone().unwrap().address)
        .collect::<Vec<String>>();

    if sender_addresses.len() != 1 {
        return Error::SenderNotFound.to_response();
    }
    let options_sender = sender_addresses[0].clone();
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

    let acc_data = match get_account_storage(&req.state().client, &account).await {
        Ok(acc_data) => acc_data,
        Err(e) => return e.to_response(),
    };

    let nonce = acc_data.nonce;

    let _response = ConstructionMetadataResponse {
        metadata: serde_json::json!({
            "nonce": nonce,
        }),
        suggested_fee: None,
    };

    Ok(Response::builder(200)
        .body(Body::from_json(&_response)?)
        .build())
}

async fn construction_parse(mut _req: Request<State>) -> tide::Result {
    Error::NotImplemented.to_response()
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

    let operations = request.operations;

    let (payload_data, sender_address) = if !operations.is_empty() {
        if operations.len() != 2 {
            return Error::InvalidOperationsLength.to_response();
        }

        let sender_operations = operations
            .iter()
            .filter(|op| {
                op.amount
                    .as_ref()
                    .map(|amount| amount.value.parse::<i128>().unwrap_or_default() < 0)
                    .unwrap_or_default()
            })
            .collect::<Vec<&Operation>>();

        let receiver_operations = operations
            .iter()
            .filter(|op| {
                op.amount
                    .as_ref()
                    .map(|amount| amount.value.parse::<i128>().unwrap_or_default() > 0)
                    .unwrap_or_default()
            })
            .collect::<Vec<&Operation>>();

        if sender_operations.len() != 1 || receiver_operations.len() != 1 {
            return Error::InvalidOperationsLength.to_response();
        }

        let sender_address = match sender_operations[0].account.clone() {
            Some(account) => account.address,
            None => return Error::SenderNotFound.to_response(),
        };

        let receiver_address = match receiver_operations[0].account.clone() {
            Some(account) => account.address,
            None => return Error::ReceiverNotFound.to_response(),
        };

        let amount = receiver_operations[0]
            .amount
            .as_ref()
            .map(|amount| amount.value.parse::<u128>().unwrap_or_default())
            .unwrap_or_default();

        if amount == 0 {
            return Error::InvalidAmount.to_response();
        }
        let receiver_account: Result<AccountId32, Error> =
            receiver_address.parse().map_err(|_| Error::InvalidAddress);
        let receiver_account = match receiver_account {
            Ok(account) => account,
            Err(error) => {
                return error.to_response();
            }
        };

        let payload = match get_transfer_payload(
            &req.state().client,
            MultiAddress::Id(receiver_account),
            amount,
        ) {
            Ok(payload) => payload,
            Err(e) => return e.to_response(),
        };

        let payload_data = match get_call_data(&payload, &req.state().client, nonce as u32) {
            Ok(payload_data) => payload_data,
            Err(_) => return Error::CouldNotCreateCallData.to_response(),
        };

        (payload_data, sender_address)
    } else {
        let pallet_name = match metadata["pallet_name"].as_str() {
            Some(pallet_name) => pallet_name,
            None => return Error::InvalidParams.to_response(),
        };
        let call_name = match metadata["call_name"].as_str() {
            Some(call_name) => call_name,
            None => return Error::InvalidParams.to_response(),
        };
        let params = metadata["params"].clone();
        let sender_address = match metadata["sender_address"].as_str() {
            Some(sender_address) => sender_address,
            None => return Error::InvalidParams.to_response(),
        };
        let dynamic_tx_payload =
            match get_runtime_call_data(&req.state().client, pallet_name, call_name, params) {
                Ok(dynamic_tx_payload) => dynamic_tx_payload,
                Err(error) => {
                    return error.to_response();
                }
            };

        let payload_data =
            match get_call_data(&dynamic_tx_payload, &req.state().client, nonce as u32) {
                Ok(payload_data) => payload_data,
                Err(e) => return e.to_response(),
            };

        (payload_data, sender_address.to_string())
    };

    let tx_hex = hex::encode(&payload_data.payload);

    let signing_payload = SigningPayload {
        address: Some(sender_address.clone()),
        account_identifier: Some(AccountIdentifier {
            address: sender_address.clone(),
            sub_account: None,
            metadata: None,
        }),
        hex_bytes: tx_hex,
        signature_type: Some(SignatureType::Sr25519),
    };

    let unsigned_tx = UnsignedTransactionData {
        signer_address: sender_address,
        additional_parmas: payload_data.additional_params,
        call_data: payload_data.call_data,
    };

    let usigned_tx = match serde_json::to_string(&unsigned_tx) {
        Ok(usigned_tx) => usigned_tx,
        Err(_) => return Error::CouldNotSerialize.to_response(),
    };

    let response = ConstructionPayloadsResponse {
        unsigned_transaction: usigned_tx,
        payloads: vec![signing_payload],
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

    let unsigned_transaction = request.unsigned_transaction;
    let signatures = request.signatures;

    if signatures.len() != 1 {
        return Error::MoreThanOneSignature.to_response();
    }

    let signature = signatures[0].clone();

    if signature.signature_type != SignatureType::Sr25519 {
        return Error::InvalidSignatureType.to_response();
    }

    let signature = signature.hex_bytes;

    let sig_bytes = match hex::decode(signature) {
        Ok(sig_bytes) => sig_bytes,
        Err(_) => return Error::InvalidHex.to_response(),
    };

    let sig_slice: &[u8] = &sig_bytes;
    let sig = match Signature::try_from(sig_slice) {
        Ok(sig) => sig,
        Err(_) => return Error::InvalidSignature.to_response(),
    };
    let multisig = MultiSignature::Sr25519(sig);

    let unsigned_tx_data: UnsignedTransactionData =
        match serde_json::from_str(&unsigned_transaction) {
            Ok(unsigned_tx_data) => unsigned_tx_data,
            Err(_) => return Error::CouldNotDeserialize.to_response(),
        };

    let signer_addr = unsigned_tx_data.signer_address;

    let sender_account: Result<AccountId32, Error> =
        signer_addr.parse().map_err(|_| Error::InvalidAddress);
    let sender_account = match sender_account {
        Ok(account) => account,
        Err(error) => {
            return error.to_response();
        }
    };

    let sender_multiaddr: MultiAddress<AccountId32, u32> = MultiAddress::Id(sender_account);

    let extrinsic = {
        let mut encoded_inner = Vec::new();
        // "is signed" + transaction protocol version (4)
        (0b10000000 + 4u8).encode_to(&mut encoded_inner);
        // from address for signature
        sender_multiaddr.encode_to(&mut encoded_inner);
        // signature encode pending to vector
        multisig.encode_to(&mut encoded_inner);
        // attach custom extra params
        encoded_inner.extend(unsigned_tx_data.additional_parmas);
        // and now, call data
        encoded_inner.extend(unsigned_tx_data.call_data);

        // now, prefix byte length:
        let len = Compact(
            u32::try_from(encoded_inner.len()).expect("extrinsic size expected to be <4GB"),
        );
        let mut encoded = Vec::new();
        len.encode_to(&mut encoded);
        encoded.extend(encoded_inner);
        encoded
    };

    let tx_hex = hex::encode(extrinsic);
    let response = ConstructionCombineResponse {
        signed_transaction: format!("0x{}", tx_hex),
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

    let received_tx_hash = request.signed_transaction;
    let tx_hash = received_tx_hash.trim_start_matches("0x");
    let encoded_tx_data = match hex::decode(tx_hash) {
        Ok(data) => data,
        Err(_) => return Error::InvalidHex.to_response(),
    };

    let sb_extrinsic =
        SubmittableExtrinsic::from_bytes(req.state().client.clone(), encoded_tx_data);

    let tx_progress = match sb_extrinsic.submit_and_watch().await {
        Ok(tx_progress) => tx_progress,
        Err(error) => {
            return string_to_err_response(get_runtime_error(error));
        }
    };

    let status = match tx_progress.wait_for_finalized_success().await {
        Ok(status) => status,
        Err(error) => {
            return string_to_err_response(get_runtime_error(error));
        }
    };

    let tx_hash = format!("{:?}", status.extrinsic_hash());

    let response = TransactionIdentifierResponse {
        transaction_identifier: TransactionIdentifier { hash: tx_hash },
        metadata: None,
    };

    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}

async fn events_blocks(mut _req: Request<State>) -> tide::Result {
    Error::NotImplemented.to_response()
}

async fn search_transactions(mut _req: Request<State>) -> tide::Result {
    Error::NotImplemented.to_response()
}

async fn mempool(_req: Request<State>) -> tide::Result {
    Error::NotImplemented.to_response()
}

async fn mempool_transaction(_req: Request<State>) -> tide::Result {
    Error::NotImplemented.to_response()
}
