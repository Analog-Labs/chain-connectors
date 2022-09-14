use anyhow::Result;
use rosetta_sdk_rust_gen::client::Client;
use rosetta_sdk_rust_gen::models::{
    AccountBalanceRequest, AccountIdentifier, MetadataRequest, NetworkRequest,
};
use rosetta_sdk_rust_gen::{
    AccountBalanceResponse, ApiNoContext, ContextWrapperExt, NetworkListResponse,
    NetworkOptionsResponse, NetworkStatusResponse,
};
use swagger::{AuthData, ContextBuilder, EmptyContext, Push, XSpanIdString};

type ClientContext = swagger::make_context_ty!(
    ContextBuilder,
    EmptyContext,
    Option<AuthData>,
    XSpanIdString,
);

#[tokio::main]
async fn main() -> Result<()> {
    let url = "http://127.0.0.1:8080";
    let context: ClientContext = swagger::make_context!(
        ContextBuilder,
        EmptyContext,
        None as Option<AuthData>,
        XSpanIdString::default(),
    );
    let client: Box<dyn ApiNoContext<ClientContext>> = {
        let client = Box::new(Client::try_new_http(&url).expect("Failed to create HTTP client"));
        Box::new(client.with_context(context))
    };

    let res = match client.network_list(MetadataRequest::new()).await? {
        NetworkListResponse::ExpectedResponseToAValidRequest(res) => res,
        NetworkListResponse::UnexpectedError(err) => {
            anyhow::bail!("{}", err.to_string());
        }
    };
    for identifier in &res.network_identifiers {
        println!("{:?}", identifier);
        let options = match client
            .network_options(NetworkRequest::new(identifier.clone()))
            .await?
        {
            NetworkOptionsResponse::ExpectedResponseToAValidRequest(res) => res,
            NetworkOptionsResponse::UnexpectedError(err) => {
                anyhow::bail!("{}", err.to_string());
            }
        };
        println!("{:#?}", options);
        let status = match client
            .network_status(NetworkRequest::new(identifier.clone()))
            .await?
        {
            NetworkStatusResponse::ExpectedResponseToAValidRequest(res) => res,
            NetworkStatusResponse::UnexpectedError(err) => {
                anyhow::bail!("{}", err.to_string());
            }
        };
        println!("{:#?}", status);
    }

    // BTC testnet address
    let address = "mk5QsyCteBgeRPF8tkiCZq9usMT8cZP2MT";
    // ETH testnet address
    // let address = "0x25c4a76E7d118705e7Ea2e9b7d8C59930d8aCD3b";

    let req = AccountBalanceRequest {
        account_identifier: AccountIdentifier::new(address.to_string()),
        block_identifier: None,
        currencies: None,
        network_identifier: res.network_identifiers[0].clone(),
    };
    let balance = match client.account_balance(req).await? {
        AccountBalanceResponse::ExpectedResponseToAValidRequest(res) => res,
        AccountBalanceResponse::UnexpectedError(err) => {
            anyhow::bail!("{}", err.to_string());
        }
    };
    println!("{:?}", balance);
    Ok(())
}
