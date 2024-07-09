#![allow(clippy::wrong_self_convention)]
use jsonrpsee::{core::ClientError as JsonRpseeError, types::InvalidRequestId};
use std::sync::Arc;

pub trait CloneableJsonRpseeError {
    fn as_error(self) -> JsonRpseeError;
}

impl CloneableJsonRpseeError for JsonRpseeError {
    fn as_error(self) -> JsonRpseeError {
        clone_error(&self)
    }
}

impl CloneableJsonRpseeError for &JsonRpseeError {
    fn as_error(self) -> JsonRpseeError {
        clone_error(self)
    }
}

impl CloneableJsonRpseeError for Arc<JsonRpseeError> {
    fn as_error(self) -> JsonRpseeError {
        clone_error(self.as_ref())
    }
}

// impl <T> CloneableJsonRpseeError for &T where T: AsRef<JsonRpseeError> {
//     fn as_error(&self) -> JsonRpseeError {
//         clone_error(AsRef::as_ref(&self))
//     }
// }

fn clone_error(error: &JsonRpseeError) -> JsonRpseeError {
    match error {
        JsonRpseeError::Call(error) => JsonRpseeError::Call(error.clone()),
        JsonRpseeError::Transport(error) => {
            JsonRpseeError::Transport(anyhow::format_err!("{error:?}"))
        },
        JsonRpseeError::RestartNeeded(reason) => JsonRpseeError::RestartNeeded(reason.clone()),
        JsonRpseeError::ParseError(error) => JsonRpseeError::Custom(format!("{error:?}")), /* TODO: return an parser error instead a custom error */
        JsonRpseeError::InvalidSubscriptionId => JsonRpseeError::InvalidSubscriptionId,
        JsonRpseeError::InvalidRequestId(error) => JsonRpseeError::InvalidRequestId(match error {
            InvalidRequestId::Invalid(message) => InvalidRequestId::Invalid(message.clone()),
            InvalidRequestId::NotPendingRequest(message) => {
                InvalidRequestId::NotPendingRequest(message.clone())
            },
            InvalidRequestId::Occupied(message) => InvalidRequestId::Occupied(message.clone()),
        }),
        JsonRpseeError::RequestTimeout => JsonRpseeError::RequestTimeout,
        JsonRpseeError::Custom(message) => JsonRpseeError::Custom(message.clone()),
        JsonRpseeError::HttpNotImplemented => JsonRpseeError::HttpNotImplemented,
        JsonRpseeError::EmptyBatchRequest(request) => JsonRpseeError::EmptyBatchRequest(*request),
        JsonRpseeError::RegisterMethod(error) => JsonRpseeError::RegisterMethod(error.clone()),
    }
}
