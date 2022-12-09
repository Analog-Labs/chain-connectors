
use clap::Parser;
use rosetta_client::Chain;

#[derive(Parser)]
pub struct IndexerArgs {
    #[clap(long)]
    pub url: String,
    #[clap(long)]
    pub port: String,
    #[clap(long)]
    pub chain: Chain,
}
