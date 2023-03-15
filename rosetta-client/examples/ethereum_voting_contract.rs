use rosetta_client::{
    create_wallet,
    types::{CallRequest, NetworkIdentifier},
    Client, Wallet,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let contract_address = "0xc3c2640cfda6cafabb33da7ac1609a0fb4c53afe";
    rosetta_wallet_methods(contract_address).await;
    rosetta_client_methods(contract_address).await;
}

async fn rosetta_wallet_methods(contract_address: &str) {
    let wallet = create_wallet(
        Some("ethereum".to_owned()),
        Some("dev".to_owned()),
        Some("http://127.0.0.1:8081".to_owned()),
        None,
    )
    .await
    .unwrap();

    method_call(&wallet, contract_address).await;
}

async fn method_call(wallet: &Wallet, contract_address: &str) {
    println!("method call ==================");
    let function_signature = "function vote_yes()";
    let method_params = format!("{}-{}", contract_address, function_signature);
    println!("{:?}", wallet.method_call(&method_params, json!([])).await);
}

async fn rosetta_client_methods(contract_address: &str) {
    let server_url = "http://127.0.0.1:8081";
    let client = Client::new(server_url).unwrap();
    let network_identifier = NetworkIdentifier {
        blockchain: "ethereum".to_owned(),
        network: "dev".to_owned(),
        sub_network_identifier: None,
    };

    contract_call(&client, network_identifier.clone(), contract_address).await;
    storage_yes_votes(&client, network_identifier.clone(), contract_address).await;
    storage_no_votes(&client, network_identifier.clone(), contract_address).await;
}

async fn contract_call(
    client: &Client,
    network_identifier: NetworkIdentifier,
    contract_address: &str,
) {
    let method_signature = "function get_votes_stats() external view returns (uint, uint)";
    let call_type = "call";

    let method = format!("{}-{}-{}", contract_address, method_signature, call_type);
    let request = CallRequest {
        network_identifier,
        method,
        parameters: json!({}),
    };
    let response = client.call(&request).await;
    println!("contract call response {:#?}\n", response);
}

async fn storage_yes_votes(
    client: &Client,
    network_identifier: NetworkIdentifier,
    contract_address: &str,
) {
    let method_signature = "0000000000000000000000000000000000000000000000000000000000000000";
    let call_type = "storage";

    let method = format!("{}-{}-{}", contract_address, method_signature, call_type);
    let request = CallRequest {
        network_identifier,
        method,
        parameters: json!({}),
    };
    let response = client.call(&request).await;
    println!("storage_yes_votes response {:#?}", response);
}

async fn storage_no_votes(
    client: &Client,
    network_identifier: NetworkIdentifier,
    contract_address: &str,
) {
    let method_signature = "0000000000000000000000000000000000000000000000000000000000000001";
    let call_type = "storage";

    let method = format!("{}-{}-{}", contract_address, method_signature, call_type);
    let request = CallRequest {
        network_identifier,
        method,
        parameters: json!({}),
    };
    let response = client.call(&request).await;
    println!("storage_no_votes response {:#?}", response);
}
