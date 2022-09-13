use futures::{future, future::BoxFuture, future::FutureExt, stream, stream::TryStreamExt, Stream};
use hyper::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use log::warn;
#[allow(unused_imports)]
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::task::{Context, Poll};
pub use swagger::auth::Authorization;
use swagger::auth::Scopes;
use swagger::{ApiError, BodyExt, Has, RequestParser, XSpanIdString};
use url::form_urlencoded;

use crate::header;
#[allow(unused_imports)]
use crate::models;

pub use crate::context;

type ServiceFuture = BoxFuture<'static, Result<Response<Body>, crate::ServiceError>>;

use crate::{
    AccountBalanceResponse, AccountCoinsResponse, Api, BlockResponse, BlockTransactionResponse,
    CallResponse, ConstructionCombineResponse, ConstructionDeriveResponse,
    ConstructionHashResponse, ConstructionMetadataResponse, ConstructionParseResponse,
    ConstructionPayloadsResponse, ConstructionPreprocessResponse, ConstructionSubmitResponse,
    EventsBlocksResponse, MempoolResponse, MempoolTransactionResponse, NetworkListResponse,
    NetworkOptionsResponse, NetworkStatusResponse, SearchTransactionsResponse,
};

mod paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GLOBAL_REGEX_SET: regex::RegexSet = regex::RegexSet::new(vec![
            r"^/account/balance$",
            r"^/account/coins$",
            r"^/block$",
            r"^/block/transaction$",
            r"^/call$",
            r"^/construction/combine$",
            r"^/construction/derive$",
            r"^/construction/hash$",
            r"^/construction/metadata$",
            r"^/construction/parse$",
            r"^/construction/payloads$",
            r"^/construction/preprocess$",
            r"^/construction/submit$",
            r"^/events/blocks$",
            r"^/mempool$",
            r"^/mempool/transaction$",
            r"^/network/list$",
            r"^/network/options$",
            r"^/network/status$",
            r"^/search/transactions$"
        ])
        .expect("Unable to create global regex set");
    }
    pub(crate) static ID_ACCOUNT_BALANCE: usize = 0;
    pub(crate) static ID_ACCOUNT_COINS: usize = 1;
    pub(crate) static ID_BLOCK: usize = 2;
    pub(crate) static ID_BLOCK_TRANSACTION: usize = 3;
    pub(crate) static ID_CALL: usize = 4;
    pub(crate) static ID_CONSTRUCTION_COMBINE: usize = 5;
    pub(crate) static ID_CONSTRUCTION_DERIVE: usize = 6;
    pub(crate) static ID_CONSTRUCTION_HASH: usize = 7;
    pub(crate) static ID_CONSTRUCTION_METADATA: usize = 8;
    pub(crate) static ID_CONSTRUCTION_PARSE: usize = 9;
    pub(crate) static ID_CONSTRUCTION_PAYLOADS: usize = 10;
    pub(crate) static ID_CONSTRUCTION_PREPROCESS: usize = 11;
    pub(crate) static ID_CONSTRUCTION_SUBMIT: usize = 12;
    pub(crate) static ID_EVENTS_BLOCKS: usize = 13;
    pub(crate) static ID_MEMPOOL: usize = 14;
    pub(crate) static ID_MEMPOOL_TRANSACTION: usize = 15;
    pub(crate) static ID_NETWORK_LIST: usize = 16;
    pub(crate) static ID_NETWORK_OPTIONS: usize = 17;
    pub(crate) static ID_NETWORK_STATUS: usize = 18;
    pub(crate) static ID_SEARCH_TRANSACTIONS: usize = 19;
}

pub struct MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    api_impl: T,
    marker: PhantomData<C>,
}

impl<T, C> MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    pub fn new(api_impl: T) -> Self {
        MakeService {
            api_impl,
            marker: PhantomData,
        }
    }
}

impl<T, C, Target> hyper::service::Service<Target> for MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    type Response = Service<T, C>;
    type Error = crate::ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, target: Target) -> Self::Future {
        futures::future::ok(Service::new(self.api_impl.clone()))
    }
}

