use rosetta_client::Client;
use rosetta_types::{BlockTransaction, Operator, SearchTransactionsRequest};
use subxt::{OnlineClient, PolkadotConfig as CustomConfig};
use utils::{get_indexed_transactions, TxIndexerProps};

mod utils;

pub async fn indexer_search_transactions(
    request: SearchTransactionsRequest,
    client: &OnlineClient<CustomConfig>,
) -> Result<Vec<BlockTransaction>, String> {
    if let Some(Operator::Or) = request.operator {
        return Err("Invalid Operator".into());
    }

    let max_block = match request.max_block {
        Some(max_block) => max_block,
        None => {
            let block = client.rpc().block(None).await.unwrap();
            match block {
                Some(block) => block.block.header.number as i64,
                None => return Err("Invalid Block".into()),
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

    let server_client = match Client::new("http://127.0.0.1:8082") {
        Ok(client) => client,
        Err(_) => return Err("Invalid Rosetta Server".into()),
    };

    let tx_data =
        get_indexed_transactions(&server_client, request.network_identifier, req_props).await?;

    Ok(tx_data)
}
