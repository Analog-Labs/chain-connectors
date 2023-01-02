use crate::State;
use anyhow::Result;
use parity_scale_codec::{Decode, Encode};
use rosetta_crypto::address::{Address, AddressFormat};
use rosetta_types::AccountIdentifier;
use rosetta_types::Amount;
use rosetta_types::{
    Operation, OperationIdentifier, PartialBlockIdentifier, SubAccountIdentifier, Transaction,
    TransactionIdentifier,
};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use sp_keyring::AccountKeyring;
use std::borrow::Borrow;
use std::fmt::format;
use subxt::events::EventDetails;
use subxt::events::Events;
use subxt::events::Phase;
use subxt::ext::scale_value::scale::TypeId;
use subxt::ext::scale_value::Composite;
use subxt::ext::scale_value::Primitive;
use subxt::ext::scale_value::ValueDef;
use subxt::ext::sp_core;
use subxt::ext::sp_core::blake2_256;
use subxt::ext::sp_core::H256;
use subxt::ext::sp_runtime::AccountId32;
use subxt::ext::sp_runtime::MultiAddress;
use subxt::metadata::DecodeStaticType;
use subxt::rpc::RpcParams;
use subxt::rpc::{BlockNumber, ChainBlockExtrinsic, ChainBlockResponse};
use subxt::rpc_params;
use subxt::storage::address;
use subxt::storage::address::StorageHasher;
use subxt::storage::address::StorageMapKey;
use subxt::storage::StaticStorageAddress;
use subxt::tx::PairSigner;
use subxt::tx::StaticTxPayload;
use subxt::tx::{ExtrinsicParams, TxPayload};
use subxt::utils::Encoded;
use subxt::Config;
use subxt::Error as SubxtError;
use subxt::{OnlineClient, PolkadotConfig as GenericConfig};
use tide::{Body, Response};

#[derive(Debug)]
pub enum Error {
    AccountNotFound,
    UnsupportedNetwork,
    UnsupportedCurveType,
    InvalidHex,
    InvalidAddress,
    BlockNotFound,
    InvalidBlockHash,
    InvalidParams,
    NotImplemented,
    OperationParse,
    TransactionNotFound,
    CouldNotSerialize,
    CouldNotDeserialize,
    MoreThanOneSignature,
    InvalidSignatureType,
    CouldNotCreateCallData,
    InvalidExtrinsic,
    InvalidOperationsLength,
    SenderNotFound,
    ReceiverNotFound,
    InvalidSignature,
    InvalidCallData,
    InvalidAmount,
    InvalidMetadata,
    EventDetailParse,
    NoBlockEvents,
    FailedTimestamp,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AccountNotFound => write!(f, "Account not found"),
            Self::UnsupportedNetwork => write!(f, "Unsupported network"),
            Self::UnsupportedCurveType => write!(f, "Unsupported curve type"),
            Self::InvalidHex => write!(f, "Invalid hex"),
            Self::InvalidAddress => write!(f, "Invalid address"),
            Self::BlockNotFound => write!(f, "Block not found"),
            Self::InvalidBlockHash => write!(f, "Invalid block hash"),
            Self::InvalidParams => write!(f, "Invalid params"),
            Self::NotImplemented => write!(f, "Not implemented"),
            Self::OperationParse => write!(f, "Operation parse error"),
            Self::TransactionNotFound => write!(f, "Transaction not found"),
            Self::CouldNotSerialize => write!(f, "Serializer error"),
            Self::MoreThanOneSignature => write!(f, "More than one signature"),
            Self::InvalidSignatureType => write!(f, "Invalid signature type"),
            Self::CouldNotCreateCallData => write!(f, "Could not create call data"),
            Self::InvalidExtrinsic => write!(f, "Invalid extrinsic"),
            Self::InvalidOperationsLength => write!(f, "Invalid operations length"),
            Self::SenderNotFound => write!(f, "Sender not found"),
            Self::ReceiverNotFound => write!(f, "Receiver not found"),
            Self::CouldNotDeserialize => write!(f, "Could not deserialize"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InvalidCallData => write!(f, "Invalid call data"),
            Self::InvalidAmount => write!(f, "Invalid amount"),
            Self::InvalidMetadata => write!(f, "Metadata error"),
            Self::EventDetailParse => write!(f, "Event detail parse error"),
            Self::NoBlockEvents => write!(f, "No block events found"),
            Self::FailedTimestamp => write!(f, "Failed to get timestamp"),
        }
    }
}

