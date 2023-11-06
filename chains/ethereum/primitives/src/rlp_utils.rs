use crate::transactions::signature::Signature;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

pub trait RlpStreamExt {
    /// Appends an optional value to the end of stream, chainable.
    ///
    /// ```
    /// use rlp::RlpStream;
    /// use rosetta_ethereum_primitives::rlp_utils::RlpStreamExt;
    /// let mut stream = RlpStream::new_list(2);
    /// stream.append_opt(Some(&"cat")).append_opt(Option::<&u32>::None);
    /// let out = stream.out();
    /// assert_eq!(out, vec![0xc5, 0x83, b'c', b'a', b't', 0x80]);
    /// ```
    fn append_opt<E: Encodable>(&mut self, value: Option<&E>) -> &mut Self;
}

impl RlpStreamExt for RlpStream {
    fn append_opt<E: Encodable>(&mut self, opt: Option<&E>) -> &mut Self {
        if let Some(inner) = opt {
            self.append(inner);
        } else {
            self.append(&"");
        }
        self
    }
}

pub trait RlpExt {
    #[allow(clippy::missing_errors_doc)]
    fn opt_at<T: Decodable>(&self, index: usize) -> Result<Option<T>, DecoderError>;
}

impl RlpExt for Rlp<'_> {
    fn opt_at<T: Decodable>(&self, index: usize) -> Result<Option<T>, DecoderError> {
        let to = {
            let to = self.at(index)?;
            if to.is_empty() {
                if to.is_data() {
                    None
                } else {
                    return Err(rlp::DecoderError::RlpExpectedToBeData);
                }
            } else {
                Some(to.as_val()?)
            }
        };
        Ok(to)
    }
}

#[cfg(feature = "with-rlp")]
pub trait RlpEncodableTransaction {
    fn rlp_append(&self, s: &mut rlp::RlpStream, signature: Option<&Signature>);

    fn rlp_unsigned(&self) -> bytes::Bytes {
        let mut stream = rlp::RlpStream::new();
        self.rlp_append(&mut stream, None);
        stream.out().freeze()
    }

    fn rlp_signed(&self, signature: &Signature) -> bytes::Bytes {
        let mut stream = rlp::RlpStream::new();
        self.rlp_append(&mut stream, Some(signature));
        stream.out().freeze()
    }
}

#[cfg(feature = "with-rlp")]
pub trait RlpDecodableTransaction: Sized {
    /// Decode a raw transaction, returning the decoded transaction and the signature if present.
    /// # Errors
    /// Returns an error if the transaction or signature is invalid.
    fn rlp_decode(
        rlp: &Rlp,
        decode_signature: bool,
    ) -> Result<(Self, Option<Signature>), rlp::DecoderError>;

    /// Decode a raw transaction without signature
    /// # Errors
    /// Returns an error if the transaction is invalid.
    fn rlp_decode_unsigned(rlp: &Rlp) -> Result<Self, DecoderError> {
        Self::rlp_decode(rlp, false).map(|tx| tx.0)
    }

    /// Decode a raw transaction with signature
    /// # Errors
    /// Returns an error if the transaction or signature is invalid.
    fn rlp_decode_signed(rlp: &Rlp) -> Result<(Self, Option<Signature>), rlp::DecoderError> {
        Self::rlp_decode(rlp, true)
    }
}
