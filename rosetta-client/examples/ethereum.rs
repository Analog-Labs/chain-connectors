use clap::Parser;
use rosetta_client::{
    create_wallet,
    types::{AccountIdentifier, Block, PartialBlockIdentifier},
    EthereumExt, Wallet,
};
use rosetta_core::BlockchainClient;

#[derive(Parser)]
struct EthereumOpts {
    #[clap(long, short)]
    contract_address: String,
}

// cargo run --example voting_contract vote --contract-address "0x678ea0447843f69805146c521afcbcc07d6e28a2"
#[tokio::main]
async fn main() {
    let opts = EthereumOpts::parse();
    let ethereum_config =
        rosetta_server_ethereum::MaybeWsEthereumClient::create_config("dev").unwrap();
    let client = rosetta_server_ethereum::MaybeWsEthereumClient::new(
        ethereum_config,
        "ws://127.0.0.1:8545".to_owned(),
    )
    .await
    .unwrap();
    rosetta_wallet_methods(client, &opts.contract_address).await;
}

/// Wallet methods
/// 1. account
/// 2. network_status
/// 3. faucet
/// 4. balance
/// 5. transfer_call
/// 6. method_call
/// 7. block
/// 8. block_transaction
/// 9. call
/// 10. storage
/// 11. storage_proof
async fn rosetta_wallet_methods<T: BlockchainClient>(client: T, contract_address: &str) {
    let wallet = create_wallet(client, None).unwrap();

    account(&wallet);
    network_status(&wallet).await;
    faucet(&wallet).await;
    balance(&wallet).await;
    transfer_call(&wallet).await;
    let block_response = block(&wallet).await;
    block_transaction(&wallet, block_response).await;
    method_call(&wallet, contract_address).await;
    contract_call(&wallet, contract_address, None).await;
    storage_yes_votes(&wallet, contract_address, None).await;
    storage_no_votes(&wallet, contract_address, None).await;
    storage_proof(&wallet, contract_address, None).await;
}

fn account<T: BlockchainClient>(wallet: &Wallet<T>) {
    println!("account identifier ==================");
    println!("{:?}", wallet.account());
}

async fn network_status<T: BlockchainClient>(wallet: &Wallet<T>) {
    println!("network status ==================");
    println!("{:?}", wallet.status().await);
}

async fn faucet<T: BlockchainClient>(wallet: &Wallet<T>) {
    println!("faucet ==================");
    println!("{:?}", wallet.faucet(1000000000000000).await);
}

async fn balance<T: BlockchainClient>(wallet: &Wallet<T>) {
    println!("balance ==================");
    println!("{:?}", wallet.balance().await);
}

async fn transfer_call<T: BlockchainClient>(wallet: &Wallet<T>) {
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

async fn block<T: BlockchainClient>(wallet: &Wallet<T>) -> Block {
    //getting latest block data
    let network_status = wallet.status().await.unwrap();

    let block_identifier = PartialBlockIdentifier {
        index: Some(network_status.index),
        hash: None,
    };
    let response = wallet.block(block_identifier).await.unwrap();
    println!("block {:#?}\n", response);
    response
}

async fn block_transaction<T: BlockchainClient>(wallet: &Wallet<T>, block_data: Block) {
    let block_identifier = block_data.block_identifier;
    //taking transaction_identifier from block data
    let transaction_identifier = block_data
        .transactions
        .last()
        .unwrap()
        .transaction_identifier
        .clone();

    let response = wallet
        .block_transaction(block_identifier, transaction_identifier)
        .await;
    println!("block transaction response {:#?}\n", response);
}

async fn method_call<T: BlockchainClient>(wallet: &Wallet<T>, contract_address: &str) {
    println!("method call ==================");
    let function_signature = "function vote_yes()";
    println!(
        "{:?}",
        wallet
            .eth_send_call(contract_address, function_signature, &[], 0)
            .await
    );
    println!("latest balance ==================");
    println!("{:?}", wallet.balance().await);
}

async fn contract_call<T: BlockchainClient>(
    wallet: &Wallet<T>,
    contract_address: &str,
    block_identifier: Option<PartialBlockIdentifier>,
) {
    let method_signature = "function get_votes_stats() external view returns (uint, uint)";
    let response = wallet
        .eth_view_call(contract_address, method_signature, &[], block_identifier)
        .await;
    println!("contract call response {:#?}\n", response);
}

async fn storage_yes_votes<T: BlockchainClient>(
    wallet: &Wallet<T>,
    contract_address: &str,
    block_identifier: Option<PartialBlockIdentifier>,
) {
    // 0th position of storage in contract
    let storage_slot = "0000000000000000000000000000000000000000000000000000000000000000";
    let response = wallet
        .eth_storage(contract_address, storage_slot, block_identifier)
        .await;
    println!("storage 0th response {:#?}", response);
}

async fn storage_no_votes<T: BlockchainClient>(
    wallet: &Wallet<T>,
    contract_address: &str,
    block_identifier: Option<PartialBlockIdentifier>,
) {
    // 0th position of storage in contract
    let storage_slot = "0000000000000000000000000000000000000000000000000000000000000001";
    let response = wallet
        .eth_storage(contract_address, storage_slot, block_identifier)
        .await;
    println!("storage 1th response {:#?}", response);
}

async fn storage_proof<T: BlockchainClient>(
    wallet: &Wallet<T>,
    contract_address: &str,
    block_identifier: Option<PartialBlockIdentifier>,
) {
    // 0th position of storage_proof in contract
    let storage_slot = "0000000000000000000000000000000000000000000000000000000000000000";
    let response = wallet
        .eth_storage_proof(contract_address, storage_slot, block_identifier)
        .await;
    println!("storage proof 0th index response {:#?}", response);
}
