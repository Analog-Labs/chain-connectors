use std::future::Future;
use serde_json::value::RawValue;
use crate::BlockchainClient;

/// An enum representing the various forms of a WebSocket message.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Message {
    /// A text WebSocket message
    Text(String),
    /// A binary WebSocket message
    Binary(Vec<u8>),
}

/// A transport implementation supporting pub sub subscriptions.
pub trait PubsubBlockchainClient: BlockchainClient {
    /// The type of stream this transport returns
    type NotificationStream: futures_util::stream::Stream<Item = Box<RawValue>> + Send + Unpin;

    /// Future that performs the handshake with the remote.
    type SubscriptionFuture: Future<Output = Result<Self::NotificationStream, Self::Error>>;

    /// Add a subscription to this transport
    fn subscribe<'a>(&'a self, sub: &'a str, params: Option<Box<RawValue>>, unsub: &'a str) -> Self::SubscriptionFuture;

    /// Remove a subscription from this transport
    fn unsubscribe<'a>(&'a self, sub: &'a str, params: Option<Box<RawValue>>, unsub: &'a str) -> Result<(), Self::Error>;
}

/// Possible upgrade on an inbound connection or substream.
pub trait RequestSubscription<C> {
    /// Output after the upgrade has been successfully negotiated and the handshake performed.
    type Output;
    /// Possible error during the handshake.
    type Error;
    /// Future that performs the handshake with the remote.
    type Future: Future<Output = Result<Self::Output, Self::Error>>;

    /// After we have determined that the remote supports one of the protocols we support, this
    /// method is called to start the handshake.
    ///
    /// The `info` is the identifier of the protocol, as produced by `protocol_info`.
    fn upgrade_inbound(self, socket: C, info: Self::Info) -> Self::Future;
}
