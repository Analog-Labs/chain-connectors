pub mod bytes_to_hex;
pub mod uint_to_hex;
use sp_std::vec::Vec;

use serde::Deserializer;

/// Expected length of bytes vector.
#[derive(Debug, PartialEq, Eq)]
pub enum ExpectedLen {
    /// Exact length in bytes.
    Exact(usize),
    /// A bytes length between (min; `slice.len()`].
    Between(usize, usize),
}

pub trait HexSerializable {
    type Output<'a>: AsRef<[u8]> + 'a
    where
        Self: 'a;

    /// Serialize a primitive uint as hexadecimal string
    fn to_bytes(&self) -> Self::Output<'_>;
}

pub trait HexDeserializable: Sized {
    /// Serialize a primitive uint as hexadecimal string
    /// # Errors
    /// should never fails
    fn from_bytes<'de, D>(bytes: Vec<u8>) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

// Default implementation for Vec<u8>
impl HexSerializable for Vec<u8> {
    type Output<'a> = &'a [u8];

    fn to_bytes(&self) -> Self::Output<'_> {
        self.as_ref()
    }
}

impl HexDeserializable for Vec<u8> {
    fn from_bytes<'de, D>(bytes: Vec<u8>) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(bytes)
    }
}

// Default implementation for [u8; N]
impl<const N: usize> HexSerializable for [u8; N] {
    type Output<'a> = &'a [u8; N];

    fn to_bytes(&self) -> Self::Output<'_> {
        self
    }
}

// Default implementation for Bytes
#[cfg(feature = "bytes")]
mod impl_bytes {
    use super::{Deserializer, HexDeserializable, HexSerializable};
    use bytes::{Bytes, BytesMut};

    impl HexSerializable for Bytes {
        type Output<'a> = &'a [u8];

        fn to_bytes(&self) -> Self::Output<'_> {
            self.as_ref()
        }
    }

    impl HexDeserializable for Bytes {
        fn from_bytes<'de, D>(bytes: Vec<u8>) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(Self::from(bytes))
        }
    }

    #[cfg(feature = "bytes")]
    impl HexSerializable for BytesMut {
        type Output<'a> = &'a [u8];

        fn to_bytes(&self) -> Self::Output<'_> {
            self.as_ref()
        }
    }
}
