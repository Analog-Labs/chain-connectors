use subxt::{
    error::RpcError,
    rpc::{RpcClientT, RawValue, RpcFuture, RpcSubscription},
};
use futures::stream::{StreamExt, TryStreamExt};
use jsonrpsee::{
    core::{client::{ClientBuilder, Client, ClientT, SubscriptionClientT, SubscriptionKind}, traits::ToRpcParams, Error as JsonRpseeError},
    types::SubscriptionId,
    client_transport::ws::{Uri, WsTransportClientBuilder},
};

pub struct Params(Option<Box<RawValue>>);

pub struct ClientWrapper(pub Client);

impl ToRpcParams for Params {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, JsonRpseeError> {
        Ok(self.0)
    }
}

impl RpcClientT for ClientWrapper {
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

pub async fn default_client(url: &str) -> anyhow::Result<ClientWrapper> {
    let uri: Uri = url.parse()?;
    let (sender, receiver) = WsTransportClientBuilder::default()
        .use_webpki_rustls()
        .build(uri)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let client = ClientBuilder::default()
        .max_buffer_capacity_per_subscription(4096)
        // .max_notifs_per_subscription(4096)
        .build_with_tokio(sender, receiver);
    Ok(ClientWrapper(client))
}
