use std::collections::HashMap;

use rosetta_client::Client;
use rosetta_types::{
    AccountIdentifier, Amount, BlockIdentifier, BlockRequest, BlockResponse, BlockTransaction,
    Currency, NetworkIdentifier, Operation, PartialBlockIdentifier, Transaction,
    TransactionIdentifier,
};
use serde_json::json;
use surf::Body;
use tide::Response;

pub struct TxIndexerProps {
    pub max_block: i64,
    pub transaction_identifier: Option<TransactionIdentifier>,
    pub account_identifier: Option<AccountIdentifier>,
    pub status: Option<String>,
    pub operation_type: Option<String>,
    pub address: Option<String>,
    pub success: Option<bool>,
    pub currency: Option<Currency>,
}

pub async fn get_indexed_transactions(
    server_client: &Client,
    network_identifier: NetworkIdentifier,
    req: TxIndexerProps,
) -> Result<Vec<BlockTransaction>, String> {
    let mut filtered_ex = vec![];
    for value in (0..req.max_block).rev() {
        let block_request = BlockRequest {
            network_identifier: network_identifier.clone(),
            block_identifier: PartialBlockIdentifier {
                index: Some(value as u64),
                hash: None,
            },
        };
        if let Ok(data) = server_client.block(&block_request).await {
            let filtered_data = filter_tx(&data, &req)?;

            let block_hash = data.block.map(|e| e.block_identifier.hash);

            for tx in filtered_data {
                let block_transaction = BlockTransaction {
                    block_identifier: BlockIdentifier {
                        index: value as u64,
                        hash: block_hash.clone().unwrap_or_else(|| "".into()),
                    },
                    transaction: tx,
                };
                filtered_ex.push(block_transaction);
            }
        } else {
            break;
        };
    }

    Ok(filtered_ex)
}

