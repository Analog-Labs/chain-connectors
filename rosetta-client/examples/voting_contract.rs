use clap::Parser;
use rosetta_client::{create_wallet, EthereumExt, Wallet};
use rosetta_core::BlockchainClient;

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
    let ethereum_config =
        rosetta_server_ethereum::MaybeWsEthereumClient::create_config("dev").unwrap();
    let client = rosetta_server_ethereum::MaybeWsEthereumClient::new(
        ethereum_config,
        "ws://127.0.0.1:8545".to_owned(),
    )
    .await
    .unwrap();
    let wallet = create_wallet(client, None).unwrap();
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

async fn faucet_etheruem<T: BlockchainClient>(wallet: &Wallet<T>) {
    println!(
        "Faucet transaction: {:?}",
        wallet.faucet(1000000000000000).await
    );
    println!("Current account balance: {:?}", wallet.balance().await);
}

async fn deploy_contract<T: BlockchainClient>(wallet: &Wallet<T>) {
    //getting compiled contract data
    let compiled_contract_bin = include_str!("../examples/voting_contract.bin")
        .strip_suffix('\n')
        .unwrap();
    let bytes = hex::decode(compiled_contract_bin).unwrap();

    //deploying contract
    let tx_hash = wallet.eth_deploy_contract(bytes).await.unwrap();

    //getting contract address
    let tx_receipt = wallet.eth_transaction_receipt(&tx_hash).await.unwrap();
    let contract_address = tx_receipt
        .get("contractAddress")
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap();
    println!("Deployed contract address: {}", contract_address);
}

async fn vote<T: BlockchainClient>(wallet: &Wallet<T>, data: VoteOpts) {
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
            .eth_view_call(&data.contract_address, function_signature, &[], None)
            .await
    );
}
