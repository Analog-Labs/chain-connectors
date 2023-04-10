use clap::Parser;
use rosetta_client::{create_wallet, EthereumExt, Wallet};

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
        Some("http://rosetta.analog.one:8081".to_owned()),
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
    let compiled_contract_bin = include_str!("../examples/voting_contract.bin")
        .strip_suffix('\n')
        .unwrap();
    let bytes = hex::decode(compiled_contract_bin).unwrap();

    //deploying contract
    let response = wallet.eth_deploy_contract(bytes).await.unwrap();

    //getting contract address
    let tx_receipt = wallet
        .eth_transaction_receipt(&response.hash)
        .await
        .unwrap();
    let contract_address = tx_receipt.result["contractAddress"].clone();
    println!("Deployed contract address: {}", contract_address);
}

async fn vote(wallet: &Wallet, data: VoteOpts) {
    // doing a vote on contract
    let function_signature = if data.vote_yes {
        "function vote_yes()"
    } else {
        "function vote_no()"
    };
    println!(
        "{:?}",
        wallet
            .eth_send_call(&data.contract_address, function_signature, &[], 0)
            .await
    );

    //doing a view call to check the vote
    let function_signature = "function get_votes_stats() external view returns (uint, uint)";
    println!(
        "{:?}",
        wallet
            .eth_view_call(&data.contract_address, function_signature)
            .await
    );
}