pub fn filter_tx(
    block_response: &BlockResponse,
    req: &TxIndexerProps,
) -> Result<Vec<Transaction>, String> {
    let mut vec_of_extrinsics = vec![];
    if let Some(block) = block_response.block.clone() {
        for tx in block.transactions {
            let mut vec_of_operations = vec![];
            let basic_tx_details = get_basic_details_from_event(&tx);

            if !match_tx_id(&req.transaction_identifier, &tx.transaction_identifier.hash) {
                continue;
            }

            let last_event = match tx.operations.last() {
                Some(event) => event,
                None => continue,
            };

            let tx_failed = last_event.r#type.to_lowercase().contains("fail");
            if !match_success(tx_failed, req.success) {
                continue;
            }

            let get_transfer_operations = find_transfer_operation(tx.operations.clone());
            let is_utxo_transfer = find_utxo_transfer_operation(tx.operations.clone());
            for op in tx.operations {
                if !match_operation_type(&req.operation_type, &op.r#type) {
                    if let Some(op_type) = &req.operation_type {
                        if op_type.to_lowercase().contains("transfer")
                            && !get_transfer_operations.contains(&op)
                            && !is_utxo_transfer
                        {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                if !match_acc_identifier(&req.account_identifier, &op.account) {
                    continue;
                };

                if !match_address(&req.address, &op.account) {
                    continue;
                };

                if !match_currency(&req.currency, &op.amount) {
                    continue;
                }

                vec_of_operations.push(op);
            }

            if !vec_of_operations.is_empty() {
                let mut transaction = Transaction {
                    transaction_identifier: tx.transaction_identifier,
                    operations: vec_of_operations,
                    related_transactions: None,
                    metadata: None,
                };

                if !basic_tx_details.sender.is_empty()
                    && !basic_tx_details.receiver.is_empty()
                    && !basic_tx_details.amount.is_empty()
                {
                    let metadata = json!({"from": basic_tx_details.sender, "to": basic_tx_details.receiver, "amount": basic_tx_details.amount,"Success": !tx_failed});
                    transaction.metadata = Some(metadata);
                }

                vec_of_extrinsics.push(transaction);
            }
        }
    }
    Ok(vec_of_extrinsics)
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

pub fn match_tx_id(tx_identifier: &Option<TransactionIdentifier>, received_tx: &String) -> bool {
    if let Some(ref tx_id) = tx_identifier {
        tx_id.hash.eq(received_tx)
    } else {
        true
    }
}

pub fn match_success(tx_success_status: bool, received_success: Option<bool>) -> bool {
    if let Some(success) = received_success {
        if success {
            !tx_success_status
        } else {
            tx_success_status
        }
    } else {
        true
    }
}

pub fn match_operation_type(op_type: &Option<String>, received_type: &str) -> bool {
    if let Some(operation_type) = op_type.as_ref() {
        received_type
            .to_lowercase()
            .ends_with(&operation_type.to_lowercase())
    } else {
        true
    }
}

pub fn match_acc_identifier(
    received_acc_identifier: &Option<AccountIdentifier>,
    op_acc_identifier: &Option<AccountIdentifier>,
) -> bool {
    if let Some(acc_identifier) = received_acc_identifier.as_ref() {
        let filter_address = acc_identifier.address.trim_start_matches("0x");
        if let Some(op_identifier) = op_acc_identifier.clone() {
            let address_match =
                if filter_address.eq(&op_identifier.address.trim_start_matches("0x").to_string()) {
                    if acc_identifier.sub_account.is_some() {
                        acc_identifier.sub_account == op_identifier.sub_account
                    } else {
                        true
                    }
                } else {
                    match op_identifier.sub_account.as_ref() {
                        Some(sub_address) => filter_address
                            .eq(&sub_address.address.trim_start_matches("0x").to_string()),
                        None => false,
                    }
                };
            address_match
        } else {
            false
        }
    } else {
        true
    }
}

pub fn match_address(
    received_address: &Option<String>,
    op_acc_identifier: &Option<AccountIdentifier>,
) -> bool {
    if let Some(address) = received_address {
        let address_without_prefix = address.trim_start_matches("0x");
        if let Some(op_identifier) = op_acc_identifier.clone() {
            let found_address = if address_without_prefix
                .eq(&op_identifier.address.trim_start_matches("0x").to_string())
            {
                true
            } else {
                match op_identifier.sub_account.as_ref() {
                    Some(sub_address) => address_without_prefix
                        .eq(&sub_address.address.trim_start_matches("0x").to_string()),
                    None => false,
                }
            };

            found_address
        } else {
            false
        }
    } else {
        true
    }
}

pub fn match_currency(received_curreny: &Option<Currency>, op_amount: &Option<Amount>) -> bool {
    if let Some(currency) = received_curreny {
        if let Some(amount) = op_amount {
            currency.symbol.eq(&amount.currency.symbol)
                && currency.decimals == amount.currency.decimals
        } else {
            true
        }
    } else {
        true
    }
}

pub fn find_transfer_operation(operations: Vec<Operation>) -> Vec<Operation> {
    let mut op_hashmap: HashMap<i128, Vec<Operation>> = HashMap::new();
    for op in operations {
        if let Some(amount) = op.amount.clone() {
            if !op.r#type.to_lowercase().contains("fee") {
                let amount: i128 = match amount.value.parse() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                op_hashmap.entry(amount.abs()).or_default().push(op);
            }
        }
    }

    let transfer_operations = op_hashmap
        .iter()
        .filter(|&(_, v)| v.len() > 1)
        .map(|(_, v)| v.clone())
        .collect::<Vec<Vec<Operation>>>()
        .into_iter()
        .flatten()
        .collect::<Vec<Operation>>();

    transfer_operations
}

fn find_utxo_transfer_operation(operations: Vec<Operation>) -> bool {
    let mut input_amount: i128 = 0;
    let mut output_amount: i128 = 0;

    for op in operations {
        if op.r#type.to_lowercase().contains("input") {
            if let Some(amount) = op.amount {
                match amount.value.parse::<i128>() {
                    Ok(e) => {
                        input_amount += e.abs();
                    }
                    Err(_) => {
                        return false;
                    }
                };
            }
        } else if op.r#type.to_lowercase().contains("output") {
            if let Some(amount) = op.amount {
                match amount.value.parse::<i128>() {
                    Ok(e) => {
                        output_amount += e.abs();
                    }
                    Err(_) => {
                        return false;
                    }
                };
            }
        }
    }

    (input_amount > 0 && output_amount > 0) && (input_amount - output_amount).abs() < 1000
}

fn get_basic_details_from_event(tx: &Transaction) -> TransferStruct {
    let mut sender: String = "".into();
    let mut receiver: String = "".into();
    let mut amount = "".into();

    let transfer_operation = tx
        .operations
        .iter()
        .find(|op| op.r#type.to_lowercase().contains("transfer"));
    if let Some(op) = transfer_operation {
        if let Some(value) = op.amount.clone() {
            amount = value.value;
        }
        if let Some(tx_sender) = op.account.clone() {
            sender = tx_sender.address;
            receiver = tx_sender.sub_account.unwrap_or_default().address;
        }
    } else {
        let transfer_operation: Vec<&Operation> = tx
            .operations
            .iter()
            .filter(|op| op.r#type.to_lowercase().contains("call"))
            .collect::<_>();

        if transfer_operation.len() == 2 {
            for tx in transfer_operation.iter() {
                if let Some(value) = tx.amount.clone() {
                    if !value.value.contains('-') {
                        amount = value.value;
                        receiver = tx.account.clone().unwrap_or_default().address;
                    } else {
                        sender = tx.account.clone().unwrap_or_default().address;
                    }
                }
            }
        } else {
            let transfer_operation = tx
                .operations
                .iter()
                .find(|op| op.r#type.to_lowercase().contains("input"));
            if let Some(op) = transfer_operation {
                if let Some(tx_sender) = op.account.clone() {
                    sender = tx_sender.address;
                    let transfer_operation = tx
                        .operations
                        .iter()
                        .filter(|op| op.r#type.to_lowercase().contains("output"))
                        .collect::<Vec<&Operation>>();
                    if transfer_operation.len() == 2 {
                        for tx in transfer_operation {
                            let temp_receiver = tx.account.clone().unwrap_or_default().address;
                            if temp_receiver != sender {
                                receiver = temp_receiver;
                                amount = tx.amount.clone().unwrap_or_default().value;
                            }
                        }
                    }
                }
            }
        }
    }
    TransferStruct {
        sender,
        receiver,
        amount,
    }
}

pub struct TransferStruct {
    pub sender: String,
    pub receiver: String,
    pub amount: String,
}
