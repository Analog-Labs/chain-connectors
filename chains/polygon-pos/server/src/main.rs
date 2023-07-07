use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    rosetta_server::main::<rosetta_server_polygon_pos::PolygonPosClient>().await
}
