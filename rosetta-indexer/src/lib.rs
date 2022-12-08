use rosetta_client::Client;
use rosetta_types::{BlockTransaction, NetworkRequest, Operator, SearchTransactionsRequest};
use utils::{get_indexed_transactions, TxIndexerProps};

mod utils;

pub async fn indexer_search_transactions(
    request: SearchTransactionsRequest,
) -> Result<Vec<BlockTransaction>, String> {
    if let Some(Operator::Or) = request.operator {
        return Err("Invalid Operator".into());
    }

    let server_client = match Client::new("http://127.0.0.1:8082") {
        Ok(client) => client,
        Err(_) => return Err("Invalid Rosetta Server".into()),
    };

    let max_block = match request.max_block {
        Some(max_block) => max_block,
        None => {
            let request = NetworkRequest {
                network_identifier: request.network_identifier.clone(),
                metadata: None,
            };
            let block = server_client.network_status(&request).await;
            match block {
                Ok(block) => block.current_block_identifier.index as i64,
                Err(_) => return Err("Invalid Block".into()),
            }
        }
    };

    let req_props = TxIndexerProps {
        max_block,
        transaction_identifier: request.transaction_identifier,
        account_identifier: request.account_identifier,
        status: request.status,
        operation_type: request.r#type,
        address: request.address,
        success: request.success,
    };

    let tx_data =
        get_indexed_transactions(&server_client, request.network_identifier, req_props).await?;

    Ok(tx_data)
}
