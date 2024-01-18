use crate::ws::reconnect::{AutoReconnectClient, Reconnect};
use futures::stream::{StreamExt, TryStreamExt};
use jsonrpsee::{
    core::{
        client::{ClientT, SubscriptionClientT, SubscriptionKind},
        traits::ToRpcParams,
    },
    types::SubscriptionId,
};
use subxt::{
    backend::rpc::{RawRpcFuture, RawRpcSubscription, RawValue, RpcClientT},
    error::RpcError,
};

#[derive(Clone)]
pub struct Params(Option<Box<RawValue>>);

impl Params {
    pub fn new<P>(params: P) -> Result<Self, serde_json::Error>
    where
        P: ToRpcParams,
    {
        let params = params.to_rpc_params()?;
        Ok(Self(params))
    }
}

impl ToRpcParams for Params {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        Ok(self.0)
    }
}

impl<T> RpcClientT for AutoReconnectClient<T>
where
    T: Reconnect,
    T::Client: SubscriptionClientT,
{
    fn request_raw<'a>(
        &'a self,
        method: &'a str,
        params: Option<Box<RawValue>>,
    ) -> RawRpcFuture<'a, Box<RawValue>> {
        Box::pin(async move {
            let res = ClientT::request(self, method, Params(params))
                .await
                .map_err(|e| RpcError::ClientError(Box::new(e)))?;
            Ok(res)
        })
    }

    fn subscribe_raw<'a>(
        &'a self,
        sub: &'a str,
        params: Option<Box<RawValue>>,
        unsub: &'a str,
    ) -> RawRpcFuture<'a, RawRpcSubscription> {
        Box::pin(async move {
            let stream = SubscriptionClientT::subscribe::<Box<RawValue>, _>(
                self,
                sub,
                Params(params),
                unsub,
            )
            .await
            .map_err(|e| RpcError::ClientError(Box::new(e)))?;

            let id = match stream.kind() {
                SubscriptionKind::Subscription(SubscriptionId::Str(id)) => {
                    Some(id.clone().into_owned())
                },
                _ => None,
            };

            let stream = stream.map_err(|e| RpcError::ClientError(Box::new(e))).boxed();
            Ok(RawRpcSubscription { stream, id })
        })
    }
}
