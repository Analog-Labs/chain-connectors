use jsonrpsee::{core::Error, types::InvalidRequestId};

/// A version of `Error` that implements `Clone`.
#[derive(Debug)]
pub struct CloneableError(pub Error);
impl Clone for CloneableError {
    fn clone(&self) -> Self {
        let error = match &self.0 {
            Error::Call(call) => Error::Call(call.clone()),
            Error::Transport(error) => Error::Transport(anyhow::format_err!("{error:?}")),
            Error::InvalidResponse(error) => Error::InvalidResponse(error.clone()),
            Error::RestartNeeded(reason) => Error::RestartNeeded(reason.clone()),
            Error::ParseError(error) => Error::Custom(format!("{error:?}")), // TODO: return an parser error instead a custom error
            Error::InvalidSubscriptionId => Error::InvalidSubscriptionId,
            Error::InvalidRequestId(error) => Error::InvalidRequestId(match error {
                InvalidRequestId::Invalid(message) => InvalidRequestId::Invalid(message.clone()),
                InvalidRequestId::NotPendingRequest(message) => {
                    InvalidRequestId::NotPendingRequest(message.clone())
                }
                InvalidRequestId::Occupied(message) => InvalidRequestId::Occupied(message.clone()),
            }),
            Error::UnregisteredNotification(error) => {
                Error::UnregisteredNotification(error.clone())
            }
            Error::DuplicateRequestId => Error::DuplicateRequestId,
            Error::MethodAlreadyRegistered(method) => {
                Error::MethodAlreadyRegistered(method.clone())
            }
            Error::MethodNotFound(method) => Error::MethodNotFound(method.clone()),
            Error::SubscriptionNameConflict(name) => Error::SubscriptionNameConflict(name.clone()),
            Error::RequestTimeout => Error::RequestTimeout,
            Error::MaxSlotsExceeded => Error::MaxSlotsExceeded,
            Error::AlreadyStopped => Error::AlreadyStopped,
            Error::EmptyAllowList(list) => Error::EmptyAllowList(list),
            Error::HttpHeaderRejected(header, value) => {
                Error::HttpHeaderRejected(header, value.to_string())
            }
            Error::Custom(message) => Error::Custom(message.clone()),
            Error::HttpNotImplemented => Error::HttpNotImplemented,
            Error::EmptyBatchRequest => Error::EmptyBatchRequest,
        };
        Self(error)
    }
}
