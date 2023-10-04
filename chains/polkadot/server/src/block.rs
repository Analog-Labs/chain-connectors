use anyhow::{Context, Result};
use rosetta_core::{
    crypto::address::Address,
    types::{
        AccountIdentifier, Amount, Operation, OperationIdentifier, Transaction,
        TransactionIdentifier,
    },
    BlockchainConfig,
};
use serde_json::{json, Value};
use subxt::{
    blocks::{ExtrinsicDetails, ExtrinsicEvents},
    config::Hasher,
    events::EventDetails,
    ext::scale_value::{scale::TypeId, Composite, Primitive, ValueDef},
    utils::H256,
    Config, OnlineClient,
};

pub fn get_transaction_identifier<T: Config<Hash = H256>>(
    extrinsic: &ExtrinsicDetails<T, OnlineClient<T>>,
) -> TransactionIdentifier {
    TransactionIdentifier { hash: hex::encode(T::Hasher::hash_of(&extrinsic.bytes())) }
}

pub fn get_transaction<T: Config<Hash = H256> + Send>(
    config: &BlockchainConfig,
    transaction_identifier: TransactionIdentifier,
    events: &ExtrinsicEvents<T>,
) -> Result<Transaction> {
    // let transaction_identifier = TransactionIdentifier {
    //     hash: hex::encode(T::Hasher::hash_of(&extrinsic.bytes())),
    // };
    // let events = extrinsic.events().await?;
    let mut operations = vec![];
    for (event_index, event_data) in events.iter().enumerate() {
        let event = event_data?;
        let event_parsed_data = get_operation_data(config, &event)?;

        let mut fields = vec![];
        for field in &event.event_metadata().variant.fields {
            fields.push(json!({"name": field.name, "type": field.type_name}));
        }
        let op_metadata = Value::Array(fields);

        let op_from: Option<AccountIdentifier> = event_parsed_data
            .from
            .map(|address| AccountIdentifier { address, sub_account: None, metadata: None });

        let op_neg_amount: Option<Amount> = event_parsed_data.amount.as_ref().map(|amount| {
            Amount { value: format!("-{amount}"), currency: config.currency(), metadata: None }
        });

        let operation = Operation {
            operation_identifier: OperationIdentifier {
                index: i64::try_from(event_index).context("event_index overflow")?,
                network_index: None,
            },
            related_operations: None,
            r#type: event_parsed_data.event_type.clone(),
            status: None,
            account: op_from,
            amount: op_neg_amount,
            coin_change: None,
            metadata: Some(op_metadata.clone()),
        };
        operations.push(operation);

        if let (Some(to), Some(amount)) = (event_parsed_data.to, event_parsed_data.amount) {
            operations.push(Operation {
                operation_identifier: OperationIdentifier {
                    index: i64::try_from(event_index).context("event_index overflow")?,
                    network_index: None,
                },
                related_operations: None,
                r#type: event_parsed_data.event_type,
                status: None,
                account: Some(AccountIdentifier { address: to, sub_account: None, metadata: None }),
                amount: Some(Amount { value: amount, currency: config.currency(), metadata: None }),
                coin_change: None,
                metadata: Some(op_metadata),
            });
        }
    }
    Ok(Transaction {
        transaction_identifier,
        operations,
        related_transactions: None,
        metadata: None,
    })
}

fn get_operation_data<T: Config<Hash = H256>>(
    config: &BlockchainConfig,
    event: &EventDetails<T>,
) -> Result<TransactionOperationStatus> {
    let pallet_name = event.pallet_name();
    let event_name = event.variant_name();

    let call_type = format!("{pallet_name}.{event_name}");

    let event_fields = event.field_values()?;
    let parsed_data = match event_fields {
        Composite::Named(value) => {
            let mut from_data =
                value.iter().filter(|(k, _)| k == "from" || k == "who" || k == "account");

            let sender_address: Option<String> = if let Some(data) = from_data.next() {
                let address = generate_address(config, &data.1.value)?;
                Some(address)
            } else {
                None
            };

            let amount: Option<String> = if let Some(value) =
                value.iter().find(|(k, _)| k == "amount" || k == "actual_fee")
            {
                match &value.1.value {
                    ValueDef::Primitive(Primitive::U128(amount)) => Some(amount.to_string()),
                    _ => {
                        anyhow::bail!("invalid operation");
                    },
                }
            } else {
                None
            };

            let to_address: Option<String> =
                if let Some(data) = value.iter().find(|(k, _)| k == "to") {
                    let address = generate_address(config, &data.1.value)?;
                    Some(address)
                } else {
                    None
                };

            (sender_address, amount, to_address)
        },
        Composite::Unnamed(_) => {
            anyhow::bail!("invalid operation");
        },
    };

    Ok(TransactionOperationStatus {
        event_type: call_type,
        from: parsed_data.0,
        amount: parsed_data.1,
        to: parsed_data.2,
    })
}

struct TransactionOperationStatus {
    event_type: String,
    from: Option<String>,
    to: Option<String>,
    amount: Option<String>,
}

fn generate_address(config: &BlockchainConfig, val: &ValueDef<TypeId>) -> Result<String> {
    let mut addr_array = vec![];
    match val {
        ValueDef::Composite(Composite::Unnamed(unamed_data)) => {
            for value_data in unamed_data {
                match &value_data.value {
                    ValueDef::Composite(data) => {
                        for data in data.values() {
                            match data.value {
                                ValueDef::Primitive(Primitive::U128(val)) => {
                                    let Ok(val) = u8::try_from(val) else {
                                        tracing::error!("overflow: {val} > 255");
                                        anyhow::bail!("overflow: {val} > 255");
                                    };
                                    addr_array.push(val);
                                },
                                _ => anyhow::bail!("invalid operation"),
                            }
                        }
                    },
                    _ => anyhow::bail!("invalid operation"),
                }
            }
        },
        _ => anyhow::bail!("invalid operation"),
    }

    let address = Address::from_public_key_bytes(config.address_format, &addr_array);
    Ok(address.address().to_string())
}
