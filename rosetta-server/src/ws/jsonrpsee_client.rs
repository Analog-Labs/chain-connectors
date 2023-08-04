use futures::stream::{StreamExt, TryStreamExt};
use jsonrpsee::{
    core::{
        client::{Client, ClientT, SubscriptionClientT, SubscriptionKind},
        traits::ToRpcParams,
        Error as JsonRpseeError,
    },
    types::SubscriptionId,
};
use subxt::{
    error::RpcError,
    rpc::{RawValue, RpcClientT, RpcFuture, RpcSubscription},
};

pub struct Params(Option<Box<RawValue>>);
pub struct RpcClient(pub Client);

impl ToRpcParams for Params {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, JsonRpseeError> {
        Ok(self.0)
    }
}

impl RpcClientT for RpcClient {
    fn request_raw<'a>(
        &'a self,
        method: &'a str,
        params: Option<Box<RawValue>>,
    ) -> RpcFuture<'a, Box<RawValue>> {
        Box::pin(async move {
            let res = ClientT::request(&self.0, method, Params(params))
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
                &self.0,
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
