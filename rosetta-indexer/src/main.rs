use anyhow::Result;
use clap::Parser;
use rosetta_client::Chain;
use rosetta_indexer::server;
use tide::http::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};

#[derive(Parser)]
pub struct IndexerArgs {
    #[clap(long, default_value = "http://127.0.0.1:8083")]
    pub url: String,
    #[clap(long)]
    pub server: Option<String>,
    #[clap(long)]
    pub chain: Chain,
}

#[tokio::main]
async fn main() -> Result<()> {
    femme::start();

    let opts = IndexerArgs::parse();

    let cors = CorsMiddleware::new()
        .allow_methods("POST".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.with(cors);
    app.at("/").nest(server(opts.chain, opts.server).await?);
    app.listen(opts.url).await?;
    Ok(())
}
