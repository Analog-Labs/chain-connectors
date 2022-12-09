use anyhow::Result;
use rosetta_client::Client;
use rosetta_types::{
    BlockTransaction, NetworkIdentifier, NetworkRequest, Operator, SearchTransactionsRequest,
};
use surf::Body;
use tide::{Request, Response};
use utils::{get_indexed_transactions, TxIndexerProps};

mod utils;
pub mod args;

#[derive(Clone)]
pub struct State {
    network_identifier: NetworkIdentifier,
}

impl State {
    pub fn new(blockchain: String, network: String) -> Self {
        State {
            network_identifier: NetworkIdentifier {
                blockchain,
                network,
                sub_network_identifier: None,
            },
        }
    }
}

pub async fn server() -> Result<tide::Server<State>> {
    let state = State::new("Polkadot".into(), "Dev".into());
    let mut app = tide::with_state(state);
    app.at("/search/transactions")
        .post(indexer_search_transactions);
    Ok(app)
}

pub async fn indexer_search_transactions(
    mut req: Request<State>, // request: SearchTransactionsRequest,
) -> tide::Result {
    // if let Some(Operator::Or) = request.operator {
    //     return Err("Invalid Operator".into());
    // }

    // let server_client = match Client::new("http://127.0.0.1:8082") {
    //     Ok(client) => client,
    //     Err(_) => return Err("Invalid Rosetta Server".into()),
    // };

    // let max_block = match request.max_block {
    //     Some(max_block) => max_block,
    //     None => {
    //         let request = NetworkRequest {
    //             network_identifier: request.network_identifier.clone(),
    //             metadata: None,
    //         };
    //         let block = server_client.network_status(&request).await;
    //         match block {
    //             Ok(block) => block.current_block_identifier.index as i64,
    //             Err(_) => return Err("Invalid Block".into()),
    //         }
    //     }
    // };

    // let req_props = TxIndexerProps {
    //     max_block,
    //     transaction_identifier: request.transaction_identifier,
    //     account_identifier: request.account_identifier,
    //     status: request.status,
    //     operation_type: request.r#type,
    //     address: request.address,
    //     success: request.success,
    // };

    // let tx_data =
    //     get_indexed_transactions(&server_client, request.network_identifier, req_props).await?;

    // Ok(tx_data)

    Ok(Response::builder(200).body(Body::from_json(&"")?).build())
}
