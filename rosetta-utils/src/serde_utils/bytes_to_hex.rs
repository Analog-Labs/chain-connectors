use super::{HexDeserializable, HexSerializable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_std::result::Result;

/// serde functions for converting `u64` to and from hexadecimal string
pub mod as_hex {
    use super::{DeserializableBytes, SerializableBytes};
    use serde::{Deserializer, Serializer};

    /// # Errors
    /// Returns `Err` value cannot be encoded as an hexadecimal string
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: SerializableBytes,
        S: Serializer,
    {
        T::serialize_bytes(value, serializer)
    }

    /// # Errors
    /// Returns `Err` value cannot be encoded as an hexadecimal string
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: DeserializableBytes<'de>,
        D: Deserializer<'de>,
    {
        T::deserialize_bytes(deserializer)
    }
}

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

impl<T> SerializableBytes for T
where
    T: HexSerializable,
{
    fn serialize_bytes<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = HexSerializable::to_bytes(self);
        let value = impl_serde_macro::serialize::serialize(bytes.as_ref(), serializer)?;
        Ok(value)
    }
}

impl<'de, T> DeserializableBytes<'de> for T
where
    T: HexDeserializable,
{
    fn deserialize_bytes<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = impl_serde_macro::serialize::deserialize(deserializer)?;
        let value = <T as HexDeserializable>::from_bytes::<D>(bytes)?;
        Ok(value)
    }
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

pub trait DeserializableBytes<'de>: Sized {
    /// Deserialize a bytes from hexadecimal string
    /// # Errors
    /// should never fails
    fn deserialize_bytes<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

/// Implement the `DeserializableBytes` trait for primitive unsigned integers
macro_rules! impl_uint_to_fixed_bytes {
    ($name: ty) => {
        impl<'de> DeserializableBytes<'de> for $name {
            fn deserialize_bytes<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let mut output = [0u8; ::core::mem::size_of::<Self>()];
                let bytes = impl_serde_macro::serialize::deserialize(deserializer)?;
                if bytes.len() != ::core::mem::size_of::<Self>() {
                    return Err(serde::de::Error::custom("invalid length"));
                }
                output.copy_from_slice(bytes.as_ref());
                Ok(Self::from_be_bytes(output))
            }
        }
    };
}

impl_uint_to_fixed_bytes!(u16);
impl_uint_to_fixed_bytes!(u32);
impl_uint_to_fixed_bytes!(u64);
impl_uint_to_fixed_bytes!(u128);

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
