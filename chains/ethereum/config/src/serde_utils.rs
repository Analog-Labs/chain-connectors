use crate::rstd::{option::Option, result::Result, vec::Vec};
use impl_serde::serialize::{deserialize_check_len, serialize_uint, ExpectedLen};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// serde functions for converting `u64` to and from hexadecimal string
pub mod uint_to_hex {
    use super::{DeserializableNumber, SerializableNumber};
    use serde::{Deserializer, Serializer};

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: SerializableNumber + core::fmt::Debug,
        S: Serializer,
    {
        T::serialize_eth_uint(value, serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: DeserializableNumber<'de>,
        D: Deserializer<'de>,
    {
        T::deserialize_eth_uint(deserializer)
    }
}

/// serde functions for converting `u64` to and from hexadecimal string
pub mod bytes_to_hex {
    use super::{DeserializableBytes, SerializableBytes};
    use serde::{Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: SerializableBytes,
        S: Serializer,
    {
        T::serialize_bytes(value, serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: DeserializableBytes<'de>,
        D: Deserializer<'de>,
    {
        T::deserialize_bytes(deserializer)
    }
}

/// Deserialize that always returns `Some(T)` or `Some(T::default())` must be used with
/// `#[serde(deserialize_with = "deserialize_null_default")]` attribute
///
/// # Errors
/// returns an error if fails to deserialize T
pub fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = <Option<T> as Deserialize<'de>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Serialize a primitive uint as hexadecimal string, must be used with `#[serde(serialize_with =
/// "serialize_uint")]` attribute
pub trait SerializableNumber {
    /// Serialize a primitive uint as hexadecimal string
    /// # Errors
    /// should never fails
    fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<T> SerializableNumber for Option<T>
where
    T: SerializableNumber,
{
    fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wrapped = self.as_ref().map(SerializeWrapper);
        <Option<SerializeWrapper<T>> as Serialize>::serialize(&wrapped, serializer)
    }
}

impl<T> SerializableNumber for Vec<T>
where
    T: SerializableNumber,
{
    fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wrapped = self.iter().map(SerializeWrapper).collect::<Vec<_>>();
        <Vec<SerializeWrapper<T>> as Serialize>::serialize(&wrapped, serializer)
    }
}

pub trait DeserializableNumber<'de>: Sized {
    /// Deserialize a primitive uint from hexadecimal string
    /// # Errors
    /// should never fails
    fn deserialize_eth_uint<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl<'de, T> DeserializableNumber<'de> for Option<T>
where
    T: DeserializableNumber<'de> + core::fmt::Debug,
{
    /// Deserialize a primitive uint from hexadecimal string
    /// # Errors
    /// should never fails
    fn deserialize_eth_uint<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wrapped =
            <Option<DeserializeWrapper<T>> as Deserialize<'de>>::deserialize(deserializer)?;
        Ok(wrapped.map(DeserializeWrapper::into_inner))
    }
}

/// Helper for deserializing optional uints from hexadecimal string
struct DeserializeWrapper<T>(T);

impl<T> core::fmt::Debug for DeserializeWrapper<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("DeserializeWrapper").field(&self.0).finish()
    }
}

impl<T> core::fmt::Display for DeserializeWrapper<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <T as core::fmt::Display>::fmt(&self.0, f)
    }
}

impl<T> DeserializeWrapper<T> {
    fn into_inner(self) -> T {
        self.0
    }
}

impl<'de, T> Deserialize<'de> for DeserializeWrapper<T>
where
    T: DeserializableNumber<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = <T as DeserializableNumber<'de>>::deserialize_eth_uint(deserializer)?;
        Ok(Self(value))
    }
}

/// Helper for serializing optional uints to hexadecimal string
struct SerializeWrapper<'a, T>(&'a T);

impl<'a, T> SerializeWrapper<'a, T> {
    const fn inner(&self) -> &T {
        self.0
    }
}

impl<'a, T> Serialize for SerializeWrapper<'a, T>
where
    T: SerializableNumber,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <T as SerializableNumber>::serialize_eth_uint(self.inner(), serializer)
    }
}

macro_rules! impl_serialize_uint {
    ($name: ident) => {
        impl SerializableNumber for $name {
            fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                const LEN: usize = $name::BITS as usize;

                let mut slice = [0u8; 2 + 2 * LEN];
                let bytes = self.to_be_bytes();
                serialize_uint(&mut slice, &bytes, serializer)
            }
        }

        impl<'de> DeserializableNumber<'de> for $name {
            fn deserialize_eth_uint<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                const LEN: usize = $name::BITS as usize;
                let mut bytes = [0u8; LEN / 8];
                let wrote =
                    deserialize_check_len(deserializer, ExpectedLen::Between(0, &mut bytes))?;
                let value = $name::from_be_bytes(bytes) >> (LEN - (wrote * 8));
                Ok(value)
            }
        }
    };
}