fn method_not_allowed() -> Result<Response<Body>, crate::ServiceError> {
    Ok(Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::empty())
        .expect("Unable to create Method Not Allowed response"))
}

pub struct Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    api_impl: T,
    marker: PhantomData<C>,
}

impl<T, C> Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    pub fn new(api_impl: T) -> Self {
        Service {
            api_impl: api_impl,
            marker: PhantomData,
        }
    }
}

impl<T, C> Clone for Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Service {
            api_impl: self.api_impl.clone(),
            marker: self.marker.clone(),
        }
    }
}

impl<T, C> hyper::service::Service<(Request<Body>, C)> for Service<T, C>
where
    T: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = crate::ServiceError;
    type Future = ServiceFuture;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.api_impl.poll_ready(cx)
    }

    fn call(&mut self, req: (Request<Body>, C)) -> Self::Future {
        async fn run<T, C>(
            mut api_impl: T,
            req: (Request<Body>, C),
        ) -> Result<Response<Body>, crate::ServiceError>
        where
            T: Api<C> + Clone + Send + 'static,
            C: Has<XSpanIdString> + Send + Sync + 'static,
        {
            let (request, context) = req;
            let (parts, body) = request.into_parts();
            let (method, uri, headers) = (parts.method, parts.uri, parts.headers);
            let path = paths::GLOBAL_REGEX_SET.matches(uri.path());

            match &method {
                // AccountBalance - POST /account/balance
                &hyper::Method::POST if path.matched(paths::ID_ACCOUNT_BALANCE) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_account_balance_request: Option<models::AccountBalanceRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_account_balance_request) => param_account_balance_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter AccountBalanceRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter AccountBalanceRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_account_balance_request = match param_account_balance_request {
                                    Some(param_account_balance_request) => param_account_balance_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter AccountBalanceRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter AccountBalanceRequest")),
                                };

                                let result = api_impl.account_balance(
                                            param_account_balance_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                AccountBalanceResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_BALANCE_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                AccountBalanceResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_BALANCE_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter AccountBalanceRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter AccountBalanceRequest")),
                        }
                }

                // AccountCoins - POST /account/coins
                &hyper::Method::POST if path.matched(paths::ID_ACCOUNT_COINS) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_account_coins_request: Option<models::AccountCoinsRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_account_coins_request) => param_account_coins_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter AccountCoinsRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter AccountCoinsRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_account_coins_request = match param_account_coins_request {
                                    Some(param_account_coins_request) => param_account_coins_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter AccountCoinsRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter AccountCoinsRequest")),
                                };

                                let result = api_impl.account_coins(
                                            param_account_coins_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                AccountCoinsResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_COINS_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                AccountCoinsResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for ACCOUNT_COINS_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter AccountCoinsRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter AccountCoinsRequest")),
                        }
                }

                // Block - POST /block
                &hyper::Method::POST if path.matched(paths::ID_BLOCK) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_block_request: Option<models::BlockRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_block_request) => param_block_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter BlockRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter BlockRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_block_request = match param_block_request {
                                    Some(param_block_request) => param_block_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter BlockRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter BlockRequest")),
                                };

                                let result = api_impl.block(
                                            param_block_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                BlockResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for BLOCK_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                BlockResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for BLOCK_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter BlockRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter BlockRequest")),
                        }
                }

                // BlockTransaction - POST /block/transaction
                &hyper::Method::POST if path.matched(paths::ID_BLOCK_TRANSACTION) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_block_transaction_request: Option<models::BlockTransactionRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_block_transaction_request) => param_block_transaction_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter BlockTransactionRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter BlockTransactionRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_block_transaction_request = match param_block_transaction_request {
                                    Some(param_block_transaction_request) => param_block_transaction_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter BlockTransactionRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter BlockTransactionRequest")),
                                };

                                let result = api_impl.block_transaction(
                                            param_block_transaction_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                BlockTransactionResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for BLOCK_TRANSACTION_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                BlockTransactionResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for BLOCK_TRANSACTION_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter BlockTransactionRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter BlockTransactionRequest")),
                        }
                }

                // Call - POST /call
                &hyper::Method::POST if path.matched(paths::ID_CALL) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_call_request: Option<models::CallRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_call_request) => param_call_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter CallRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter CallRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_call_request = match param_call_request {
                                    Some(param_call_request) => param_call_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter CallRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter CallRequest")),
                                };

                                let result = api_impl.call(
                                            param_call_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                CallResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CALL_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                CallResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CALL_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter CallRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter CallRequest")),
                        }
                }

                // ConstructionCombine - POST /construction/combine
                &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_COMBINE) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_construction_combine_request: Option<models::ConstructionCombineRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_construction_combine_request) => param_construction_combine_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ConstructionCombineRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ConstructionCombineRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_construction_combine_request = match param_construction_combine_request {
                                    Some(param_construction_combine_request) => param_construction_combine_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ConstructionCombineRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ConstructionCombineRequest")),
                                };

                                let result = api_impl.construction_combine(
                                            param_construction_combine_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ConstructionCombineResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_COMBINE_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ConstructionCombineResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_COMBINE_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ConstructionCombineRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ConstructionCombineRequest")),
                        }
                }

                // ConstructionDerive - POST /construction/derive
                &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_DERIVE) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_construction_derive_request: Option<models::ConstructionDeriveRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_construction_derive_request) => param_construction_derive_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ConstructionDeriveRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ConstructionDeriveRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_construction_derive_request = match param_construction_derive_request {
                                    Some(param_construction_derive_request) => param_construction_derive_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ConstructionDeriveRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ConstructionDeriveRequest")),
                                };

                                let result = api_impl.construction_derive(
                                            param_construction_derive_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ConstructionDeriveResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_DERIVE_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ConstructionDeriveResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_DERIVE_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ConstructionDeriveRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ConstructionDeriveRequest")),
                        }
                }

                // ConstructionHash - POST /construction/hash
                &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_HASH) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_construction_hash_request: Option<models::ConstructionHashRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_construction_hash_request) => param_construction_hash_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ConstructionHashRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ConstructionHashRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_construction_hash_request = match param_construction_hash_request {
                                    Some(param_construction_hash_request) => param_construction_hash_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ConstructionHashRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ConstructionHashRequest")),
                                };

                                let result = api_impl.construction_hash(
                                            param_construction_hash_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ConstructionHashResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_HASH_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ConstructionHashResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_HASH_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ConstructionHashRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ConstructionHashRequest")),
                        }
                }

                // ConstructionMetadata - POST /construction/metadata
                &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_METADATA) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_construction_metadata_request: Option<models::ConstructionMetadataRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_construction_metadata_request) => param_construction_metadata_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ConstructionMetadataRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ConstructionMetadataRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_construction_metadata_request = match param_construction_metadata_request {
                                    Some(param_construction_metadata_request) => param_construction_metadata_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ConstructionMetadataRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ConstructionMetadataRequest")),
                                };

                                let result = api_impl.construction_metadata(
                                            param_construction_metadata_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ConstructionMetadataResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_METADATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ConstructionMetadataResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_METADATA_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ConstructionMetadataRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ConstructionMetadataRequest")),
                        }
                }

                // ConstructionParse - POST /construction/parse
                &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_PARSE) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_construction_parse_request: Option<models::ConstructionParseRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_construction_parse_request) => param_construction_parse_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ConstructionParseRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ConstructionParseRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_construction_parse_request = match param_construction_parse_request {
                                    Some(param_construction_parse_request) => param_construction_parse_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ConstructionParseRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ConstructionParseRequest")),
                                };

                                let result = api_impl.construction_parse(
                                            param_construction_parse_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ConstructionParseResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_PARSE_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ConstructionParseResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_PARSE_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ConstructionParseRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ConstructionParseRequest")),
                        }
                }

                // ConstructionPayloads - POST /construction/payloads
                &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_PAYLOADS) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_construction_payloads_request: Option<models::ConstructionPayloadsRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_construction_payloads_request) => param_construction_payloads_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ConstructionPayloadsRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ConstructionPayloadsRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_construction_payloads_request = match param_construction_payloads_request {
                                    Some(param_construction_payloads_request) => param_construction_payloads_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ConstructionPayloadsRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ConstructionPayloadsRequest")),
                                };

                                let result = api_impl.construction_payloads(
                                            param_construction_payloads_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ConstructionPayloadsResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_PAYLOADS_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ConstructionPayloadsResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_PAYLOADS_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ConstructionPayloadsRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ConstructionPayloadsRequest")),
                        }
                }

                // ConstructionPreprocess - POST /construction/preprocess
                &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_PREPROCESS) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_construction_preprocess_request: Option<models::ConstructionPreprocessRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_construction_preprocess_request) => param_construction_preprocess_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ConstructionPreprocessRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ConstructionPreprocessRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_construction_preprocess_request = match param_construction_preprocess_request {
                                    Some(param_construction_preprocess_request) => param_construction_preprocess_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ConstructionPreprocessRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ConstructionPreprocessRequest")),
                                };

                                let result = api_impl.construction_preprocess(
                                            param_construction_preprocess_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ConstructionPreprocessResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_PREPROCESS_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ConstructionPreprocessResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_PREPROCESS_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ConstructionPreprocessRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ConstructionPreprocessRequest")),
                        }
                }

                // ConstructionSubmit - POST /construction/submit
                &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_SUBMIT) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_construction_submit_request: Option<models::ConstructionSubmitRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_construction_submit_request) => param_construction_submit_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ConstructionSubmitRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ConstructionSubmitRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_construction_submit_request = match param_construction_submit_request {
                                    Some(param_construction_submit_request) => param_construction_submit_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ConstructionSubmitRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ConstructionSubmitRequest")),
                                };

                                let result = api_impl.construction_submit(
                                            param_construction_submit_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                ConstructionSubmitResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_SUBMIT_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ConstructionSubmitResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for CONSTRUCTION_SUBMIT_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ConstructionSubmitRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ConstructionSubmitRequest")),
                        }
                }

                // EventsBlocks - POST /events/blocks
                &hyper::Method::POST if path.matched(paths::ID_EVENTS_BLOCKS) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_events_blocks_request: Option<models::EventsBlocksRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_events_blocks_request) => param_events_blocks_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter EventsBlocksRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter EventsBlocksRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_events_blocks_request = match param_events_blocks_request {
                                    Some(param_events_blocks_request) => param_events_blocks_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter EventsBlocksRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter EventsBlocksRequest")),
                                };

                                let result = api_impl.events_blocks(
                                            param_events_blocks_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                EventsBlocksResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for EVENTS_BLOCKS_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                EventsBlocksResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for EVENTS_BLOCKS_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter EventsBlocksRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter EventsBlocksRequest")),
                        }
                }

                // Mempool - POST /mempool
                &hyper::Method::POST if path.matched(paths::ID_MEMPOOL) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_network_request: Option<models::NetworkRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_network_request) => param_network_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter NetworkRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter NetworkRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_network_request = match param_network_request {
                                    Some(param_network_request) => param_network_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter NetworkRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter NetworkRequest")),
                                };

                                let result = api_impl.mempool(
                                            param_network_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                MempoolResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MEMPOOL_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                MempoolResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MEMPOOL_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter NetworkRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter NetworkRequest")),
                        }
                }

                // MempoolTransaction - POST /mempool/transaction
                &hyper::Method::POST if path.matched(paths::ID_MEMPOOL_TRANSACTION) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_mempool_transaction_request: Option<models::MempoolTransactionRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_mempool_transaction_request) => param_mempool_transaction_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter MempoolTransactionRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter MempoolTransactionRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_mempool_transaction_request = match param_mempool_transaction_request {
                                    Some(param_mempool_transaction_request) => param_mempool_transaction_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter MempoolTransactionRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter MempoolTransactionRequest")),
                                };

                                let result = api_impl.mempool_transaction(
                                            param_mempool_transaction_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                MempoolTransactionResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MEMPOOL_TRANSACTION_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                MempoolTransactionResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MEMPOOL_TRANSACTION_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter MempoolTransactionRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter MempoolTransactionRequest")),
                        }
                }

                // NetworkList - POST /network/list
                &hyper::Method::POST if path.matched(paths::ID_NETWORK_LIST) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_metadata_request: Option<models::MetadataRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_metadata_request) => param_metadata_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter MetadataRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter MetadataRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_metadata_request = match param_metadata_request {
                                    Some(param_metadata_request) => param_metadata_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter MetadataRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter MetadataRequest")),
                                };

                                let result = api_impl.network_list(
                                            param_metadata_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                NetworkListResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NETWORK_LIST_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NetworkListResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NETWORK_LIST_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter MetadataRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter MetadataRequest")),
                        }
                }

                // NetworkOptions - POST /network/options
                &hyper::Method::POST if path.matched(paths::ID_NETWORK_OPTIONS) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_network_request: Option<models::NetworkRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_network_request) => param_network_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter NetworkRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter NetworkRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_network_request = match param_network_request {
                                    Some(param_network_request) => param_network_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter NetworkRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter NetworkRequest")),
                                };

                                let result = api_impl.network_options(
                                            param_network_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                NetworkOptionsResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NETWORK_OPTIONS_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NetworkOptionsResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NETWORK_OPTIONS_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter NetworkRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter NetworkRequest")),
                        }
                }

                // NetworkStatus - POST /network/status
                &hyper::Method::POST if path.matched(paths::ID_NETWORK_STATUS) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_network_request: Option<models::NetworkRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_network_request) => param_network_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter NetworkRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter NetworkRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_network_request = match param_network_request {
                                    Some(param_network_request) => param_network_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter NetworkRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter NetworkRequest")),
                                };

                                let result = api_impl.network_status(
                                            param_network_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                NetworkStatusResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NETWORK_STATUS_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NetworkStatusResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NETWORK_STATUS_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter NetworkRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter NetworkRequest")),
                        }
                }

                // SearchTransactions - POST /search/transactions
                &hyper::Method::POST if path.matched(paths::ID_SEARCH_TRANSACTIONS) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_search_transactions_request: Option<models::SearchTransactionsRequest> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_search_transactions_request) => param_search_transactions_request,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter SearchTransactionsRequest - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter SearchTransactionsRequest due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_search_transactions_request = match param_search_transactions_request {
                                    Some(param_search_transactions_request) => param_search_transactions_request,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter SearchTransactionsRequest"))
                                                        .expect("Unable to create Bad Request response for missing body parameter SearchTransactionsRequest")),
                                };

                                let result = api_impl.search_transactions(
                                            param_search_transactions_request,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().to_string().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                SearchTransactionsResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SEARCH_TRANSACTIONS_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SearchTransactionsResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SEARCH_TRANSACTIONS_UNEXPECTED_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter SearchTransactionsRequest: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SearchTransactionsRequest")),
                        }
                }

                _ if path.matched(paths::ID_ACCOUNT_BALANCE) => method_not_allowed(),
                _ if path.matched(paths::ID_ACCOUNT_COINS) => method_not_allowed(),
                _ if path.matched(paths::ID_BLOCK) => method_not_allowed(),
                _ if path.matched(paths::ID_BLOCK_TRANSACTION) => method_not_allowed(),
                _ if path.matched(paths::ID_CALL) => method_not_allowed(),
                _ if path.matched(paths::ID_CONSTRUCTION_COMBINE) => method_not_allowed(),
                _ if path.matched(paths::ID_CONSTRUCTION_DERIVE) => method_not_allowed(),
                _ if path.matched(paths::ID_CONSTRUCTION_HASH) => method_not_allowed(),
                _ if path.matched(paths::ID_CONSTRUCTION_METADATA) => method_not_allowed(),
                _ if path.matched(paths::ID_CONSTRUCTION_PARSE) => method_not_allowed(),
                _ if path.matched(paths::ID_CONSTRUCTION_PAYLOADS) => method_not_allowed(),
                _ if path.matched(paths::ID_CONSTRUCTION_PREPROCESS) => method_not_allowed(),
                _ if path.matched(paths::ID_CONSTRUCTION_SUBMIT) => method_not_allowed(),
                _ if path.matched(paths::ID_EVENTS_BLOCKS) => method_not_allowed(),
                _ if path.matched(paths::ID_MEMPOOL) => method_not_allowed(),
                _ if path.matched(paths::ID_MEMPOOL_TRANSACTION) => method_not_allowed(),
                _ if path.matched(paths::ID_NETWORK_LIST) => method_not_allowed(),
                _ if path.matched(paths::ID_NETWORK_OPTIONS) => method_not_allowed(),
                _ if path.matched(paths::ID_NETWORK_STATUS) => method_not_allowed(),
                _ if path.matched(paths::ID_SEARCH_TRANSACTIONS) => method_not_allowed(),
                _ => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .expect("Unable to create Not Found response")),
            }
        }
        Box::pin(run(self.api_impl.clone(), req))
    }
}

