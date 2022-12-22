use crate::types::{
    AccountBalanceRequest, AccountBalanceResponse, AccountCoinsRequest, AccountCoinsResponse,
    AccountFaucetRequest, BlockRequest, BlockResponse, BlockTransactionRequest,
    BlockTransactionResponse, ConstructionCombineRequest, ConstructionCombineResponse,
    ConstructionDeriveRequest, ConstructionDeriveResponse, ConstructionHashRequest,
    ConstructionMetadataRequest, ConstructionMetadataResponse, ConstructionParseRequest,
    ConstructionParseResponse, ConstructionPayloadsRequest, ConstructionPayloadsResponse,
    ConstructionPreprocessRequest, ConstructionPreprocessResponse, ConstructionSubmitRequest,
    EventsBlocksRequest, EventsBlocksResponse, MempoolResponse, MempoolTransactionRequest,
    MempoolTransactionResponse, MetadataRequest, NetworkListResponse, NetworkOptionsResponse,
    NetworkRequest, NetworkStatusResponse, RuntimeCallRequest, SearchTransactionsRequest,
    SearchTransactionsResponse, TransactionIdentifierResponse,
};
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

/// The client struct to interface with a rosetta endpoint.
#[derive(Clone)]
pub struct Client {
    /// The http client.
    http: surf::Client,
}

impl Client {
    /// `url` should have the form `http[s]://hostname:port`.
    pub fn new(url: &str) -> Result<Self> {
        let http = surf::Config::new().set_base_url(url.parse()?).try_into()?;
        Ok(Self { http })
    }

    /// Makes a POST request to the rosetta endpoint.
    async fn post<Q: Serialize, R: DeserializeOwned>(&self, path: &str, request: &Q) -> Result<R> {
        let mut res = self
            .http
            .post(path)
            .body_json(request)
            .map_err(|e| e.into_inner())?
            .send()
            .await
            .map_err(|e| e.into_inner())?;
        match res.status() as u16 {
            200 => Ok(res.body_json().await.map_err(|e| e.into_inner())?),
            404 => anyhow::bail!("unsupported endpoint {}", path),
            500 => {
                let error: crate::types::Error =
                    res.body_json().await.map_err(|e| e.into_inner())?;
                log::error!("{:#?}", error);
                Err(error.into())
            }
            _ => anyhow::bail!("unexpected status code {}", res.status()),
        }
    }

    /// Make a call to the /network/list endpoint.
    pub async fn network_list(&self) -> Result<Vec<NetworkIdentifier>> {
        let request = MetadataRequest { metadata: None };
        let response: NetworkListResponse = self.post("/network/list", &request).await?;
        Ok(response.network_identifiers)
    }

    /// Make a call to the /network/options endpoint.
    pub async fn network_options(
        &self,
        network_identifier: NetworkIdentifier,
    ) -> Result<NetworkOptionsResponse> {
        let request = NetworkRequest {
            network_identifier,
            metadata: None,
        };
        self.post("/network/options", &request).await
    }

    /// Make a call to the /network/status endpoint.
    pub async fn network_status(
        &self,
        network_identifier: NetworkIdentifier,
    ) -> Result<NetworkStatusResponse> {
        let request = NetworkRequest {
            network_identifier,
            metadata: None,
        };
        self.post("/network/status", &request).await
    }

    /// Make a call to the /account/balance endpoint.
    pub async fn account_balance(
        &self,
        request: &AccountBalanceRequest,
    ) -> Result<AccountBalanceResponse> {
        self.post("/account/balance", &request).await
    }

    /// Make a call to the /account/coins endpoint.
    pub async fn account_coins(
        &self,
        request: &AccountCoinsRequest,
    ) -> Result<AccountCoinsResponse> {
        self.post("/account/coins", &request).await
    }

