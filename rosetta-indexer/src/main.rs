use anyhow::Result;
use clap::Parser;
use rosetta_indexer::args::IndexerArgs;
use rosetta_indexer::server;

#[tokio::main]
async fn main() -> Result<()> {
    femme::start();

    let opts = IndexerArgs::parse();

    let chain = opts.chain;

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.at("/").nest(server(chain, opts.server).await?);
    app.listen(opts.url).await?;
    Ok(())
}
