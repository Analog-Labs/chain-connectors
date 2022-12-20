use clap::Parser;
use rosetta_client::Chain;

#[derive(Parser)]
pub struct IndexerArgs {
    #[clap(long, default_value="http://127.0.0.1:8083")]
    pub url: String,
    #[clap(long)]
    pub server: Option<String>,
    #[clap(long)]
    pub chain: Chain,
}
