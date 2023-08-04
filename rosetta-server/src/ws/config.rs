/// Ten megabytes.
pub const TEN_MB_SIZE_BYTES: usize = 10 * 1024 * 1024;

/// Supported WebSocket transport clients.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WsTransportClient {
    /// Auto will try to use Socketto first, if it fails, it will fallback to Tungstenite.
    #[default]
    Auto,

    /// Socketto is the default WebSocket client for Substrate and Subxt.
    /// Whoever have an issue when connecting to some RPC nodes using TLS.
    /// https://github.com/paritytech/jsonrpsee/issues/1142
    Socketto,

    /// Tungstenite is the most used stream-based WebSocket Client
    /// Use this if you have issues with Socketto.
    Tungstenite,
}

/// Common configuration for Socketto and Tungstenite clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RpcClientConfig {
    /// Supported WebSocket transport clients.
    pub client: WsTransportClient,

    /// The target minimum size of the write buffer to reach before writing the data
    /// to the underlying stream.
    /// The default value is 128 KiB.
    pub write_buffer_size: usize,

    /// The max size of the write buffer in bytes. Setting this can provide backpressure
    /// in the case the write buffer is filling up due to write errors.
    /// The default value is unlimited.
    ///
    /// Note: Should always be at least [`write_buffer_size + 1 message`](Self::write_buffer_size)
    /// and probably a little more depending on error handling strategy.
    pub max_write_buffer_size: usize,

    /// The maximum size of a message. `None` means no size limit. The default value is 10 MiB
    /// which should be reasonably big for all normal use-cases but small enough to prevent
    /// memory eating by a malicious user.
    pub max_message_size: Option<usize>,

    /// The maximum size of a single message frame. `None` means no size limit. The limit is for
    /// frame payload NOT including the frame header. The default value is 16 MiB which should
    /// be reasonably big for all normal use-cases but small enough to prevent memory eating
    /// by a malicious user.
    pub max_frame_size: Option<usize>,

    /// Whether to accept unmasked frames from the peer. The default value is `false`.
    /// from the client. According to the RFC 6455, the server must close the
    /// connection to the client in such cases, however it seems like there are
    /// some popular libraries that are sending unmasked frames, ignoring the RFC.
    /// By default this option is set to `false`, i.e. according to RFC 6455.
    ///
    /// OBS: not supported for Socketto client.
    pub accept_unmasked_frames: bool,
}

impl Default for RpcClientConfig {
    fn default() -> Self {
        Self {
            client: WsTransportClient::Auto,
            write_buffer_size: 128 * 1024,
            max_write_buffer_size: usize::MAX,
            max_message_size: Some(TEN_MB_SIZE_BYTES),
            max_frame_size: Some(16 << 20),
            accept_unmasked_frames: false,
        }
    }
}
