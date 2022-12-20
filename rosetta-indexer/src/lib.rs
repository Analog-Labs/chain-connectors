use anyhow::Result;
use rosetta_client::{Chain, Client};
use rosetta_types::{
    NetworkIdentifier, NetworkRequest, Operator, SearchTransactionsRequest,
    SearchTransactionsResponse,
};
use surf::Body;
use tide::{Request, Response};
use utils::{get_indexed_transactions, string_to_err_response, TxIndexerProps};

pub mod args;
mod utils;

#[derive(Clone)]
pub struct State {
    network: NetworkIdentifier,
    rosetta_server: Client,
}

impl State {
    pub fn new(chain: Chain, server_url: Option<String>) -> Self {
        let url = if let Some(url) = server_url {
            url
        } else {
            chain.url().to_string()
        };
        State {
            network: chain.config().network,
            rosetta_server: Client::new(&url).unwrap(),
        }
    }
}

pub async fn server(config: Chain, url: Option<String>) -> Result<tide::Server<State>> {
    let state = State::new(config, url);
    let mut app = tide::with_state(state);
    app.at("/search/transactions")
        .post(indexer_search_transactions);
    Ok(app)
}

pub async fn indexer_search_transactions(mut req: Request<State>) -> tide::Result {
    let request: SearchTransactionsRequest = req.body_json().await?;

    if request.network_identifier != req.state().network {
        return string_to_err_response("Unsupported Network".into());
    }

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

    if let Some(Operator::Or) = request.operator {
        return string_to_err_response("Invalid Operator".into());
    }

    let max_block = match request.max_block {
        Some(max_block) => max_block,
        None => {
            let request = NetworkRequest {
                network_identifier: request.network_identifier.clone(),
                metadata: None,
            };
            let block = req.state().rosetta_server.network_status(&request).await;
            match block {
                Ok(block) => block.current_block_identifier.index as i64,
                Err(e) => {
                    return string_to_err_response(e.to_string());
                }
            }
        }
    };

    let req_props = TxIndexerProps {
        max_block: max_block + 1,
        transaction_identifier: request.transaction_identifier,
        account_identifier: request.account_identifier,
        status: request.status,
        operation_type: request.r#type,
        address: request.address,
        success: request.success,
        currency: request.currency,
    };

    let filtered_ex = match get_indexed_transactions(
        &req.state().rosetta_server,
        request.network_identifier,
        req_props,
    )
    .await
    {
        Ok(filtered_ex) => filtered_ex,
        Err(e) => {
            return string_to_err_response(e);
        }
    };

    let total_count = filtered_ex.len() as i64;

    if offset > 0 && offset >= total_count {
        return string_to_err_response("Invalid Offset".into());
    }

    let idx_end = if offset + limit > total_count {
        total_count
    } else {
        offset + limit
    };

    let limited_tx = if total_count <= limit {
        filtered_ex
    } else {
        filtered_ex[offset as usize..idx_end as usize].to_vec()
    };

    let next_offset = if idx_end == total_count {
        None
    } else {
        Some(idx_end)
    };

    let response = SearchTransactionsResponse {
        transactions: limited_tx,
        total_count,
        next_offset,
    };

    Ok(Response::builder(200)
        .body(Body::from_json(&response)?)
        .build())
}
