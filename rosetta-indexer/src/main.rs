use anyhow::Result;
use clap::Parser;
use rosetta_indexer::args::IndexerArgs;
use rosetta_indexer::server;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let opts = IndexerArgs::parse();

    let chain = opts.chain;

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.at("/").nest(server(chain).await?);
    app.listen(format!("127.0.0.1:{}", opts.port)).await?;
    Ok(())
}