/// Request parser for `Api`.
pub struct ApiRequestParser;
impl<T> RequestParser<T> for ApiRequestParser {
    fn parse_operation_id(request: &Request<T>) -> Option<&'static str> {
        let path = paths::GLOBAL_REGEX_SET.matches(request.uri().path());
        match request.method() {
            // AccountBalance - POST /account/balance
            &hyper::Method::POST if path.matched(paths::ID_ACCOUNT_BALANCE) => {
                Some("AccountBalance")
            }
            // AccountCoins - POST /account/coins
            &hyper::Method::POST if path.matched(paths::ID_ACCOUNT_COINS) => Some("AccountCoins"),
            // Block - POST /block
            &hyper::Method::POST if path.matched(paths::ID_BLOCK) => Some("Block"),
            // BlockTransaction - POST /block/transaction
            &hyper::Method::POST if path.matched(paths::ID_BLOCK_TRANSACTION) => {
                Some("BlockTransaction")
            }
            // Call - POST /call
            &hyper::Method::POST if path.matched(paths::ID_CALL) => Some("Call"),
            // ConstructionCombine - POST /construction/combine
            &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_COMBINE) => {
                Some("ConstructionCombine")
            }
            // ConstructionDerive - POST /construction/derive
            &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_DERIVE) => {
                Some("ConstructionDerive")
            }
            // ConstructionHash - POST /construction/hash
            &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_HASH) => {
                Some("ConstructionHash")
            }
            // ConstructionMetadata - POST /construction/metadata
            &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_METADATA) => {
                Some("ConstructionMetadata")
            }
            // ConstructionParse - POST /construction/parse
            &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_PARSE) => {
                Some("ConstructionParse")
            }
            // ConstructionPayloads - POST /construction/payloads
            &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_PAYLOADS) => {
                Some("ConstructionPayloads")
            }
            // ConstructionPreprocess - POST /construction/preprocess
            &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_PREPROCESS) => {
                Some("ConstructionPreprocess")
            }
            // ConstructionSubmit - POST /construction/submit
            &hyper::Method::POST if path.matched(paths::ID_CONSTRUCTION_SUBMIT) => {
                Some("ConstructionSubmit")
            }
            // EventsBlocks - POST /events/blocks
            &hyper::Method::POST if path.matched(paths::ID_EVENTS_BLOCKS) => Some("EventsBlocks"),
            // Mempool - POST /mempool
            &hyper::Method::POST if path.matched(paths::ID_MEMPOOL) => Some("Mempool"),
            // MempoolTransaction - POST /mempool/transaction
            &hyper::Method::POST if path.matched(paths::ID_MEMPOOL_TRANSACTION) => {
                Some("MempoolTransaction")
            }
            // NetworkList - POST /network/list
            &hyper::Method::POST if path.matched(paths::ID_NETWORK_LIST) => Some("NetworkList"),
            // NetworkOptions - POST /network/options
            &hyper::Method::POST if path.matched(paths::ID_NETWORK_OPTIONS) => {
                Some("NetworkOptions")
            }
            // NetworkStatus - POST /network/status
            &hyper::Method::POST if path.matched(paths::ID_NETWORK_STATUS) => Some("NetworkStatus"),
            // SearchTransactions - POST /search/transactions
            &hyper::Method::POST if path.matched(paths::ID_SEARCH_TRANSACTIONS) => {
                Some("SearchTransactions")
            }
            _ => None,
        }
    }
}
