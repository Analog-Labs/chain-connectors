use anyhow::Result;
use rosetta_server_substrate::Config;
use ss58_registry::Ss58AddressFormatRegistry;

#[tokio::main]
async fn main() -> Result<()> {
    femme::start();
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8082".into())
        .parse()?;
    let chain = std::env::var("NETWORK").unwrap_or_else(|_| "DEV".into());
    let node = std::env::var("NODE").unwrap_or_else(|_| "ws://127.0.0.1:9944".into());
    let config = match chain.as_str() {
        "POLKADOT" => Config::new(
            &node,
            "Polkadot",
            "Dev",
            "DOT",
            10,
            Ss58AddressFormatRegistry::PolkadotAccount,
            true,
        ),
        "KUSAMA" => Config::new(
            &node,
            "Kusama",
            "Kusama",
            "KSM",
            12,
            Ss58AddressFormatRegistry::KusamaAccount,
            false,
        ),
        "DEV" => Config::new(
            &node,
            "Polkadot",
            "Polkadot",
            "DOT",
            10,
            Ss58AddressFormatRegistry::PolkadotAccount,
            false,
        ),
        _ => anyhow::bail!("unsupported chain"),
    };

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.at("/")
        .nest(rosetta_server_substrate::server(&config).await?);
    app.listen(format!("http://0.0.0.0:{}", port)).await?;
    Ok(())
}
