#![allow(missing_docs, unused_variables, trivial_casts)]

use clap::{App, Arg};
#[allow(unused_imports)]
use futures::{future, stream, Stream};
#[allow(unused_imports)]
use openapi_client::{
    models, AccountBalanceResponse, AccountCoinsResponse, Api, ApiNoContext, BlockResponse,
    BlockTransactionResponse, CallResponse, Client, ConstructionCombineResponse,
    ConstructionDeriveResponse, ConstructionHashResponse, ConstructionMetadataResponse,
    ConstructionParseResponse, ConstructionPayloadsResponse, ConstructionPreprocessResponse,
    ConstructionSubmitResponse, ContextWrapperExt, EventsBlocksResponse, MempoolResponse,
    MempoolTransactionResponse, NetworkListResponse, NetworkOptionsResponse, NetworkStatusResponse,
    SearchTransactionsResponse,
};

#[allow(unused_imports)]
use log::info;

// swagger::Has may be unused if there are no examples
#[allow(unused_imports)]
use swagger::{AuthData, ContextBuilder, EmptyContext, Has, Push, XSpanIdString};

type ClientContext = swagger::make_context_ty!(
    ContextBuilder,
    EmptyContext,
    Option<AuthData>,
    XSpanIdString
);

// rt may be unused if there are no examples
#[allow(unused_mut)]
fn main() {
    env_logger::init();

    let matches = App::new("client")
        .arg(
            Arg::with_name("operation")
                .help("Sets the operation to run")
                .possible_values(&[])
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("https")
                .long("https")
                .help("Whether to use HTTPS or not"),
        )
        .arg(
            Arg::with_name("host")
                .long("host")
                .takes_value(true)
                .default_value("localhost")
                .help("Hostname to contact"),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .takes_value(true)
                .default_value("8080")
                .help("Port to contact"),
        )
        .get_matches();

    let is_https = matches.is_present("https");
    let base_url = format!(
        "{}://{}:{}",
        if is_https { "https" } else { "http" },
        matches.value_of("host").unwrap(),
        matches.value_of("port").unwrap()
    );

    let context: ClientContext = swagger::make_context!(
        ContextBuilder,
        EmptyContext,
        None as Option<AuthData>,
        XSpanIdString::default()
    );

    let mut client: Box<dyn ApiNoContext<ClientContext>> = if matches.is_present("https") {
        // Using Simple HTTPS
        let client =
            Box::new(Client::try_new_https(&base_url).expect("Failed to create HTTPS client"));
        Box::new(client.with_context(context))
    } else {
        // Using HTTP
        let client =
            Box::new(Client::try_new_http(&base_url).expect("Failed to create HTTP client"));
        Box::new(client.with_context(context))
    };

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    match matches.value_of("operation") {
        /* Disabled because there's no example.
        Some("AccountBalance") => {
            let result = rt.block_on(client.account_balance(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("AccountCoins") => {
            let result = rt.block_on(client.account_coins(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("Block") => {
            let result = rt.block_on(client.block(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("BlockTransaction") => {
            let result = rt.block_on(client.block_transaction(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("Call") => {
            let result = rt.block_on(client.call(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("ConstructionCombine") => {
            let result = rt.block_on(client.construction_combine(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("ConstructionDerive") => {
            let result = rt.block_on(client.construction_derive(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("ConstructionHash") => {
            let result = rt.block_on(client.construction_hash(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("ConstructionMetadata") => {
            let result = rt.block_on(client.construction_metadata(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("ConstructionParse") => {
            let result = rt.block_on(client.construction_parse(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("ConstructionPayloads") => {
            let result = rt.block_on(client.construction_payloads(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("ConstructionPreprocess") => {
            let result = rt.block_on(client.construction_preprocess(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("ConstructionSubmit") => {
            let result = rt.block_on(client.construction_submit(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("EventsBlocks") => {
            let result = rt.block_on(client.events_blocks(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("Mempool") => {
            let result = rt.block_on(client.mempool(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("MempoolTransaction") => {
            let result = rt.block_on(client.mempool_transaction(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("NetworkList") => {
            let result = rt.block_on(client.network_list(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("NetworkOptions") => {
            let result = rt.block_on(client.network_options(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("NetworkStatus") => {
            let result = rt.block_on(client.network_status(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("SearchTransactions") => {
            let result = rt.block_on(client.search_transactions(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        _ => {
            panic!("Invalid operation provided")
        }
    }
}
