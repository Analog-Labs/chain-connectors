use crate::State;
use anyhow::Result;
use parity_scale_codec::{Decode, Encode};
use rosetta_types::AccountIdentifier;
use rosetta_types::Amount;
use rosetta_types::{
    Operation, OperationIdentifier, PartialBlockIdentifier, SubAccountIdentifier, Transaction,
    TransactionIdentifier,
};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::borrow::Borrow;
use subxt::events::Phase;
use subxt::ext::sp_core;
use subxt::ext::sp_core::H256;
use subxt::ext::sp_runtime::generic::{Block as SPBlock, Header, SignedBlock};
use subxt::ext::sp_runtime::traits::BlakeTwo256;
use subxt::ext::sp_runtime::AccountId32;
use subxt::ext::sp_runtime::MultiAddress;
use subxt::ext::sp_runtime::OpaqueExtrinsic;
use subxt::metadata::DecodeStaticType;
use subxt::rpc::BlockNumber;
use subxt::storage::address;
use subxt::storage::address::StorageHasher;
use subxt::storage::address::StorageMapKey;
use subxt::storage::StaticStorageAddress;
use subxt::tx::AssetTip;
use subxt::tx::BaseExtrinsicParamsBuilder;
use subxt::tx::StaticTxPayload;
use subxt::tx::SubstrateExtrinsicParams;
use subxt::tx::{ExtrinsicParams, TxPayload};
use subxt::utils::Encoded;
use subxt::{OnlineClient, SubstrateConfig};
use tide::{Body, Response};

