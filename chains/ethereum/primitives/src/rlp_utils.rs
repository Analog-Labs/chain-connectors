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
