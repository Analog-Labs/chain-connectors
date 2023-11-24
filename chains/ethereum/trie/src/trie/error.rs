use crate::{
    layout::TrieError,
    rstd::{boxed::Box, vec::Vec},
};
use rlp::DecoderError;

/// Error type used for trie related errors.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum Error {
    #[cfg_attr(feature = "std", error("Bad format"))]
    BadFormat,
    #[cfg_attr(feature = "std", error("Decoding failed: {0}"))]
    Decode(#[cfg_attr(feature = "std", source)] DecoderError),
    #[cfg_attr(
		feature = "std",
		error("Recorded key ({0:x?}) access with value as found={1}, but could not confirm with trie.")
	)]
    InvalidRecording(Vec<u8>, bool),
    #[cfg_attr(feature = "std", error("Trie error: {0:?}"))]
    TrieError(Box<TrieError>),
}

impl From<DecoderError> for Error {
    fn from(x: DecoderError) -> Self {
        Self::Decode(x)
    }
}

impl From<Box<TrieError>> for Error {
    fn from(x: Box<TrieError>) -> Self {
        Self::TrieError(x)
    }
}
