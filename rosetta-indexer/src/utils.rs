use rosetta_client::Client;
use rosetta_types::{
    AccountIdentifier, BlockIdentifier, BlockRequest, BlockResponse, BlockTransaction,
    NetworkIdentifier, PartialBlockIdentifier, Transaction, TransactionIdentifier,
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
            if let Some(ref tx_id) = req.transaction_identifier {
                if !tx_id.hash.eq(&tx.transaction_identifier.hash) {
                    continue;
                }
            }

            let last_event = match tx.operations.last() {
                Some(event) => event,
                None => continue,
            };

            if let Some(success_status) = req.success {
                if success_status {
                    if !last_event.r#type.ends_with(".ExtrinsicSuccess") {
                        continue;
                    }
                } else if !last_event.r#type.ends_with(".ExtrinsicFailed") {
                    continue;
                }
            }

            for op in tx.operations {
                if let Some(operation_type) = req.operation_type.as_ref() {
                    if !op.r#type.ends_with(operation_type) {
                        continue;
                    }
                }

                if let Some(acc_identifier) = req.account_identifier.as_ref() {
                    let filter_address = acc_identifier.address.trim_start_matches("0x");
                    if let Some(op_identifier) = op.account.clone() {
                        let address_match = if filter_address.eq(&op_identifier.address) {
                            true
                        } else {
                            match op_identifier.sub_account.as_ref() {
                                Some(sub_address) => filter_address.eq(&sub_address.address),
                                None => false,
                            }
                        };

                        if !address_match {
                            continue;
                        }
                    } else {
                        continue;
                    }
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
