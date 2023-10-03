use super::config::RpcClientConfig;
use async_trait::async_trait;
use futures::{
    stream::{SplitSink, SplitStream, StreamExt},
    SinkExt,
};
use jsonrpsee::core::client::{ReceivedMessage, TransportReceiverT, TransportSenderT};
use tide::http::url::Url;
use tokio::net::TcpStream;
pub use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{
        error::CapacityError,
        protocol::{Message, WebSocketConfig},
    },
    MaybeTlsStream, WebSocketStream,
};

impl From<&RpcClientConfig> for WebSocketConfig {
    fn from(config: &RpcClientConfig) -> Self {
        Self {
            write_buffer_size: config.write_buffer_size,
            max_write_buffer_size: config.max_write_buffer_size,
            max_message_size: config.max_message_size,
            max_frame_size: config.max_frame_size,
            accept_unmasked_frames: config.accept_unmasked_frames,
            ..Self::default()
        }
    }
}

/// Tungstenite websocket transport for Jsonrpsee.
pub struct TungsteniteClient {
    sender: Sender,
    receiver: Receiver,
}

impl TungsteniteClient {
    /// Creates a websocket client using the provided `config` and performs the handshare to `url`.
    ///
    /// # Errors
    /// Returns `Err` if the handshake fails
    pub async fn new(url: Url, config: &RpcClientConfig) -> Result<Self, WsError> {
        let config = WebSocketConfig::from(config);
        let (ws_stream, response) = connect_async_with_config(url, Some(config), false).await?;
        let (send, receive) = ws_stream.split();
        tracing::trace!(
            "Successfully connected to the server using Tungstenite. Handshake HTTP code: {}",
            response.status()
        );

        let sender = Sender {
            inner: send,
            max_request_size: config.max_message_size.unwrap_or(usize::MAX),
        };

        let receiver = Receiver { inner: receive };

        Ok(Self { sender, receiver })
    }

    pub(crate) fn split(self) -> (Sender, Receiver) {
        (self.sender, self.receiver)
    }
}

/// Sending end of websocket transport.
#[derive(Debug)]
pub struct Sender {
    inner: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    max_request_size: usize,
}

/// Receiving end of websocket transport.
#[derive(Debug)]
pub struct Receiver {
    inner: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

#[async_trait]
impl TransportSenderT for Sender {
    type Error = WsError;

    /// Sends out a request. Returns a `Future` that finishes when the request has been
    /// successfully sent.
    async fn send(&mut self, body: String) -> Result<(), Self::Error> {
        if body.len() > self.max_request_size {
            return Err(WsError::Capacity(CapacityError::MessageTooLong {
                size: body.len(),
                max_size: self.max_request_size,
            }));
        }

        tracing::trace!("send: {}", body);
        self.inner.send(Message::Text(body)).await?;
        self.inner.flush().await?;
        Ok(())
    }

    /// Sends out a ping request. Returns a `Future` that finishes when the request has been
    /// successfully sent.
    async fn send_ping(&mut self) -> Result<(), Self::Error> {
        self.inner.send(Message::Ping(Vec::default())).await?;
        self.inner.flush().await?;
        Ok(())
    }

    /// Send a close message and close the connection.
    async fn close(&mut self) -> Result<(), Self::Error> {
        self.inner.close().await
    }
}

#[async_trait::async_trait]
impl TransportReceiverT for Receiver {
    type Error = WsError;

    /// Returns a `Future` resolving when the server sent us something back.
    async fn receive(&mut self) -> Result<ReceivedMessage, Self::Error> {
        loop {
            let Some(result) = self.inner.next().await else {
                return Err(WsError::ConnectionClosed);
            };

            match result? {
                Message::Text(text) => break Ok(ReceivedMessage::Text(text)),
                Message::Binary(bytes) => break Ok(ReceivedMessage::Bytes(bytes)),
                Message::Pong(_) => break Ok(ReceivedMessage::Pong),
                Message::Close(_) | Message::Ping(_) | Message::Frame(_) => {}
            }
        }
    }
}
