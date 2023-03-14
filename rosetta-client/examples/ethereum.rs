use rosetta_client::{
    create_wallet,
    types::{
        AccountIdentifier, BlockIdentifier, BlockRequest, BlockTransactionRequest, CallRequest,
        NetworkIdentifier, PartialBlockIdentifier, TransactionIdentifier,
    },
    Client, Wallet,
};
use serde_json::json;
use tokio;

#[tokio::main]
async fn main() {
    rosetta_wallet_methods().await;
    rosetta_client_methods().await;
}

/// Wallet methods
/// 1. account
/// 2. network_status
/// 3. faucet
/// 4. balance
/// 5. transfer_call
/// 6. method_call

async fn rosetta_wallet_methods() {
    let wallet = create_wallet(
        Some("ethereum".to_owned()),
        Some("dev".to_owned()),
        Some("http://127.0.0.1:8081".to_owned()),
        None,
    )
    .await
    .unwrap();

    account(&wallet);
    network_status(&wallet).await;
    faucet(&wallet).await;
    balance(&wallet).await;
    transfer_call(&wallet).await;
    method_call(&wallet).await;
}

fn account(wallet: &Wallet) {
    println!("account identifier ==================");
    println!("{:?}", wallet.account());
}

async fn network_status(wallet: &Wallet) {
    println!("network status ==================");
    println!("{:?}", wallet.status().await);
}

async fn faucet(wallet: &Wallet) {
    println!("faucet ==================");
    println!("{:?}", wallet.faucet(1000000000000).await);
}

async fn balance(wallet: &Wallet) {
    println!("balance ==================");
    println!("{:?}", wallet.balance().await);
}

async fn transfer_call(wallet: &Wallet) {
    println!("transfer ==================");
    println!(
        "{:?}",
        wallet
            .transfer(
                &AccountIdentifier {
                    //eth.coinbase address of local network
                    address: "0x166aae20169fe6e4c79fd5f060a9c6306f09d8e0".to_owned(),
                    sub_account: None,
                    metadata: None,
                },
                1000000000000
            )
            .await
    );
    println!("latest balance ==================");
    println!("{:?}", wallet.balance().await);
}

async fn method_call(wallet: &Wallet) {
    println!("method call ==================");
    let contract_address = "0xb38dfb93a3da0f56736a0ce020bc28c141ca09bc";
    let function_signature = "function changeOwner(address newOwner)";
    let method_params = format!("{}-{}", contract_address, function_signature);
    println!(
        "{:?}",
        wallet
            .method_call(
                &method_params,
                json!(["0x166aae20169fe6e4c79fd5f060a9c6306f09d8e0"])
            )
            .await
    );
    println!("latest balance ==================");
    println!("{:?}", wallet.balance().await);
}

/// Api methods
/// 1. block
/// 2. block_transaction
/// 3. contract_call
/// 4. storage
/// 5. storage_proof
async fn rosetta_client_methods() {
    let server_url = "http://127.0.0.1:8081";
    let client = Client::new(server_url).unwrap();
    let network_identifier = NetworkIdentifier {
        blockchain: "ethereum".to_owned(),
        network: "dev".to_owned(),
        sub_network_identifier: None,
    };

    block(&client, network_identifier.clone()).await;
    block_transaction(&client, network_identifier.clone()).await;
    contract_call(&client, network_identifier.clone()).await;
    storage(&client, network_identifier.clone()).await;
    stroage_proof(&client, network_identifier.clone()).await;
}

async fn block(client: &Client, network_identifier: NetworkIdentifier) {
    let request = BlockRequest {
        network_identifier,
        block_identifier: PartialBlockIdentifier {
            index: Some(1),
            hash: None,
        },
    };
    let response = client.block(&request).await;
    println!("block response {:#?}\n", response);
}

async fn block_transaction(client: &Client, network_identifier: NetworkIdentifier) {
    let request = BlockTransactionRequest {
        network_identifier,
        block_identifier: BlockIdentifier {
            index: 1,
            hash: "d3d376808a1fa60f88845ef6c3c548b232bca9b7ab0a7caf0b757249f667a17d".to_owned(),
        },
        transaction_identifier: TransactionIdentifier {
            hash: "1712954814870eaf10405a475673eb53ccbb11d04cb4b26a433ac7e343b75db7".to_owned(),
        },
    };
    let response = client.block_transaction(&request).await;
    println!("block transaction response {:#?}\n", response);
}

async fn contract_call(client: &Client, network_identifier: NetworkIdentifier) {
    let contract_address = "0xb38dfb93a3da0f56736a0ce020bc28c141ca09bc";
    let method_signature = "function getOwner() external view returns (address)";
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

async fn storage(client: &Client, network_identifier: NetworkIdentifier) {
    let contract_address = "0xb38dfb93a3da0f56736a0ce020bc28c141ca09bc";
    let method_signature = "0000000000000000000000000000000000000000000000000000000000000000";
    let call_type = "storage";

    let method = format!("{}-{}-{}", contract_address, method_signature, call_type);
    let request = CallRequest {
        network_identifier,
        method,
        parameters: json!({}),
    };
    let response = client.call(&request).await;
    println!("storage response {:#?}", response);
}

async fn stroage_proof(client: &Client, network_identifier: NetworkIdentifier) {
    let contract_address = "0xb38dfb93a3da0f56736a0ce020bc28c141ca09bc";
    let method_signature = "0000000000000000000000000000000000000000000000000000000000000000";
    let call_type = "storage_proof";

    let method = format!("{}-{}-{}", contract_address, method_signature, call_type);

    let request = CallRequest {
        network_identifier,
        method,
        parameters: json!({}),
    };

    let response = client.call(&request).await;
    println!("storage proof response {:#?}", response);
}