pub enum Error {
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
    StorageFetch,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
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
            Self::StorageFetch => write!(f, "Storage fetch error"),
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

pub fn get_block_transactions(
    state: &State,
    block: SignedBlock<SPBlock<Header<u32, BlakeTwo256>, OpaqueExtrinsic>>,
    events: &[EventRecord<RuntimeEvent, H256>],
) -> Result<Vec<Transaction>, Error> {
    let mut vec_of_extrinsics = vec![];

    let extrinsics = block.block.extrinsics;
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
            .collect::<Vec<&EventRecord<RuntimeEvent, H256>>>();

        for (event_index, event) in events_for_current_extrinsic.iter().enumerate() {
            let operation_identifier = OperationIdentifier {
                index: event_index as i64,
                network_index: None,
            };

            let data = format!("{:?}", event);
            println!("{}", data);

            let json_string =
                serde_json::to_string(&event.event).map_err(|_| Error::CouldNotSerialize)?;

            let json_event: Value =
                serde_json::from_str(&json_string).map_err(|_| Error::CouldNotSerialize)?;

            let event_parsed_data = get_operation_data(json_event.clone())?;

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

pub fn get_operation_data(data: Value) -> Result<TransactionOperationStatus, Error> {
    let root_object = data.as_object().ok_or(Error::OperationParse)?;

    let pallet_name = root_object.keys().next().ok_or(Error::OperationParse)?;

    let pallet_object = match root_object.get(pallet_name) {
        Some(pallet_obj) => pallet_obj.as_object().ok_or(Error::OperationParse)?,
        None => return Err(Error::OperationParse),
    };

    let event_name = pallet_object.keys().next().ok_or(Error::OperationParse)?;

    let event_object = match pallet_object.get(event_name) {
        Some(event_obj) => event_obj.as_object().ok_or(Error::OperationParse)?,
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
        Some(who) => who.as_str().map(|who_str| who_str.to_string()),
        None => match event_object.get("account") {
            Some(account) => account.as_str().map(|account_str| account_str.to_string()),
            None => match event_object.get("from") {
                Some(from) => from.as_str().map(|from_str| from_str.to_string()),
                None => None,
            },
        },
    };

    let to: Option<String> = match event_object.get("to") {
        Some(to) => to.as_str().map(|to_str| to_str.to_string()),
        None => None,
    };

    let transaction_operation_status = TransactionOperationStatus {
        event_type: call_type,
        amount,
        from: who,
        to,
    };

    Ok(transaction_operation_status)
}

pub fn get_transaction_detail(
    transaction_hash: String,
    state: &State,
    block: SignedBlock<SPBlock<Header<u32, BlakeTwo256>, OpaqueExtrinsic>>,
    events: &[EventRecord<RuntimeEvent, H256>],
) -> Result<Option<Transaction>, Error> {
    let tx_hash = transaction_hash.trim_start_matches("0x");
    let extrinsics = block.block.extrinsics;
    for (ex_index, extrinsic) in extrinsics.iter().enumerate() {
        let encoded_item: &[u8] = &extrinsic.encode();
        let hex_val = hex::encode(encoded_item);

        if hex_val.eq(&tx_hash) {
            let mut vec_of_operations = vec![];
            let transaction_identifier = TransactionIdentifier { hash: hex_val };

            let events_for_current_extrinsic = events
                .iter()
                .filter(|e| e.phase == Phase::ApplyExtrinsic(ex_index as u32))
                .collect::<Vec<&EventRecord<RuntimeEvent, H256>>>();

            for (event_index, event) in events_for_current_extrinsic.iter().enumerate() {
                let operation_identifier = OperationIdentifier {
                    index: event_index as i64,
                    network_index: None,
                };
                let json_string =
                    serde_json::to_string(&event.event).map_err(|_| Error::CouldNotSerialize)?;
                let json_event: Value =
                    serde_json::from_str(&json_string).map_err(|_| Error::CouldNotSerialize)?;

                let event_parsed_data = get_operation_data(json_event.clone())?;

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

pub struct TransactionOperationStatus {
    event_type: String,
    from: Option<String>,
    to: Option<String>,
    amount: Option<String>,
}

pub fn encode_call_data<Call>(
    call: &Call,
    subxt: &OnlineClient<SubstrateConfig>,
    account_nonce: u32,
    other_params: BaseExtrinsicParamsBuilder<SubstrateConfig, AssetTip>,
) -> Result<PayloadData, Error>
where
    Call: TxPayload,
{
    let metadata = subxt.metadata();
    let mut bytes = Vec::new();
    call.encode_call_data(&metadata, &mut bytes)
        .map_err(|_| Error::CouldNotSerialize)?;

    subxt
        .tx()
        .validate(call)
        .map_err(|_| Error::InvalidCallData)?;

    let encoded_call_data = Encoded(bytes);

    let additional_and_extra_params = {
        // Obtain spec version and transaction version from the runtime version of the client.
        let runtime = subxt.runtime_version();
        SubstrateExtrinsicParams::<SubstrateConfig>::new(
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

pub async fn get_unix_timestamp(client: &OnlineClient<SubstrateConfig>) -> Result<u64, Error> {
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
        .map_err(|_| Error::StorageFetch)?;

    Ok(unix_timestamp_millis)
}

pub async fn get_account_storage(
    client: &OnlineClient<SubstrateConfig>,
    account: &AccountId32,
) -> Result<AccountInfo<u32, AccountData<u128>>, Error> {
    let metadata = client.metadata();
    let storage_hash = metadata
        .storage_hash("System", "Account")
        .map_err(|_| Error::InvalidMetadata)?;
    let acc_key = StaticStorageAddress::<
        DecodeStaticType<AccountInfo<u32, AccountData<u128>>>,
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
    let account_data = match client.storage().fetch(&acc_key, None).await {
        Ok(data) => data.ok_or(Error::StorageFetch)?,
        Err(_) => return Err(Error::StorageFetch),
    };
    Ok(account_data)
}

pub async fn get_block_events(
    client: &OnlineClient<SubstrateConfig>,
    block: H256,
) -> Result<Vec<EventRecord<RuntimeEvent, H256>>, Error> {
    let metadata = client.metadata();
    let storage_hash = metadata
        .storage_hash("System", "Events")
        .map_err(|_| Error::InvalidMetadata)?;

    let st_key = StaticStorageAddress::<
        DecodeStaticType<Vec<EventRecord<RuntimeEvent, H256>>>,
        address::Yes,
        address::Yes,
        (),
    >::new("System", "Events", vec![], storage_hash);

    let data = client
        .storage()
        .fetch_or_default(&st_key, Some(block))
        .await
        .map_err(|_| Error::StorageFetch)?;

    Ok(data)
}

pub fn get_transfer_payload(
    client: &OnlineClient<SubstrateConfig>,
    dest: MultiAddress<AccountId32, u32>,
    value: u128,
) -> Result<StaticTxPayload<Transfer>, Error> {
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
