use std::str::FromStr;

use anyhow::Result;
// use futures_util::StreamExt;
// use rosetta_core::{BlockchainClient, ClientEvent};
use rosetta_config_ethereum::types::primitives::{Address, BlockIdentifier, Bytes, Call, U256};
use rosetta_server::ws::default_client;
use rosetta_server_ethereum::client_impl::{EthereumRpc, EthereumRpcT, GenericErrorTransform};

const NODE_URL: &str = "ws://127.0.0.1:8545/ws";

#[tokio::main]
async fn main() -> Result<()> {
    let client = default_client(NODE_URL, None).await?;

    let client = EthereumRpc::new(
        client,
        GenericErrorTransform {
            revert_code: 3,
            header_not_found_code: -32000,
            out_of_gas_code: 10,
        },
    );

    let call = Call {
        from: None,
        to: None,
        gas: None,
        gas_price: None,
        value: None,
        data: Some(Bytes::from_str("0x60020260005260206000f3")?),
    };
    let block = BlockIdentifier::Number(0x1.into());
    {
        let value = serde_json::to_value(&call)?;
        println!(
            "RPC Request: [\n{},\n{}\n]",
            serde_json::to_string_pretty(&value)?,
            serde_json::to_string_pretty(&block)?
        );
    }

    let result = client.call(call, block).await;
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
