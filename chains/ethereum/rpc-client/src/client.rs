use crate::{error::Error, params::RpcParams};
use async_trait::async_trait;
use ethers::providers::JsonRpcClient;
use jsonrpsee::core::client::ClientT;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

// Websocket Client that supports reconnecting
pub struct ClientAdapter<C> {
    pub(crate) client: C,
}

impl<C> ClientAdapter<C>
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

impl<C> Debug for ClientAdapter<C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientAdapter")
            .field("client", &self.client)
            .finish()
    }
}

impl<C> Clone for ClientAdapter<C>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
        }
    }
}

impl<C> AsMut<C> for ClientAdapter<C> {
    fn as_mut(&mut self) -> &mut C {
        &mut self.client
    }
}

impl<C> AsRef<C> for ClientAdapter<C> {
    fn as_ref(&self) -> &C {
        &self.client
    }
}

impl<C> Deref for ClientAdapter<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<C> DerefMut for ClientAdapter<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

#[async_trait]
impl<C> JsonRpcClient for ClientAdapter<C>
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