impl Error {
    pub fn to_response(&self) -> tide::Result {
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

pub async fn resolve_block(
    subxt: &OnlineClient<GenericConfig>,
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

pub async fn get_block_events<T>(
    client: &OnlineClient<T>,
    block: <T as subxt::Config>::Hash,
) -> Result<Events<T>, Error>
where
    T: Config,
{
    let abc = client
        .events()
        .at(Some(block))
        .await
        .map_err(|_| Error::NoBlockEvents)?;

    Ok(abc)
}

pub fn get_block_transactions<T: Config>(
    state: &State,
    block: &ChainBlockResponse<T>,
    events: &Events<T>,
) -> Result<Vec<Transaction>, Error> {
    let mut vec_of_extrinsics = vec![];
    for (ex_index, extrinsic) in block.block.extrinsics.iter().enumerate() {
        let hex_val = convert_extrinsic_to_hash(extrinsic);

        let mut vec_of_operations = vec![];

        let transaction_identifier = TransactionIdentifier {
            hash: hex_val.clone(),
        };

        for (event_index, event_data) in events.iter().enumerate() {
            let event = event_data.map_err(|_| Error::EventDetailParse)?;
            if event.phase() == Phase::ApplyExtrinsic(ex_index as u32) {
                let operation_identifier = OperationIdentifier {
                    index: event_index as i64,
                    network_index: None,
                };

                let event_metadata = event.event_metadata();
                let mut vec_metadata = vec![];
                for event in event_metadata.fields().iter() {
                    let name = event.name();
                    let type_name = event.type_name();
                    vec_metadata.push(json!({"name":name, "type": type_name}));
                }
                let op_metadata = Value::Array(vec_metadata);

                let event_parsed_data = get_operation_data(event, &state.address_format)?;

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
                    metadata: Some(op_metadata),
                };

                vec_of_operations.push(operation)
            }
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

pub fn get_operation_data(
    event: EventDetails,
    address_format: &AddressFormat,
) -> Result<TransactionOperationStatus, Error> {
    let pallet_name = event.pallet_name();
    let event_name = event.variant_name();

    let call_type = format!("{}.{}", pallet_name, event_name);

    let event_fields = event.field_values().map_err(|_| Error::OperationParse)?;
    let parsed_data = match event_fields {
        subxt::ext::scale_value::Composite::Named(value) => {
            let from_data = value
                .iter()
                .filter(|(k, _)| k == "from" || k == "who" || k == "account")
                .collect::<Vec<_>>();

            let sender_address: Option<String> = if !from_data.is_empty() {
                let data = from_data.into_iter().next().ok_or(Error::OperationParse)?;

                let address = generate_address(data.1.value.clone(), address_format)?;
                Some(address)
            } else {
                None
            };

            let amount_data = value
                .iter()
                .filter(|(k, _)| k == "amount" || k == "actual_fee")
                .collect::<Vec<_>>();

            let amount: Option<String> = if !amount_data.is_empty() {
                let value = amount_data
                    .into_iter()
                    .next()
                    .ok_or(Error::OperationParse)?;

                match value.1.value.clone() {
                    ValueDef::Primitive(Primitive::U128(amount)) => Some(amount.to_string()),
                    _ => {
                        return Err(Error::OperationParse);
                    }
                }
            } else {
                None
            };

            let to_data = value.iter().filter(|(k, _)| k == "to").collect::<Vec<_>>();

            let to_address: Option<String> = if !to_data.is_empty() {
                let data = to_data.into_iter().next().ok_or(Error::OperationParse)?;

                let address = generate_address(data.1.value.clone(), address_format)?;
                Some(address)
            } else {
                None
            };

            (sender_address, amount, to_address)
        }
        _ => {
            return Err(Error::OperationParse);
        }
    };

    let transaction_operation_status = TransactionOperationStatus {
        event_type: call_type,
        from: parsed_data.0,
        amount: parsed_data.1,
        to: parsed_data.2,
    };
    Ok(transaction_operation_status)
}

pub fn generate_address(
    val: ValueDef<TypeId>,
    address_format: &AddressFormat,
) -> Result<String, Error> {
    let mut addr_array: Vec<u8> = vec![];
    match val {
        ValueDef::Composite(Composite::Unnamed(unamed_data)) => {
            for value_data in unamed_data {
                match value_data.value {
                    ValueDef::Composite(data) => {
                        for data in data.into_values() {
                            match data.value {
                                ValueDef::Primitive(Primitive::U128(val)) => {
                                    addr_array.push(val as u8);
                                }
                                _ => return Err(Error::OperationParse),
                            }
                        }
                    }
                    _ => return Err(Error::OperationParse),
                }
            }
        }
        _ => return Err(Error::OperationParse),
    }

    let address = Address::from_public_key_bytes(*address_format, &addr_array);
    Ok(address.address().to_string())
}

pub fn get_transaction_detail<T: Config>(
    transaction_hash: String,
    state: &State,
    block: &ChainBlockResponse<T>,
    events: &Events<T>,
) -> Result<Option<Transaction>, Error> {
    let tx_hash = transaction_hash.trim_start_matches("0x");
    for (ex_index, extrinsic) in block.block.extrinsics.iter().enumerate() {
        let hex_val: String = convert_extrinsic_to_hash(extrinsic)
            .trim_start_matches("0x")
            .into();

        if hex_val.eq(tx_hash) {
            let mut vec_of_operations = vec![];
            let transaction_identifier = TransactionIdentifier { hash: hex_val };

            for (event_index, event_data) in events.iter().enumerate() {
                let event = event_data.map_err(|_| Error::EventDetailParse)?;
                if event.phase() == Phase::ApplyExtrinsic(ex_index as u32) {
                    let operation_identifier = OperationIdentifier {
                        index: event_index as i64,
                        network_index: None,
                    };

                    let event_metadata = event.event_metadata();
                    let mut vec_metadata = vec![];
                    for event in event_metadata.fields().iter() {
                        let name = event.name();
                        let type_name = event.type_name();
                        vec_metadata.push(json!({"name":name, "type": type_name}));
                    }
                    let op_metadata = Value::Array(vec_metadata);
                    let event_parsed_data = get_operation_data(event, &state.address_format)?;

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
                        metadata: Some(op_metadata),
                    };

                    vec_of_operations.push(operation)
                }
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

pub fn convert_extrinsic_to_hash(extrinsic: &ChainBlockExtrinsic) -> String {
    let hash = blake2_256(&extrinsic.0);
    format!("0x{}", hex::encode(hash))
}

pub struct TransactionOperationStatus {
    event_type: String,
    from: Option<String>,
    to: Option<String>,
    amount: Option<String>,
}

pub fn get_call_data<T, Call>(
    call: &Call,
    subxt: &OnlineClient<T>,
    account_nonce: T::Index,
) -> Result<PayloadData, Error>
where
    Call: TxPayload,
    T: Config,
    <T::ExtrinsicParams as ExtrinsicParams<T::Index, T::Hash>>::OtherParams: Default,
{
    encode_call_data(call, subxt, account_nonce, Default::default())
}

fn encode_call_data<T, Call>(
    call: &Call,
    subxt: &OnlineClient<T>,
    account_nonce: T::Index,
    other_params: <T::ExtrinsicParams as ExtrinsicParams<T::Index, T::Hash>>::OtherParams,
) -> Result<PayloadData, Error>
where
    Call: TxPayload,
    T: Config,
{
    let metadata = subxt.metadata();
    let bytes = call
        .encode_call_data(&metadata)
        .map_err(|_| Error::CouldNotSerialize)?;

    subxt
        .tx()
        .validate(call)
        .map_err(|_| Error::InvalidCallData)?;

    let encoded_call_data = Encoded(bytes);

    let additional_and_extra_params = {
        // Obtain spec version and transaction version from the runtime version of the client.
        let runtime = subxt.runtime_version();
        <T::ExtrinsicParams as ExtrinsicParams<T::Index, T::Hash>>::new(
            runtime.spec_version,
            runtime.transaction_version,
            account_nonce,
            subxt.genesis_hash(),
            other_params,
        )
    };

    let mut params_bytes = Vec::new();
    encoded_call_data.encode_to(&mut params_bytes);
    additional_and_extra_params.encode_extra_to(&mut params_bytes);
    additional_and_extra_params.encode_additional_to(&mut params_bytes);

    let payload = if params_bytes.len() > 256 {
        sp_core::blake2_256(&params_bytes).to_vec()
    } else {
        params_bytes
    };

    let mut params_vec = Vec::new();
    let mut call_data_vec = Vec::new();
    additional_and_extra_params.encode_extra_to(&mut params_vec);
    encoded_call_data.encode_to(&mut call_data_vec);

    let payload_data = PayloadData {
        payload,
        additional_params: params_vec,
        call_data: call_data_vec,
    };

    Ok(payload_data)
}

pub async fn get_unix_timestamp(client: &OnlineClient<GenericConfig>) -> Result<u64, Error> {
    let metadata = client.metadata();
    let storage_hash = metadata
        .storage_hash("Timestamp", "Now")
        .map_err(|_| Error::InvalidMetadata)?;

    let current_block_timestamp = StaticStorageAddress::<
        DecodeStaticType<u64>,
        address::Yes,
        address::Yes,
        (),
    >::new("Timestamp", "Now", vec![], storage_hash);

    let unix_timestamp_millis = client
        .storage()
        .fetch_or_default(&current_block_timestamp, None)
        .await
        .map_err(|_| Error::FailedTimestamp)?;

    Ok(unix_timestamp_millis)
}

pub async fn get_account_storage(
    client: &OnlineClient<GenericConfig>,
    account: &AccountId32,
) -> Result<AccountInfo<u32, AccountData>, Error> {
    let metadata = client.metadata();
    let storage_hash = metadata
        .storage_hash("System", "Account")
        .map_err(|_| Error::InvalidMetadata)?;
    let acc_key = StaticStorageAddress::<
        DecodeStaticType<AccountInfo<u32, AccountData>>,
        address::Yes,
        address::Yes,
        address::Yes,
    >::new(
        "System",
        "Account",
        vec![StorageMapKey::new(
            account.borrow(),
            StorageHasher::Blake2_128Concat,
        )],
        storage_hash,
    );
    let account_data = match client.storage().fetch_or_default(&acc_key, None).await {
        Ok(data) => data,
        Err(_) => return Err(Error::AccountNotFound),
    };
    Ok(account_data)
}

pub fn get_transfer_payload<T>(
    client: &OnlineClient<T>,
    dest: MultiAddress<AccountId32, u32>,
    value: u128,
) -> Result<StaticTxPayload<Transfer>, Error>
where
    T: Config,
{
    let metadata = client.metadata();
    let storage_hash = metadata
        .call_hash("Balances", "transfer")
        .map_err(|_| Error::InvalidMetadata)?;

    let call_data = StaticTxPayload::new(
        "Balances",
        "transfer",
        Transfer { dest, value },
        storage_hash,
    );
    Ok(call_data)
}

pub async fn faucet_substrate(
    api: &OnlineClient<GenericConfig>,
    address: &str,
    amount: u128,
) -> Result<H256, String> {
    let signer = PairSigner::<GenericConfig, _>::new(AccountKeyring::Alice.pair());

    let receiver_account: AccountId32 = address
        .parse()
        .map_err(|_| format!("{}", Error::InvalidAddress))?;
    let receiver_multiaddr: MultiAddress<AccountId32, u32> = MultiAddress::Id(receiver_account);

    let call_data = match get_transfer_payload(api, receiver_multiaddr, amount) {
        Ok(call_data) => call_data,
        Err(error) => return Err(format!("{}", error)),
    };

    let tx_progress = match api
        .tx()
        .sign_and_submit_then_watch_default(&call_data, &signer)
        .await
    {
        Ok(tx_progress) => tx_progress,
        Err(error) => {
            return Err(error.to_string());
        }
    };

    let status = match tx_progress.wait_for_finalized_success().await {
        Ok(status) => status,
        Err(error) => {
            return Err(error.to_string());
        }
    };

    Ok(status.extrinsic_hash())
}

pub async fn make_runtime_call_data(api: &OnlineClient<GenericConfig>, pallet_name: &str, call_name: &str) {

    let metadata = api.metadata();

}

pub async fn make_runtime_call(
    api: &OnlineClient<GenericConfig>,
    pallet_name: &str,
    call_name: &str,
    params: &Value,
) -> Result<Vec<u8>, Error> {
    println!("params {:?}", params);
    let mut params_vec = RpcParams::new();
    match params {
        Value::Array(e) => {
            for value in e {
                match params_vec.push(value) {
                    Ok(_) => {}
                    Err(_) => {}
                };
            }
        }
        _ => {}
    }
    let metadata = api.metadata();
    let pallet = metadata.pallet(pallet_name).unwrap();
    let pallet_index = pallet.index();
    let call_index = pallet.call_index(call_name).unwrap();
    let event_metadata = match metadata.event(pallet_index, call_index) {
        Ok(event_metadata) => event_metadata,
        Err(_) => return Err(Error::InvalidMetadata),
    };

    println!("event_metadaa {:?}", event_metadata);

    let types = metadata.types();
    let _tmp_breakpoint = "tmp";
    Ok(Vec::new())
}

pub fn get_runtime_error(error: SubxtError) -> String {
    if let SubxtError::Runtime(subxt::error::DispatchError::Module(msg)) = error {
        msg.error
    } else {
        format!("{}", Error::InvalidExtrinsic)
    }
}

pub fn string_to_err_response(err: String) -> tide::Result {
    let error = rosetta_types::Error {
        code: 500,
        message: err,
        description: None,
        retriable: false,
        details: None,
    };
    Ok(Response::builder(500)
        .body(Body::from_json(&error)?)
        .build())
}

#[derive(Serialize, Deserialize)]
pub struct UnsignedTransactionData {
    pub signer_address: String,
    pub additional_parmas: Vec<u8>,
    pub call_data: Vec<u8>,
}

pub struct PayloadData {
    pub payload: Vec<u8>,
    pub additional_params: Vec<u8>,
    pub call_data: Vec<u8>,
}

#[derive(Decode, Encode, Debug)]
pub struct Transfer {
    pub dest: MultiAddress<AccountId32, u32>,
    #[codec(compact)]
    pub value: u128,
}

#[derive(Decode, Encode, Debug)]
pub struct AccountInfo<Index, AccountData> {
    pub nonce: Index,
    pub consumers: Index,
    pub providers: Index,
    pub sufficients: Index,
    pub data: AccountData,
}

#[derive(Decode, Encode, Debug)]
pub struct AccountData {
    pub free: u128,
    pub reserved: u128,
    pub misc_frozen: u128,
    pub fee_frozen: u128,
}

#[derive(Decode, Encode, Debug)]
pub struct EventRecord<Event, Hash> {
    pub phase: Phase,
    pub event: Event,
    pub topics: Vec<Hash>,
}

#[derive(Debug)]
pub struct FilteredIndexData {
    pub ex_hash: String,
    pub event_details_data: EventDetailsData,
}

#[derive(Clone, Debug)]
pub struct EventDetailsData {
    pub op_index: usize,
    pub event_detail: EventDetails,
}
