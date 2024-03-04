use impl_serde_macro::serialize::{deserialize_check_len, serialize_uint, ExpectedLen};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_std::vec::Vec;

/// serde functions for converting `u64` to and from hexadecimal string
pub mod as_hex {
    use super::{DeserializableNumber, SerializableNumber};
    use serde::{Deserializer, Serializer};

    #[allow(clippy::trivially_copy_pass_by_ref)]
    /// # Errors
    /// Returns `Err` if the value cannot be encoded as bytes
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: SerializableNumber + core::fmt::Debug,
        S: Serializer,
    {
        T::serialize_eth_uint(value, serializer)
    }

    /// # Errors
    /// Returns `Err` source is not a valid hexadecimal string
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: DeserializableNumber<'de>,
        D: Deserializer<'de>,
    {
        T::deserialize_eth_uint(deserializer)
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
