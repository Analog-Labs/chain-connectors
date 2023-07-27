use thiserror::Error;

/// Errors that can occur while converting or parsing addresses.
#[derive(Debug, Error, PartialEq)]
pub enum AddressError {
    #[error("Invalid address format")]
    InvalidAddressFormat,

    #[error("Failed to decode address")]
    FailedToDecodeAddress,
}
