use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    rosetta_server::main::<rosetta_server_astar::AstarClient>().await
}
