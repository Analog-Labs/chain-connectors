use super::jsonrpsee_client::Params as RpcParams;
use async_trait::async_trait;
use jsonrpsee::core::client::BatchResponse;
use jsonrpsee::core::params::BatchRequestBuilder;
use jsonrpsee::core::{client::ClientT, traits::ToRpcParams, Error};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub trait Reconnect<C>: 'static + Send + Sync
where
    C: ClientT + Send + Sync,
{
    type ClientRef: AsRef<C> + Send + Sync;

    type ReadyFuture<'a>: Future<Output = Result<Self::ClientRef, Error>> + 'a + Send + Sync
    where
        Self: 'a;

    type ReconnectFuture<'a>: Future<Output = Result<Option<Self::ClientRef>, Error>>
        + 'a
        + Send
        + Sync
    where
        Self: 'a;

    fn client(&self) -> Self::ReadyFuture<'_>;

    fn reconnect(&self) -> Self::ReconnectFuture<'_>;
}

pub struct AutoReconnectClient<C, T> {
    _marker: PhantomData<C>,
    client: T,
}

impl<C, T> Deref for AutoReconnectClient<C, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<C, T> DerefMut for AutoReconnectClient<C, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

#[async_trait]
impl<C, T> ClientT for AutoReconnectClient<C, T>
where
    C: ClientT + Send + Sync,
    T: Reconnect<C>,
{
    async fn notification<Params>(&self, method: &str, params: Params) -> Result<(), Error>
    where
        Params: ToRpcParams + Send,
    {
        let client = Reconnect::client(&self.client).await?;
        let params = RpcParams::new(params)?;
        match ClientT::notification(client.as_ref(), method, params.clone()).await {
            Ok(r) => Ok(r),
            Err(Error::RestartNeeded(reason)) => {
                if let Some(client) = Reconnect::reconnect(&self.client).await? {
                    ClientT::notification(client.as_ref(), method, params).await
                } else {
                    Err(Error::RestartNeeded(reason))
                }
            }
            Err(error) => Err(error),
        }
    }

    async fn request<R, Params>(&self, method: &str, params: Params) -> Result<R, Error>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        let (error, params) = {
            let client = Reconnect::client(&self.client).await?;
            let params = RpcParams::new(params)?;
            match ClientT::request::<R, _>(client.as_ref(), method, params.clone()).await {
                Ok(r) => return Ok(r),
                Err(error) => (error, params),
            }
        };

        match error {
            Error::RestartNeeded(reason) => {
                if let Some(client) = Reconnect::reconnect(&self.client).await? {
                    ClientT::request::<R, _>(client.as_ref(), method, params).await
                } else {
                    Err(Error::RestartNeeded(reason))
                }
            }
            error => Err(error),
        }
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, Error>
    where
        R: DeserializeOwned + Debug + 'a,
    {
        let error = {
            let client = Reconnect::client(&self.client).await?;
            match ClientT::batch_request(client.as_ref(), batch.clone()).await {
                Ok(r) => return Ok(r),
                Err(error) => error,
            }
        };

        match error {
            Error::RestartNeeded(reason) => {
                if let Some(client) = Reconnect::reconnect(&self.client).await? {
                    ClientT::batch_request(client.as_ref(), batch).await
                } else {
                    Err(Error::RestartNeeded(reason))
                }
            }
            error => Err(error),
        }
    }
}

// #[async_trait]
// impl<C, T> SubscriptionClientT for AutoReconnectClient<T>
// where
//     C: SubscriptionClientT,
//     T: Reconnect<C>,
// {
//     async fn subscribe<'a, Notif, Params>(
//         &self,
//         subscribe_method: &'a str,
//         params: Params,
//         unsubscribe_method: &'a str,
//     ) -> Result<Subscription<Notif>, Error>
//     where
//         Params: ToRpcParams + Send,
//         Notif: DeserializeOwned,
//     {
//         let client = Reconnect::client(&self.client).await?.deref();
//         let params = params.to_rpc_params()?;
//         let result = SubscriptionClientT::subscribe(
//             &self.client,
//             subscribe_method,
//             params.clone(),
//             unsubscribe_method,
//         )
//         .await;
//         if let Err(Error::RestartNeeded(_)) = result {
//             self.client.reconnect().await?;
//             return SubscriptionClientT::subscribe(
//                 &self.client,
//                 subscribe_method,
//                 params,
//                 unsubscribe_method,
//             )
//             .await;
//         }
//         result
//     }
//
//     async fn subscribe_to_method<Notif>(&self, method: &str) -> Result<Subscription<Notif>, Error>
//     where
//         Notif: DeserializeOwned,
//     {
//         let result = SubscriptionClientT::subscribe_to_method(&self.client, method).await;
//         if let Err(Error::RestartNeeded(_)) = result {
//             self.client.reconnect().await?;
//             return SubscriptionClientT::subscribe_to_method(&self.client, method).await;
//         }
//         result
//     }
// }
