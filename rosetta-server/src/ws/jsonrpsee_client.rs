use crate::ws::reconnect::{AutoReconnectClient, Reconnect};
use futures::stream::{StreamExt, TryStreamExt};
use jsonrpsee::{
    core::{
        client::{ClientT, SubscriptionClientT, SubscriptionKind},
        traits::ToRpcParams,
        Error as JsonRpseeError,
    },
    types::SubscriptionId,
};
use std::fmt::{Debug, Formatter};
use subxt::{
    error::RpcError,
    rpc::{RawValue, RpcClientT, RpcFuture, RpcSubscription},
};

#[derive(Clone)]
pub struct Params(Option<Box<RawValue>>);

impl Params {
    pub fn new<P>(params: P) -> Result<Self, JsonRpseeError>
    where
        P: ToRpcParams,
    {
        let params = params.to_rpc_params()?;
        Ok(Self(params))
    }
}

impl ToRpcParams for Params {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, JsonRpseeError> {
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
    ) -> RpcFuture<'a, Box<RawValue>> {
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
    ) -> RpcFuture<'a, RpcSubscription> {
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
                }
                _ => None,
            };

            let stream = stream
                .map_err(|e| RpcError::ClientError(Box::new(e)))
                .boxed();
            Ok(RpcSubscription { stream, id })
        })
    }
}

impl<T> Debug for AutoReconnectClient<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoReconnectClient")
            .finish_non_exhaustive()
    }
}
