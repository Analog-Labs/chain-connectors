use crate::rstd::{
    borrow::Borrow,
    fmt::{Debug, Display, Formatter, LowerHex, Result as FmtResult},
    ops::Deref,
    str::FromStr,
    string::String,
    vec::Vec,
};

/// Wrapper type around [`bytes::Bytes`] to support "0x" prefixed hex strings.
#[derive(Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "with-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bytes(
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "serialize_bytes", deserialize_with = "deserialize_bytes")
    )]
    pub bytes::Bytes,
);

#[cfg(feature = "with-codec")]
impl scale_info::TypeInfo for Bytes {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("Bytes", module_path!()))
            .composite(
                scale_info::build::FieldsBuilder::<_, scale_info::build::UnnamedFields>::default()
                    .field(|f| f.ty::<[u8]>().type_name("Vec<u8>")),
            )
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Encodable for Bytes {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        <bytes::Bytes as rlp::Encodable>::rlp_append(&self.0, s);
    }
    fn rlp_bytes(&self) -> bytes::BytesMut {
        <bytes::Bytes as rlp::Encodable>::rlp_bytes(&self.0)
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Decodable for Bytes {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let bytes = <bytes::Bytes as rlp::Decodable>::decode(rlp)?;
        Ok(Self(bytes))
    }
}

impl const_hex::FromHex for Bytes {
    type Error = const_hex::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        const_hex::decode(hex).map(Into::into)
    }
}

impl FromIterator<u8> for Bytes {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        iter.into_iter().collect::<bytes::Bytes>().into()
    }
}

impl<'a> FromIterator<&'a u8> for Bytes {
    fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
        iter.into_iter().copied().collect::<bytes::Bytes>().into()
    }
}

impl Bytes {
    /// Creates a new empty `Bytes`.
    ///
    /// This will not allocate and the returned `Bytes` handle will be empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use rosetta_ethereum_types::Bytes;
    ///
    /// let b = Bytes::new();
    /// assert_eq!(&b[..], b"");
    /// ```
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(bytes::Bytes::new())
    }

    /// Creates a new `Bytes` from a static slice.
    ///
    /// The returned `Bytes` will point directly to the static slice. There is
    /// no allocating or copying.
    ///
    /// # Examples
    ///
    /// ```
    /// use rosetta_ethereum_types::Bytes;
    ///
    /// let b = Bytes::from_static(b"hello");
    /// assert_eq!(&b[..], b"hello");
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_static(bytes: &'static [u8]) -> Self {
        Self(bytes::Bytes::from_static(bytes))
    }

    pub fn hex_encode(&self) -> String {
        const_hex::encode(self.0.as_ref())
    }

    pub const fn len(&self) -> usize {
        bytes::Bytes::len(&self.0)
    }

    pub const fn is_empty(&self) -> bool {
        bytes::Bytes::is_empty(&self.0)
    }
}

impl Debug for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Bytes(0x{})", self.hex_encode())
    }
}

impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "0x{}", self.hex_encode())
    }
}

impl LowerHex for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "0x{}", self.hex_encode())
    }
}

impl Deref for Bytes {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Borrow<[u8]> for Bytes {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}

impl IntoIterator for Bytes {
    type Item = u8;
    type IntoIter = bytes::buf::IntoIter<bytes::Bytes>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Bytes {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().iter()
    }
}

impl From<bytes::Bytes> for Bytes {
    fn from(src: bytes::Bytes) -> Self {
        Self(src)
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(src: Vec<u8>) -> Self {
        Self(src.into())
    }
}

impl<const N: usize> From<[u8; N]> for Bytes {
    fn from(src: [u8; N]) -> Self {
        Self(bytes::Bytes::copy_from_slice(src.as_slice()))
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for Bytes {
    fn from(src: &'a [u8; N]) -> Self {
        Self(bytes::Bytes::copy_from_slice(src))
    }
}

impl PartialEq<[u8]> for Bytes {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<Bytes> for [u8] {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

impl PartialEq<Vec<u8>> for Bytes {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_ref() == &other[..]
    }
}

impl PartialEq<Bytes> for Vec<u8> {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

impl PartialEq<bytes::Bytes> for Bytes {
    fn eq(&self, other: &bytes::Bytes) -> bool {
        other == self.as_ref()
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(thiserror::Error), error("Failed to parse bytes: {0}"))]
pub struct ParseBytesError(const_hex::FromHexError);

impl FromStr for Bytes {
    type Err = ParseBytesError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        const_hex::FromHex::from_hex(value).map_err(ParseBytesError)
    }
}

/// Serialize bytes as "0x" prefixed hex string
///
/// # Errors
/// never fails
#[cfg(feature = "serde")]
pub fn serialize_bytes<S, T>(d: T, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: AsRef<[u8]>,
{
    const_hex::serialize::<S, T>(d, s)
}

/// Deserialize bytes as "0x" prefixed hex string
///
/// # Errors
/// never fails
#[cfg(feature = "serde")]
pub fn deserialize_bytes<'de, D>(d: D) -> Result<bytes::Bytes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = <String as serde::Deserialize>::deserialize(d)?;
    const_hex::decode(value).map(Into::into).map_err(serde::de::Error::custom)
}
