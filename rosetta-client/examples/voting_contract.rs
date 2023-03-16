use clap::Parser;
use rosetta_client::{create_wallet, EthereumExt, Wallet};
use serde_json::json;

#[derive(Parser)]
pub enum Command {
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
        Command::Deploy => {
            deploy_contract(&wallet).await;
        }
        Command::Vote(vote_opts) => {
            vote(&wallet, vote_opts).await;
        }
    }
}

async fn deploy_contract(wallet: &Wallet) {
    let bytes = hex::decode("608060405234801561001057600080fd5b5060008081905550600060018190555061019b8061002f6000396000f3fe608060405234801561001057600080fd5b50600436106100415760003560e01c806322721754146100465780635b7e0f27146100505780636fe95c461461005a575b600080fd5b61004e610079565b005b610058610094565b005b6100626100af565b6040516100709291906100d9565b60405180910390f35b600180600082825461008b9190610131565b92505081905550565b60016000808282546100a69190610131565b92505081905550565b600080600054600154915091509091565b6000819050919050565b6100d3816100c0565b82525050565b60006040820190506100ee60008301856100ca565b6100fb60208301846100ca565b9392505050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b600061013c826100c0565b9150610147836100c0565b925082820190508082111561015f5761015e610102565b5b9291505056fea2646970667358221220d6dcd80743cd85a7570e21755b69b903685d6b2d7ac85d68f2499adbd442480e64736f6c63430008120033").unwrap();
    let response = wallet.deploy_contract(bytes).await.unwrap();
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
    let function_signature = if data.vote_yes{
        "function vote_yes()"
    }else{
        "function vote_no()"
    };
    let method_params = format!("{}-{}", data.contract_address, function_signature);
    println!("{:?}", wallet.method_call(&method_params, json!([])).await);
    
    //doing a view call to check the vote
    let function_signature = "function get_votes_stats() external view returns (uint, uint)";
    let call_type = "call";
    let method = format!("{}-{}-{}", data.contract_address, function_signature, call_type);
    println!("{:?}", wallet.call(method, &json!({})).await);
}

