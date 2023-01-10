use anyhow::Result;
use clap::Parser;
use rosetta_indexer::args::IndexerArgs;
use rosetta_indexer::server;
use tide::http::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};

#[tokio::main]
async fn main() -> Result<()> {
    femme::start();

    let opts = IndexerArgs::parse();

    let chain = opts.chain;

    let cors = CorsMiddleware::new()
        .allow_methods("POST".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.with(cors);
    app.at("/").nest(server(chain, opts.server).await?);
    app.listen(opts.url).await?;
    Ok(())
}