impl_serialize_uint!(u8);
impl_serialize_uint!(u16);
impl_serialize_uint!(u32);
impl_serialize_uint!(u64);
impl_serialize_uint!(u128);
impl_serialize_uint!(usize);

/// Serialize a primitive uint as hexadecimal string, must be used with `#[serde(serialize_with =
/// "serialize_uint")]` attribute
pub trait SerializableBytes {
    /// Serialize a primitive uint as hexadecimal string
    /// # Errors
    /// should never fails
    fn serialize_bytes<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

struct SerializeBytesWrapper<'a, T>(&'a T);

impl<'a, T> SerializeBytesWrapper<'a, T> {
    const fn inner(&self) -> &T {
        self.0
    }
}

impl<'a, T> Serialize for SerializeBytesWrapper<'a, T>
where
    T: SerializableBytes,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <T as SerializableBytes>::serialize_bytes(self.inner(), serializer)
    }
}

impl<T> SerializableBytes for Option<T>
where
    T: SerializableBytes,
{
    fn serialize_bytes<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wrapped = self.as_ref().map(SerializeBytesWrapper);
        <Option<SerializeBytesWrapper<T>> as Serialize>::serialize(&wrapped, serializer)
    }
}

impl<T> SerializableBytes for Vec<T>
where
    T: SerializableBytes,
{
    fn serialize_bytes<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wrapped = self.iter().map(SerializeBytesWrapper).collect::<Vec<_>>();
        <Vec<SerializeBytesWrapper<T>> as Serialize>::serialize(&wrapped, serializer)
    }
}

impl SerializableBytes for Vec<u8> {
    fn serialize_bytes<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        impl_serde::serialize::serialize(self.as_ref(), serializer)
    }
}

impl<const N: usize> SerializableBytes for [u8; N] {
    fn serialize_bytes<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        impl_serde::serialize::serialize(self.as_ref(), serializer)
    }
}

impl SerializableBytes for [u8] {
    fn serialize_bytes<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        impl_serde::serialize::serialize(self, serializer)
    }
}

pub trait DeserializableBytes<'de>: Sized {
    /// Deserialize a bytes from hexadecimal string
    /// # Errors
    /// should never fails
    fn deserialize_bytes<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl<'de> DeserializableBytes<'de> for Vec<u8> {
    fn deserialize_bytes<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        impl_serde::serialize::deserialize(deserializer)
    }
}

impl<'de, const N: usize> DeserializableBytes<'de> for [u8; N] {
    fn deserialize_bytes<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut output = [0u8; N];
        let bytes = impl_serde::serialize::deserialize(deserializer)?;
        if bytes.len() != N {
            return Err(serde::de::Error::custom(format!("invalid length: {}", bytes.len())));
        }
        output.copy_from_slice(bytes.as_ref());
        Ok(output)
    }
}

/// Helper for deserializing bytes from hexadecimal string
struct DeserializeBytesWrapper<T>(T);

impl<T> DeserializeBytesWrapper<T> {
    fn into_inner(self) -> T {
        self.0
    }
}

impl<'de, T> Deserialize<'de> for DeserializeBytesWrapper<T>
where
    T: DeserializableBytes<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = <T as DeserializableBytes<'de>>::deserialize_bytes(deserializer)?;
        Ok(Self(value))
    }
}

impl<'de, T> DeserializableBytes<'de> for Option<T>
where
    T: DeserializableBytes<'de>,
{
    /// Deserialize from hexadecimal string
    /// # Errors
    /// should never fails
    fn deserialize_bytes<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wrapped =
            <Option<DeserializeBytesWrapper<T>> as Deserialize<'de>>::deserialize(deserializer)?;
        Ok(wrapped.map(DeserializeBytesWrapper::into_inner))
    }
}

impl<'de, T> DeserializableBytes<'de> for Vec<T>
where
    T: DeserializableBytes<'de>,
{
    /// Deserialize from hexadecimal string
    /// # Errors
    /// should never fails
    fn deserialize_bytes<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let res = unsafe {
            let wrapped =
                <Vec<DeserializeBytesWrapper<T>> as Deserialize<'de>>::deserialize(deserializer)?;
            core::mem::transmute::<Vec<DeserializeBytesWrapper<T>>, Self>(wrapped)
        };
        Ok(res)
    }
}
