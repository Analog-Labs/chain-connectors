use anyhow::Result;
use rosetta_server_substrate::Config;

#[tokio::main]
async fn main() -> Result<()> {
    femme::start();
    let config = Config::dev();
    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.at("/")
        .nest(rosetta_server_substrate::server(&config).await?);
    app.listen(config.url).await?;
    Ok(())
}
