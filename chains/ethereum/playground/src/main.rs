mod compiler;

use std::path::PathBuf;

use jsonrpsee::ws_client::WsClientBuilder;
use rosetta_ethereum_executor::{
    backend::{jsonrpsee::Adapter, AtBlock, EthereumRpc, TransactionCall},
    primitives::{Address, Bytes, H256, U256, U64},
};

#[cfg(feature = "sputnik-vm")]
use rosetta_ethereum_executor::vms::SputnikEVM as EVM;

#[cfg(feature = "rust-vm")]
use rosetta_ethereum_executor::vms::RustEVM as EVM;

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
    .with_level(true)
    .with_max_level(tracing::Level::TRACE)
    .init();

    let solidity_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("res");
    println!("solidity_dir: {solidity_dir:?}");
    compiler::compile(&solidity_dir);
    return Ok(());

    let uri = url::Url::parse("ws://127.0.0.1:8545")?;
    let client = Adapter(WsClientBuilder::default().build(uri).await?);

    let chain_id = client.chain_id().await?;
    println!("chain_id: {chain_id:?}\n");
    let chain_id = u64::try_from(chain_id).map_err(|error| eyre::format_err!("{error}"))?;

    let result = client
        .get_proof(
            Address::from(hex_literal::hex!("34964a63A099F8DE44AcD5318374c6395c052734")),
            &[],
            AtBlock::Latest,
        )
        .await?;
    println!("account: {}\n", serde_json::to_string_pretty(&result)?);

    let result = client
        .storage(
            Address::from(hex_literal::hex!("34964a63A099F8DE44AcD5318374c6395c052734")),
            H256::zero(),
            AtBlock::Latest,
        )
        .await?;
    println!("storage: {}\n", serde_json::to_string_pretty(&result)?);

    let result = client
        .create_access_list(
            &TransactionCall {
                from: Some(Address::zero()),
                to: Some(Address::from(hex_literal::hex!(
                    "99d6b5638DD27BC2fE201FB6854C61F6b698C403"
                ))),
                gas_limit: Some(U64::MAX),
                gas_price: None,
                value: Some(U256::zero()),
                data: Some(Bytes::from(hex_literal::hex!(
                    "ee58e99d0000000000000000000000000000000000000000000000000000000000000001"
                ))),
                chain_id: Some(chain_id.into()),
                ..TransactionCall::default()
            },
            AtBlock::Latest,
        )
        .await?;
    println!("access list: {}\n", serde_json::to_string_pretty(&result)?);

    let result = client.block(AtBlock::Latest).await?;
    println!("block: {}\n", serde_json::to_string_pretty(&result)?);

    // Contract Addr 21030 23630
    // let contract_addr = hex_literal::hex!("34964a63A099F8DE44AcD5318374c6395c052734");
    // let data = hex_literal::hex!("fe3fb5c7");

    let contract_addr = hex_literal::hex!("e0607131aBEE20c0d351EE83AB981091798FdA4C");
    let data = hex_literal::hex!(
        "ee58e99d0000000000000000000000000000000000000000000000000000000000000001"
    );
    let gas_limit = 0x0000_0fff_ffff_u64;

    let tx = TransactionCall {
        from: Some(Address::zero()),
        to: Some(Address::from(contract_addr)),
        gas_limit: Some(U64::from(gas_limit)),
        gas_price: None,
        value: Some(U256::zero()),
        data: Some(Bytes::from(data)),
        chain_id: Some(chain_id.into()),
        ..TransactionCall::default()
    };

    let result = client.call(&tx, AtBlock::Latest).await;
    let gas_used = client.estimate_gas(&tx, AtBlock::Latest).await?;
    if let Ok(result) = result {
        println!("\n\ngas: {gas_used}\n{}\n", serde_json::to_string_pretty(&result)?);
    } else {
        println!("\n\ngas: {gas_used}\n{result:?}\n");
    }

    // let mut evm = evm::Executor::new(client).await?;
    let mut evm = EVM::new(client);
    let res = evm.call(&tx, AtBlock::Latest).await;
    println!("{res:?}");
    Ok(())
}
