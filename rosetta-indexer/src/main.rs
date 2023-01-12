use anyhow::Result;
use clap::Parser;
use rosetta_client::Chain;
use rosetta_indexer::{server, Indexer};
use std::path::PathBuf;
use tide::http::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};

#[derive(Parser)]
pub struct IndexerArgs {
    #[clap(long)]
    pub path: PathBuf,
    #[clap(long)]
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
    let indexer = Indexer::new(&opts.path, opts.server.as_deref(), opts.chain)?;

    let cors = CorsMiddleware::new()
        .allow_methods("POST".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.with(cors);
    app.at("/").nest(server(indexer).await?);
    app.listen(opts.url).await?;
    Ok(())
}
