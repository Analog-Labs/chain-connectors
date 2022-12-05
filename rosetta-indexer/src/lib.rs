use rosetta_client::Client;
use rosetta_types::{Currency, Operator, SearchTransactionsRequest, network_identifier};
use subxt::{
    ext::sp_core::crypto::Ss58AddressFormat, Config, OnlineClient, PolkadotConfig as CustomConfig,
};
use tide::{Body, Response};
use utils::{TxIndexerProps, get_indexed_transactions};

mod utils;

pub async fn indexer_search_transactions(
    request: SearchTransactionsRequest,
    client: &OnlineClient<CustomConfig>,
    currency: &Currency,
    address_format: &Ss58AddressFormat,
) -> tide::Result {
    if let Some(Operator::Or) = request.operator {
        // return Error::NotSupported.to_response();
    }

    let max_block = match request.max_block {
        Some(max_block) => max_block,
        None => {
            let block = client.rpc().block(None).await?;
            match block {
                Some(block) => block.block.header.number as i64,
                None => 0,
                // None => return Err(Error::BlockNotFound),
            }
        }
    };

    let offset = request.offset.unwrap_or(0);

    let limit = match request.limit {
        Some(limit) => {
            if limit > 1000 {
                1000
            } else {
                limit
            }
        }
        None => 100,
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

    let server_client = Client::new("http://rosetta.analog.one:8082".into())?;

    get_indexed_transactions(&server_client, client, request.network_identifier, req_props).await.unwrap();

    Ok(Response::builder(200).body(Body::from_json(&"")?).build())
}
