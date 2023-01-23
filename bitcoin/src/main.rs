use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    rosetta::server::main::<rosetta_server_bitcoin::BitcoinClient>().await
}
