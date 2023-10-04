use jsonrpsee::{core::Error as JsonRpseeError, types::InvalidRequestId};

/// A version of [`jsonrpsee::core::Error`] that implements [`core::clone::Clone`] trait.
/// Cloning the error is necessary when using [`futures_util::future::Shared`]. if a reconnect
/// fails, the error must be cloned and send back to all pending requests.
///
/// See [`super::reconnect_impl::ReconnectFuture`] and [`super::reconnect_impl::ReadyOrWaitFuture`]
#[allow(dead_code)]
#[derive(Debug)]
pub struct CloneableError {
    inner: JsonRpseeError,
}

#[allow(dead_code)]
impl CloneableError {
    /// Returns the inner error.
    pub fn into_inner(self) -> JsonRpseeError {
        self.inner
    }
}

impl From<JsonRpseeError> for CloneableError {
    fn from(error: JsonRpseeError) -> Self {
        Self { inner: error }
    }
}

impl From<CloneableError> for JsonRpseeError {
    fn from(error: CloneableError) -> Self {
        error.inner
    }
}

impl Clone for CloneableError {
    fn clone(&self) -> Self {
        let error = match &self.inner {
            JsonRpseeError::Call(call) => JsonRpseeError::Call(call.clone()),
            JsonRpseeError::Transport(error) => {
                JsonRpseeError::Transport(anyhow::format_err!("{error:?}"))
            },
            JsonRpseeError::InvalidResponse(error) => {
                JsonRpseeError::InvalidResponse(error.clone())
            },
            JsonRpseeError::RestartNeeded(reason) => JsonRpseeError::RestartNeeded(reason.clone()),
            JsonRpseeError::ParseError(error) => JsonRpseeError::Custom(format!("{error:?}")), /* TODO: return an parser error instead a custom error */
            JsonRpseeError::InvalidSubscriptionId => JsonRpseeError::InvalidSubscriptionId,
            JsonRpseeError::InvalidRequestId(error) => {
                JsonRpseeError::InvalidRequestId(match error {
                    InvalidRequestId::Invalid(message) => {
                        InvalidRequestId::Invalid(message.clone())
                    },
                    InvalidRequestId::NotPendingRequest(message) => {
                        InvalidRequestId::NotPendingRequest(message.clone())
                    },
                    InvalidRequestId::Occupied(message) => {
                        InvalidRequestId::Occupied(message.clone())
                    },
                })
            },
            JsonRpseeError::UnregisteredNotification(error) => {
                JsonRpseeError::UnregisteredNotification(error.clone())
            },
            JsonRpseeError::DuplicateRequestId => JsonRpseeError::DuplicateRequestId,
            JsonRpseeError::MethodAlreadyRegistered(method) => {
                JsonRpseeError::MethodAlreadyRegistered(method.clone())
            },
            JsonRpseeError::MethodNotFound(method) => {
                JsonRpseeError::MethodNotFound(method.clone())
            },
            JsonRpseeError::SubscriptionNameConflict(name) => {
                JsonRpseeError::SubscriptionNameConflict(name.clone())
            },
            JsonRpseeError::RequestTimeout => JsonRpseeError::RequestTimeout,
            JsonRpseeError::MaxSlotsExceeded => JsonRpseeError::MaxSlotsExceeded,
            JsonRpseeError::AlreadyStopped => JsonRpseeError::AlreadyStopped,
            JsonRpseeError::EmptyAllowList(list) => JsonRpseeError::EmptyAllowList(list),
            JsonRpseeError::HttpHeaderRejected(header, value) => {
                JsonRpseeError::HttpHeaderRejected(header, value.to_string())
            },
            JsonRpseeError::Custom(message) => JsonRpseeError::Custom(message.clone()),
            JsonRpseeError::HttpNotImplemented => JsonRpseeError::HttpNotImplemented,
            JsonRpseeError::EmptyBatchRequest => JsonRpseeError::EmptyBatchRequest,
        };
        Self { inner: error }
    }
}
