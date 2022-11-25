use anyhow::Result;
use rosetta_server_substrate::Config;
use ss58_registry::Ss58AddressFormatRegistry;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    femme::start();
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8082".into())
        .parse()?;
    let chain = std::env::var("NETWORK").unwrap_or_else(|_| "DEV".into());
    let (config, args) = match chain.as_str() {
        "POLKADOT" => (
            Config::new(
                "Polkadot",
                "Dev",
                "DOT",
                10,
                Ss58AddressFormatRegistry::PolkadotAccount,
                true,
            ),
            ["--dev"],
        ),
        "KUSAMA" => (
            Config::new(
                "Kusama",
                "Kusama",
                "KSM",
                12,
                Ss58AddressFormatRegistry::KusamaAccount,
                false,
            ),
            ["--chain=kusama"],
        ),
        "DEV" => (
            Config::new(
                "Polkadot",
                "Polkadot",
                "DOT",
                10,
                Ss58AddressFormatRegistry::PolkadotAccount,
                false,
            ),
            ["--chain=polkadot"],
        ),
        _ => anyhow::bail!("unsupported chain"),
    };

    Command::new("polkadot").args(args).spawn()?;
    std::thread::sleep(Duration::from_secs(10));

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.at("/")
        .nest(rosetta_server_substrate::server(&config).await?);
    app.listen(format!("http://0.0.0.0:{}", port)).await?;
    Ok(())
}
