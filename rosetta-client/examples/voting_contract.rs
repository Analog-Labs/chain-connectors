use clap::Parser;
use rosetta_client::{create_wallet, EthereumExt, Wallet};
use serde_json::{json, Value};

#[derive(Parser)]
pub enum Command {
    Faucet,
    Deploy,
    Vote(VoteOpts),
}

#[derive(Parser)]
pub struct VoteOpts {
    #[clap(long, short)]
    pub contract_address: String,
    #[clap(long, short)]
    pub vote_yes: bool,
}

#[tokio::main]
async fn main() {
    let args = Command::parse();

    let wallet = create_wallet(
        Some("ethereum".to_owned()),
        Some("dev".to_owned()),
        Some("http://127.0.0.1:8081".to_owned()),
        None,
    )
    .await
    .unwrap();
    match args {
        Command::Faucet => {
            faucet_etheruem(&wallet).await;
        }
        Command::Deploy => {
            deploy_contract(&wallet).await;
        }
        Command::Vote(vote_opts) => {
            vote(&wallet, vote_opts).await;
        }
    }
}

async fn faucet_etheruem(wallet: &Wallet) {
    println!(
        "Faucet transaction: {:?}",
        wallet.faucet(1000000000000000).await
    );
    println!("Current account balance: {:?}", wallet.balance().await);
}

async fn deploy_contract(wallet: &Wallet) {
    //getting compiled contract data
    let compiled_contract_json = include_str!("../examples/compiled_voting_contract.json");
    let json_object = serde_json::from_str::<Value>(&compiled_contract_json).unwrap();
    let contract_binary = json_object["contracts"]["voting_machine.sol:VotingMachine"]["bin"]
        .as_str()
        .unwrap();
    let bytes = hex::decode(contract_binary).unwrap();

    //deploying contract
    let response = wallet.deploy_contract(bytes).await.unwrap();

    //getting contract address
    let call_method = format!("{}--transaction_receipt", response.hash);
    let value = json!({});
    let tx_receipt = wallet.call(call_method, &value).await.unwrap();
    let contract_address = tx_receipt.result["contractAddress"].clone();
    println!(
        "Deployed contract address: {}",
        contract_address.to_string()
    );
}

async fn vote(wallet: &Wallet, data: VoteOpts) {
    // doing a vote on contract
    let function_signature = if data.vote_yes {
        "function vote_yes()"
    } else {
        "function vote_no()"
    };
    let method_params = format!("{}-{}", data.contract_address, function_signature);
    println!("{:?}", wallet.method_call(&method_params, json!([])).await);

    //doing a view call to check the vote
    let function_signature = "function get_votes_stats() external view returns (uint, uint)";
    let call_type = "call";
    let method = format!(
        "{}-{}-{}",
        data.contract_address, function_signature, call_type
    );
    println!("{:?}", wallet.call(method, &json!({})).await);
}
