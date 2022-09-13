//! Main library entry point for openapi_client implementation.

#![allow(unused_imports)]

use async_trait::async_trait;
use futures::{future, Stream, StreamExt, TryFutureExt, TryStreamExt};
use hyper::server::conn::Http;
use hyper::service::Service;
use log::info;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use swagger::{Has, XSpanIdString};
use tokio::net::TcpListener;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{Ssl, SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use openapi_client::models;

/// Builds an SSL implementation for Simple HTTPS from some hard-coded file names
pub async fn create(addr: &str, https: bool) {
    let addr = addr.parse().expect("Failed to parse bind address");

    let server = Server::new();

    let service = MakeService::new(server);

    let service = MakeAllowAllAuthenticator::new(service, "cosmo");

    let mut service =
        openapi_client::server::context::MakeAddContext::<_, EmptyContext>::new(service);

    if https {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
        {
            unimplemented!("SSL is not implemented for the examples on MacOS, Windows or iOS");
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
        {
            let mut ssl = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls())
                .expect("Failed to create SSL Acceptor");

            // Server authentication
            ssl.set_private_key_file("examples/server-key.pem", SslFiletype::PEM)
                .expect("Failed to set private key");
            ssl.set_certificate_chain_file("examples/server-chain.pem")
                .expect("Failed to set certificate chain");
            ssl.check_private_key()
                .expect("Failed to check private key");

            let tls_acceptor = ssl.build();
            let tcp_listener = TcpListener::bind(&addr).await.unwrap();

            loop {
                if let Ok((tcp, _)) = tcp_listener.accept().await {
                    let ssl = Ssl::new(tls_acceptor.context()).unwrap();
                    let addr = tcp.peer_addr().expect("Unable to get remote address");
                    let service = service.call(addr);

                    tokio::spawn(async move {
                        let tls = tokio_openssl::SslStream::new(ssl, tcp).map_err(|_| ())?;
                        let service = service.await.map_err(|_| ())?;

                        Http::new()
                            .serve_connection(tls, service)
                            .await
                            .map_err(|_| ())
                    });
                }
            }
        }
    } else {
        // Using HTTP
        hyper::server::Server::bind(&addr)
            .serve(service)
            .await
            .unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new() -> Self {
        Server {
            marker: PhantomData,
        }
    }
}

use openapi_client::server::MakeService;
use openapi_client::{
    AccountBalanceResponse, AccountCoinsResponse, Api, BlockResponse, BlockTransactionResponse,
    CallResponse, ConstructionCombineResponse, ConstructionDeriveResponse,
    ConstructionHashResponse, ConstructionMetadataResponse, ConstructionParseResponse,
    ConstructionPayloadsResponse, ConstructionPreprocessResponse, ConstructionSubmitResponse,
    EventsBlocksResponse, MempoolResponse, MempoolTransactionResponse, NetworkListResponse,
    NetworkOptionsResponse, NetworkStatusResponse, SearchTransactionsResponse,
};
use std::error::Error;
use swagger::ApiError;

#[async_trait]
impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString> + Send + Sync,
{
    /// Get an Account's Balance
    async fn account_balance(
        &self,
        account_balance_request: models::AccountBalanceRequest,
        context: &C,
    ) -> Result<AccountBalanceResponse, ApiError> {
        let context = context.clone();
        info!(
            "account_balance({:?}) - X-Span-ID: {:?}",
            account_balance_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Get an Account's Unspent Coins
    async fn account_coins(
        &self,
        account_coins_request: models::AccountCoinsRequest,
        context: &C,
    ) -> Result<AccountCoinsResponse, ApiError> {
        let context = context.clone();
        info!(
            "account_coins({:?}) - X-Span-ID: {:?}",
            account_coins_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Get a Block
    async fn block(
        &self,
        block_request: models::BlockRequest,
        context: &C,
    ) -> Result<BlockResponse, ApiError> {
        let context = context.clone();
        info!(
            "block({:?}) - X-Span-ID: {:?}",
            block_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Get a Block Transaction
    async fn block_transaction(
        &self,
        block_transaction_request: models::BlockTransactionRequest,
        context: &C,
    ) -> Result<BlockTransactionResponse, ApiError> {
        let context = context.clone();
        info!(
            "block_transaction({:?}) - X-Span-ID: {:?}",
            block_transaction_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Make a Network-Specific Procedure Call
    async fn call(
        &self,
        call_request: models::CallRequest,
        context: &C,
    ) -> Result<CallResponse, ApiError> {
        let context = context.clone();
        info!(
            "call({:?}) - X-Span-ID: {:?}",
            call_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Create Network Transaction from Signatures
    async fn construction_combine(
        &self,
        construction_combine_request: models::ConstructionCombineRequest,
        context: &C,
    ) -> Result<ConstructionCombineResponse, ApiError> {
        let context = context.clone();
        info!(
            "construction_combine({:?}) - X-Span-ID: {:?}",
            construction_combine_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Derive an AccountIdentifier from a PublicKey
    async fn construction_derive(
        &self,
        construction_derive_request: models::ConstructionDeriveRequest,
        context: &C,
    ) -> Result<ConstructionDeriveResponse, ApiError> {
        let context = context.clone();
        info!(
            "construction_derive({:?}) - X-Span-ID: {:?}",
            construction_derive_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Get the Hash of a Signed Transaction
    async fn construction_hash(
        &self,
        construction_hash_request: models::ConstructionHashRequest,
        context: &C,
    ) -> Result<ConstructionHashResponse, ApiError> {
        let context = context.clone();
        info!(
            "construction_hash({:?}) - X-Span-ID: {:?}",
            construction_hash_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Get Metadata for Transaction Construction
    async fn construction_metadata(
        &self,
        construction_metadata_request: models::ConstructionMetadataRequest,
        context: &C,
    ) -> Result<ConstructionMetadataResponse, ApiError> {
        let context = context.clone();
        info!(
            "construction_metadata({:?}) - X-Span-ID: {:?}",
            construction_metadata_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Parse a Transaction
    async fn construction_parse(
        &self,
        construction_parse_request: models::ConstructionParseRequest,
        context: &C,
    ) -> Result<ConstructionParseResponse, ApiError> {
        let context = context.clone();
        info!(
            "construction_parse({:?}) - X-Span-ID: {:?}",
            construction_parse_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Generate an Unsigned Transaction and Signing Payloads
    async fn construction_payloads(
        &self,
        construction_payloads_request: models::ConstructionPayloadsRequest,
        context: &C,
    ) -> Result<ConstructionPayloadsResponse, ApiError> {
        let context = context.clone();
        info!(
            "construction_payloads({:?}) - X-Span-ID: {:?}",
            construction_payloads_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Create a Request to Fetch Metadata
    async fn construction_preprocess(
        &self,
        construction_preprocess_request: models::ConstructionPreprocessRequest,
        context: &C,
    ) -> Result<ConstructionPreprocessResponse, ApiError> {
        let context = context.clone();
        info!(
            "construction_preprocess({:?}) - X-Span-ID: {:?}",
            construction_preprocess_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Submit a Signed Transaction
    async fn construction_submit(
        &self,
        construction_submit_request: models::ConstructionSubmitRequest,
        context: &C,
    ) -> Result<ConstructionSubmitResponse, ApiError> {
        let context = context.clone();
        info!(
            "construction_submit({:?}) - X-Span-ID: {:?}",
            construction_submit_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// [INDEXER] Get a range of BlockEvents
    async fn events_blocks(
        &self,
        events_blocks_request: models::EventsBlocksRequest,
        context: &C,
    ) -> Result<EventsBlocksResponse, ApiError> {
        let context = context.clone();
        info!(
            "events_blocks({:?}) - X-Span-ID: {:?}",
            events_blocks_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Get All Mempool Transactions
    async fn mempool(
        &self,
        network_request: models::NetworkRequest,
        context: &C,
    ) -> Result<MempoolResponse, ApiError> {
        let context = context.clone();
        info!(
            "mempool({:?}) - X-Span-ID: {:?}",
            network_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Get a Mempool Transaction
    async fn mempool_transaction(
        &self,
        mempool_transaction_request: models::MempoolTransactionRequest,
        context: &C,
    ) -> Result<MempoolTransactionResponse, ApiError> {
        let context = context.clone();
        info!(
            "mempool_transaction({:?}) - X-Span-ID: {:?}",
            mempool_transaction_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Get List of Available Networks
    async fn network_list(
        &self,
        metadata_request: models::MetadataRequest,
        context: &C,
    ) -> Result<NetworkListResponse, ApiError> {
        let context = context.clone();
        info!(
            "network_list({:?}) - X-Span-ID: {:?}",
            metadata_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Get Network Options
    async fn network_options(
        &self,
        network_request: models::NetworkRequest,
        context: &C,
    ) -> Result<NetworkOptionsResponse, ApiError> {
        let context = context.clone();
        info!(
            "network_options({:?}) - X-Span-ID: {:?}",
            network_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Get Network Status
    async fn network_status(
        &self,
        network_request: models::NetworkRequest,
        context: &C,
    ) -> Result<NetworkStatusResponse, ApiError> {
        let context = context.clone();
        info!(
            "network_status({:?}) - X-Span-ID: {:?}",
            network_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// [INDEXER] Search for Transactions
    async fn search_transactions(
        &self,
        search_transactions_request: models::SearchTransactionsRequest,
        context: &C,
    ) -> Result<SearchTransactionsResponse, ApiError> {
        let context = context.clone();
        info!(
            "search_transactions({:?}) - X-Span-ID: {:?}",
            search_transactions_request,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }
}
