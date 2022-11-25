use anyhow::{Context, Result};
use clap::Parser;
use rosetta_server_substrate::{Config, Ss58AddressFormatRegistry};

#[derive(Parser)]
struct Opts {
    #[clap(long, default_value = "dev")]
    network: String,
    #[clap(long, default_value = "http://127.0.0.1:8082")]
    url: String,
    #[clap(long, default_value = "ws://127.0.0.1:9944")]
    rpc_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    femme::start();
    let opts = Opts::parse();

    let config = match opts.network.as_str() {
        "dev" => Config::new(
            &opts.rpc_url,
            "Polkadot",
            "Dev",
            "DOT",
            10,
            Ss58AddressFormatRegistry::PolkadotAccount,
            true,
        ),
        "kusama" => Config::new(
            &opts.rpc_url,
            "Kusama",
            "Kusama",
            "KSM",
            12,
            Ss58AddressFormatRegistry::KusamaAccount,
            false,
        ),
        "polkadot" => Config::new(
            &opts.rpc_url,
            "Polkadot",
            "Polkadot",
            "DOT",
            10,
            Ss58AddressFormatRegistry::PolkadotAccount,
            false,
        ),
        _ => anyhow::bail!("unsupported network"),
    };

    let server = rosetta_server_substrate::server(&config)
        .await
        .with_context(|| format!("connecting to {}", &opts.rpc_url))?;

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.at("/").nest(server);
    app.listen(&opts.url)
        .await
        .with_context(|| format!("listening on {}", &opts.url))?;

    Ok(())
}
