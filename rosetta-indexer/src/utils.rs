use rosetta_client::Client;
use rosetta_types::{
    AccountIdentifier, Amount, BlockIdentifier, BlockRequest, BlockResponse, BlockTransaction,
    Currency, NetworkIdentifier, PartialBlockIdentifier, Transaction, TransactionIdentifier,
};
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

            if !match_tx_id(&req.transaction_identifier, &tx.transaction_identifier.hash) {
                continue;
            }

            let last_event = match tx.operations.last() {
                Some(event) => event,
                None => continue,
            };

            if !match_success(&last_event.r#type, req.success) {
                continue;
            }

            for op in tx.operations {
                if !match_operation_type(&req.operation_type, &op.r#type) {
                    continue;
                }

                if !match_address(&req.account_identifier, &op.account) {
                    continue;
                };

                if !match_currency(&req.currency, &op.amount){
                    continue;
                }

                vec_of_operations.push(op);
            }

            if !vec_of_operations.is_empty() {
                let transaction = Transaction {
                    transaction_identifier: tx.transaction_identifier,
                    operations: vec_of_operations,
                    related_transactions: None,
                    metadata: None,
                };
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

pub fn match_success(tx_success: &String, received_success: Option<bool>) -> bool {
    if let Some(success) = received_success {
        if success {
            tx_success.to_lowercase().contains("success")
        } else {
            tx_success.to_lowercase().contains("fail")
        }
    } else {
        true
    }
}

pub fn match_operation_type(op_type: &Option<String>, received_type: &String) -> bool {
    if let Some(operation_type) = op_type.as_ref() {
        received_type
            .to_lowercase()
            .ends_with(&operation_type.to_lowercase())
    } else {
        true
    }
}

pub fn match_address(
    received_acc_identifier: &Option<AccountIdentifier>,
    op_acc_identifier: &Option<AccountIdentifier>,
) -> bool {
    if let Some(acc_identifier) = received_acc_identifier.as_ref() {
        let filter_address = acc_identifier.address.trim_start_matches("0x");
        if let Some(op_identifier) = op_acc_identifier.clone() {
            let address_match = if filter_address.eq(&op_identifier.address) {
                true
            } else {
                match op_identifier.sub_account.as_ref() {
                    Some(sub_address) => filter_address.eq(&sub_address.address),
                    None => false,
                }
            };
            !address_match
        } else {
            return false;
        }
    } else {
        return true;
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
