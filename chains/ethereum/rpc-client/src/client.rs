use crate::{error::Error, params::RpcParams};
use async_trait::async_trait;
use ethers::providers::JsonRpcClient;
use jsonrpsee::core::{
    client::{BatchResponse, ClientT},
    params::BatchRequestBuilder,
    traits::ToRpcParams,
    Error as JsonRpseeError,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

// Adapter Client for JsonRpcClient trait
#[repr(transparent)]
pub struct EthClientAdapter<C> {
    pub(crate) client: C,
}

impl<C> EthClientAdapter<C>
where
    C: ClientT + Debug + Send + Sync,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn eth_request<R>(&self, method: &str, params: RpcParams) -> Result<R, Error>
    where
        R: DeserializeOwned + Send,
    {
        ClientT::request::<R, RpcParams>(&self.client, method, params)
            .await
            .map_err(Error::from)
    }
}

impl<C> Debug for EthClientAdapter<C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientAdapter")
            .field("client", &self.client)
            .finish()
    }
}

impl<C> Clone for EthClientAdapter<C>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}

impl<C> AsMut<C> for EthClientAdapter<C> {
    fn as_mut(&mut self) -> &mut C {
        &mut self.client
    }
}

impl<C> AsRef<C> for EthClientAdapter<C> {
    fn as_ref(&self) -> &C {
        &self.client
    }
}

impl<C> Deref for EthClientAdapter<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<C> DerefMut for EthClientAdapter<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

#[async_trait]
impl<C> JsonRpcClient for EthClientAdapter<C>
where
    C: ClientT + Debug + Send + Sync,
{
    type Error = Error;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        let params = RpcParams::from_serializable(&params)?;
        ClientT::request::<R, RpcParams>(&self.client, method, params)
            .await
            .map_err(Error::from)
    }
}

#[async_trait]
impl<C> ClientT for EthClientAdapter<C>
where
    C: ClientT + Debug + Send + Sync,
{
    async fn notification<Params>(&self, method: &str, params: Params) -> Result<(), JsonRpseeError>
    where
        Params: ToRpcParams + Send,
    {
        ClientT::notification(self, method, params).await
    }

    async fn request<R, Params>(&self, method: &str, params: Params) -> Result<R, JsonRpseeError>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        ClientT::request(self, method, params).await
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, JsonRpseeError>
    where
        R: DeserializeOwned + Debug + 'a,
    {
        ClientT::batch_request(self, batch).await
    }
}
