use ethers::providers::{
    JsonRpcError as EthJsonRpcError, ProviderError as EthProviderError,
    RpcError as EthRpcErrorTrait,
};
use jsonrpsee::{client_transport::ws::WsHandshakeError, core::Error as JsonRpseeError};

/// Adapter for [`jsonrpsee::core::Error`] to [`ethers::providers::RpcError`].
#[derive(Debug, thiserror::Error)]
pub enum EthError {
    /// Thrown if the response could not be parsed
    #[error("{original}")]
    JsonRpsee { original: JsonRpseeError, message: Option<EthJsonRpcError> },

    /// Failed to parse the data.
    #[allow(clippy::enum_variant_names)]
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

    /// Error that can happen during the WebSocket handshake.
    #[error("WS Handshake failed: {0}")]
    HandshakeFailed(WsHandshakeError),

    /// The background task has been terminated.
    #[error("The background task been terminated because: {0}; restart required")]
    RestartNeeded(String),

    /// The client is reconnecting
    #[error("The client is restarting the background task")]
    Reconnecting,
}

impl From<JsonRpseeError> for EthError {
    fn from(error: JsonRpseeError) -> Self {
        match error {
            JsonRpseeError::Call(call) => {
                let code = i64::from(call.code());
                let data =
                    call.data().and_then(|raw_value| serde_json::value::to_value(raw_value).ok());
                let message = call.message().to_string();
                Self::JsonRpsee {
                    original: JsonRpseeError::Call(call),
                    message: Some(EthJsonRpcError { code, message, data }),
                }
            },
            JsonRpseeError::ParseError(serde_error) => Self::ParseError(serde_error),
            JsonRpseeError::RestartNeeded(reason) => Self::RestartNeeded(reason),
            error => {
                let message = format!("{}", &error);
                Self::JsonRpsee {
                    original: error,
                    message: Some(EthJsonRpcError { code: 9999, message, data: None }),
                }
            },
        }
    }
}

impl From<WsHandshakeError> for EthError {
    fn from(error: WsHandshakeError) -> Self {
        Self::HandshakeFailed(error)
    }
}

impl From<EthError> for EthProviderError {
    fn from(error: EthError) -> Self {
        match error {
            EthError::ParseError(error) => Self::SerdeJson(error),
            EthError::HandshakeFailed(error) => Self::CustomError(error.to_string()),
            error => Self::JsonRpcClientError(Box::new(error)),
        }
    }
}

impl EthRpcErrorTrait for EthError {
    fn as_error_response(&self) -> Option<&EthJsonRpcError> {
        match self {
            Self::JsonRpsee { message, .. } => message.as_ref(),
            _ => None,
        }
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        match self {
            Self::ParseError(error) => Some(error),
            _ => None,
        }
    }
}
