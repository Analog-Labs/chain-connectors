use core::{num::NonZeroUsize, time::Duration};
use jsonrpsee::{
    client_transport::ws::WsTransportClientBuilder,
    core::client::{ClientBuilder, IdKind},
};

/// Ten megabytes.
pub const TEN_MB_SIZE_BYTES: usize = 10 * 1024 * 1024;

/// Supported websocket transport clients.
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

/// Retry strategies including fixed interval and exponential back-off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetryStrategyConfig {
    /// fixed interval strategy, given a duration in milliseconds.
    FixedInterval(Duration),

    /// A retry strategy driven by exponential back-off.
    /// The power corresponds to the number of past attempts.
    ExponentialBackoff {
        /// base duration in milliseconds.
        /// The resulting duration is calculated by taking the base to the n-th power, where n denotes the number of past attempts.
        base: u64,
        /// A multiplicative factor that will be applied to the retry delay.
        /// For example, using a factor of 1000 will make each delay in units of seconds.
        /// Default factor is 1.
        factor: Option<u64>,
        /// Apply a maximum delay. No retry delay will be longer than this Duration.
        max_delay: Option<Duration>,
    },

    /// A retry strategy driven by the fibonacci series.
    /// Each retry uses a delay which is the sum of the two previous delays.
    /// Depending on the problem at hand, a fibonacci retry strategy might perform better and lead to better throughput than the ExponentialBackoff strategy.
    /// See "A Performance Comparison of Different Backoff Algorithms under Different Rebroadcast Probabilities for MANETs."  for more details.
    FibonacciBackoff {
        /// Initial base duration in milliseconds.
        initial: u64,
        /// A multiplicative factor that will be applied to the retry delay.
        /// For example, using a factor of 1000 will make each delay in units of seconds.
        /// Default factor is 1.
        factor: Option<u64>,
        /// Apply a maximum delay. No retry delay will be longer than this Duration.
        max_delay: Option<Duration>,
    },
}

/// Common configuration for Socketto and Tungstenite clients.
#[derive(Debug, Clone)]
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

    /// JSON-RPC timeout for the RPC request. Defaults to 60sec.
    pub rpc_request_timeout: Duration,

    /// JSON-RPC max concurrent requests (default is 256).
    pub rpc_max_concurrent_requests: usize,

    /// JSON-RPC max buffer capacity for each subscription; when the capacity is exceeded the subscription will be dropped (default is 1024).
    /// You may prevent the subscription from being dropped by polling often enough Subscription::next() such that it can keep with the rate as server produces new items on the subscription.
    pub rpc_max_buffer_capacity_per_subscription: NonZeroUsize,

    /// JSON-RPC request object id data type. (default is IdKind::Number)
    pub rpc_id_kind: IdKind,

    /// Max length for logging for requests and responses.
    /// Entries bigger than this limit will be truncated.
    /// (default is 4096)
    pub rpc_max_log_length: u32,

    /// Set the interval at which pings frames are submitted (disabled by default).
    ///
    /// Periodically submitting pings at a defined interval has mainly two benefits:
    ///  - Directly, it acts as a "keep-alive" alternative in the WebSocket world.
    ///  - Indirectly by inspecting debug logs, it ensures that the endpoint is still responding to messages.
    ///
    /// The underlying implementation does not make any assumptions about at which intervals pongs are received.
    ///
    /// Note: The interval duration is restarted when
    ///  - a frontend command is submitted
    ///  - a reply is received from the backend
    ///  - the interval duration expires
    pub rpc_ping_interval: Option<Duration>,

    /// Retry strategy for reconnecting to the server.
    /// Default is [`RetryStrategyConfig::FibonacciBackoff`] with 5 seconds base and
    /// 30 seconds maximum between retries.
    pub retry_strategy: RetryStrategyConfig,
}

impl Default for RpcClientConfig {
    fn default() -> Self {
        Self {
            // Default WS Transport config.
            client: WsTransportClient::Auto,
            write_buffer_size: 128 * 1024,
            max_write_buffer_size: usize::MAX,
            max_message_size: Some(TEN_MB_SIZE_BYTES),
            max_frame_size: Some(16 << 20),
            accept_unmasked_frames: false,

            // Default JSON-RPC config.
            rpc_request_timeout: Duration::from_secs(60),
            rpc_max_concurrent_requests: 256,
            rpc_max_buffer_capacity_per_subscription: unsafe { NonZeroUsize::new_unchecked(1024) },
            rpc_id_kind: IdKind::Number,
            rpc_max_log_length: 4096,
            rpc_ping_interval: None,

            // Reconnect Retry strategy.
            retry_strategy: RetryStrategyConfig::FibonacciBackoff {
                initial: 5000,
                factor: None,
                max_delay: Some(Duration::from_secs(30)),
            },
        }
    }
}

impl From<&RpcClientConfig> for ClientBuilder {
    fn from(config: &RpcClientConfig) -> Self {
        let mut builder = Self::new()
            .request_timeout(config.rpc_request_timeout)
            .max_concurrent_requests(config.rpc_max_concurrent_requests)
            .max_buffer_capacity_per_subscription(
                config.rpc_max_buffer_capacity_per_subscription.get(),
            )
            .id_format(config.rpc_id_kind)
            .set_max_logging_length(config.rpc_max_log_length);
        if let Some(ping_internal) = config.rpc_ping_interval {
            builder = builder.ping_interval(ping_internal);
        }
        builder
    }
}

impl From<&RpcClientConfig> for WsTransportClientBuilder {
    fn from(config: &RpcClientConfig) -> Self {
        let message_size =
            u32::try_from(config.max_message_size.unwrap_or(TEN_MB_SIZE_BYTES)).unwrap_or(u32::MAX);
        let mut builder = Self::default()
            .max_request_size(message_size)
            .max_response_size(message_size)
            .max_redirections(5);

        builder = if cfg!(feature = "webpki-tls") {
            builder.use_webpki_rustls()
        } else {
            builder.use_native_rustls()
        };

        builder
    }
}
