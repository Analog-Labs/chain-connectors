use crate::{error::EthError, extension::impl_client_trait, params::EthRpcParams};
use async_trait::async_trait;
use ethers::providers::JsonRpcClient;
use jsonrpsee::core::client::ClientT;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::{Debug, Formatter},
    ops::{Deref, DerefMut},
};

/// Adapter for [`jsonrpsee::core::client::ClientT`] to [`ethers::providers::JsonRpcClient`].
#[repr(transparent)]
pub struct EthClientAdapter<C> {
    pub(crate) client: C,
}

impl<C> EthClientAdapter<C>
where
    C: ClientT + Debug + Send + Sync,
{
    pub const fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C> AsRef<C> for EthClientAdapter<C> {
    fn as_ref(&self) -> &C {
        &self.client
    }
}

impl_client_trait!(EthClientAdapter<C> where C: ClientT + Debug + Send + Sync);

impl<C> Debug for EthClientAdapter<C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientAdapter").field("client", &self.client).finish()
    }
}

impl<C> Clone for EthClientAdapter<C>
where
    C: Clone,
{
    fn clone(&self) -> Self {
        Self { client: self.client.clone() }
    }
}

impl<C> AsMut<C> for EthClientAdapter<C> {
    fn as_mut(&mut self) -> &mut C {
        &mut self.client
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
    type Error = EthError;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        let params = EthRpcParams::from_serializable(&params)?;
        ClientT::request::<R, EthRpcParams>(&self.client, method, params)
            .await
            .map_err(EthError::from)
    }
}
