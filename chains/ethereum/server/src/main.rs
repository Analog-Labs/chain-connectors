use std::str::FromStr;

use anyhow::Result;
// use futures_util::StreamExt;
// use rosetta_core::{BlockchainClient, ClientEvent};
// use rosetta_config_ethereum::types::queries::Call;
use rosetta_ethereum_backend::{
    __reexports::primitives::{BlockIdentifier, Bytes, CallRequest},
    jsonrpsee::Adapter as JsonrpseeAdapter,
    prelude::*,
    AtBlock,
};
use rosetta_server::ws::default_client;

const NODE_URL: &str = "ws://127.0.0.1:8545";

#[tokio::main]
async fn main() -> Result<()> {
    let client = JsonrpseeAdapter(default_client(NODE_URL, None).await?);

    // let call = Call {
    //     from: None,
    //     to: None,
    //     value: None,
    //     data: Some(Bytes::from_str("0x60020260005260206000f3")?),
    // };

    let call = CallRequest {
        data: Some(Bytes::from_str("0x60020260005260206000f3")?),
        ..Default::default()
    };

    let block = BlockIdentifier::Number(1);
    {
        let value = serde_json::to_value(&call)?;
        println!(
            "RPC Request: [\n{},\n{}\n]",
            serde_json::to_string_pretty(&value)?,
            serde_json::to_string_pretty(&block)?
        );
    }

    let result = client.call(&call, AtBlock::from(block)).await;
    println!("\nresult: {result:?}");
    Ok(())

    // let client = rosetta_server_ethereum::MaybeWsEthereumClient::new(
    //     "ethereum",
    //     "dev",
    //     NODE_URL,
    // )
    // .await?;

    // for i in 0..5 {
    //     println!("Openning stream attempt {i}");
    //     let stream = client.listen().await;
    //     match stream {
    //         Ok(Some(mut stream)) => loop {
    //             let event = stream.next().await;
    //             match event {
    //                 Some(ClientEvent::Close(reason)) => {
    //                     println!("Stream closed due '{reason}'");
    //                     break;
    //                 },
    //                 Some(event) => {
    //                     println!("Event: {event:?}");
    //                 },
    //                 None => {
    //                     println!("Stream closes");
    //                     break;
    //                 },
    //             }
    //         },
    //         Ok(None) => {
    //             println!("Client doesn't support stream");
    //             break;
    //         },
    //         Err(error) => {
    //             println!("failed to open stream: {error:?}");
    //             tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    //         },
    //     }
    // }

    // Ok(())
}