    /// Make a call to the /account/faucet endpoint.
    pub async fn account_faucet(
        &self,
        request: &AccountFaucetRequest,
    ) -> Result<TransactionIdentifierResponse> {
        self.post("/account/faucet", &request).await
    }

    /// Make a call to the /block endpoint.
    pub async fn block(&self, request: &BlockRequest) -> Result<BlockResponse> {
        self.post("/block", &request).await
    }

    /// Make a call to the /block/transaction endpoint.
    pub async fn block_transaction(
        &self,
        request: &BlockTransactionRequest,
    ) -> Result<BlockTransactionResponse> {
        self.post("/block/transaction", &request).await
    }

    /// Make a call to the /mempool endpoint.
    pub async fn mempool(&self, network_identifier: NetworkIdentifier) -> Result<MempoolResponse> {
        let request = NetworkRequest {
            network_identifier,
            metadata: None,
        };
        self.post("/mempool", &request).await
    }

    /// Make a call to the /mempool/transaction endpoint.
    pub async fn mempool_transaction(
        &self,
        request: &MempoolTransactionRequest,
    ) -> Result<MempoolTransactionResponse> {
        self.post("/mempool/transaction", &request).await
    }

    /// Make a call to the /construction/combine endpoint.
    pub async fn construction_combine(
        &self,
        request: &ConstructionCombineRequest,
    ) -> Result<ConstructionCombineResponse> {
        self.post("/construction/combine", &request).await
    }

    /// Make a call to the /construction/derive endpoint.
    pub async fn construction_derive(
        &self,
        request: &ConstructionDeriveRequest,
    ) -> Result<ConstructionDeriveResponse> {
        self.post("/construction/derive", &request).await
    }

    /// Make a call to the /construction/hash endpoint.
    pub async fn construction_hash(
        &self,
        request: &ConstructionHashRequest,
    ) -> Result<TransactionIdentifierResponse> {
        self.post("/construction/hash", &request).await
    }

    /// Make a call to the /construction/metadata endpoint.
    pub async fn construction_metadata(
        &self,
        request: &ConstructionMetadataRequest,
    ) -> Result<ConstructionMetadataResponse> {
        self.post("/construction/metadata", &request).await
    }

    /// Make a call to the /construction/parse endpoint.
    pub async fn construction_parse(
        &self,
        request: &ConstructionParseRequest,
    ) -> Result<ConstructionParseResponse> {
        self.post("/construction/parse", &request).await
    }

    /// Make a call to the /construction/payloads endpoint.
    pub async fn construction_payloads(
        &self,
        request: &ConstructionPayloadsRequest,
    ) -> Result<ConstructionPayloadsResponse> {
        self.post("/construction/payloads", &request).await
    }

    /// Make a call to the /construction/preprocess endpoint.
    pub async fn construction_preprocess(
        &self,
        request: &ConstructionPreprocessRequest,
    ) -> Result<ConstructionPreprocessResponse> {
        self.post("/construction/preprocess", &request).await
    }

    /// Make a call to the /construction/submit endpoint.
    pub async fn construction_submit(
        &self,
        request: &ConstructionSubmitRequest,
    ) -> Result<TransactionIdentifierResponse> {
        self.post("/construction/submit", &request).await
    }

    /// Make a call to the /events/blocks endpoint.
    pub async fn events_blocks(
        &self,
        request: &EventsBlocksRequest,
    ) -> Result<EventsBlocksResponse> {
        self.post("/events/blocks", &request).await
    }

    /// Make a call to the /search/transactions endpoint.
    pub async fn search_transactions(
        &self,
        request: &SearchTransactionsRequest,
    ) -> Result<SearchTransactionsResponse> {
        self.post("/search/transactions", &request).await
    }

    /// Make a call to the /runtime/call endpoint.
    pub async fn runtime_call(
        &self,
        request: &RuntimeCallRequest,
    ) -> Result<SearchTransactionsResponse> {
        self.post("/runtime/call", &request).await
    }
}
