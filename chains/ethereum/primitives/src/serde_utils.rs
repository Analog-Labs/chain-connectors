use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

/// Deserialize u64 from hexadecimal string, must be used with
/// `#[serde(deserialize_with = "deserialize_uint")]` attribute
///
/// # Errors
/// returns an error if fails to deserialize T
pub fn deserialize_uint<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializableUInt<'de>,
    D: Deserializer<'de>,
{
    <T as DeserializableUInt<'de>>::deserialize_eth_uint(deserializer)
}

/// Serialize a primitive uint as hexadecimal string, must be used with
/// `#[serde(serialize_with = "serialize_uint")]` attribute
///
/// # Errors
/// returns an error if fails to deserialize T
pub fn serialize_uint<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: SerializableUInt,
    S: Serializer,
{
    <T as SerializableUInt>::serialize_eth_uint(value, serializer)
}

/// Helper for deserializing optional uints from hexadecimal string
struct DeserializeWrapper<T>(T);

impl<T> DeserializeWrapper<T> {
    fn into_inner(self) -> T {
        self.0
    }
}

impl<'de, T> Deserialize<'de> for DeserializeWrapper<T>
where
    T: DeserializableUInt<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = <T as DeserializableUInt<'de>>::deserialize_eth_uint(deserializer)?;
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
    T: SerializableUInt,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <T as SerializableUInt>::serialize_eth_uint(self.inner(), serializer)
    }
}

/// Serialize a primitive uint as hexadecimal string, must be used with `#[serde(serialize_with =
/// "serialize_uint")]` attribute
pub trait SerializableUInt {
    /// Serialize a primitive uint as hexadecimal string
    /// # Errors
    /// should never fails
    fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<T> SerializableUInt for Option<T>
where
    T: SerializableUInt,
{
    fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wrapped = self.as_ref().map(SerializeWrapper);
        <Option<SerializeWrapper<T>> as Serialize>::serialize(&wrapped, serializer)
    }
}

pub trait DeserializableUInt<'de>: Sized {
    /// Deserialize a primitive uint from hexadecimal string
    /// # Errors
    /// should never fails
    fn deserialize_eth_uint<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl<'de, T> DeserializableUInt<'de> for Option<T>
where
    T: DeserializableUInt<'de>,
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

macro_rules! impl_serialize_uint {
    ($name: ident, $len: expr) => {
        impl SerializableUInt for $name {
            fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut slice = [0u8; 2 + 2 * $len];
                let bytes = $name::to_be_bytes(*self);
                ::impl_serde_macro::serialize::serialize_uint(&mut slice, &bytes, serializer)
            }
        }

        impl<'de> DeserializableUInt<'de> for $name {
            fn deserialize_eth_uint<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let mut bytes = [0u8; $len];
                let _ = impl_serde_macro::serialize::deserialize_check_len(
                    deserializer,
                    impl_serde_macro::serialize::ExpectedLen::Between(0, &mut bytes),
                )?;
                Ok(Self::from_le_bytes(bytes))
            }
        }
    };
}

impl_serialize_uint!(u8, 1);
impl_serialize_uint!(u16, 2);
impl_serialize_uint!(u32, 4);
// impl_serialize_uint!(u64, 8);
impl_serialize_uint!(u128, 16);

impl SerializableUInt for u64 {
    fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut slice = [0u8; 2 + 2 * 8];
        let bytes = Self::to_be_bytes(*self);
        impl_serde_macro::serialize::serialize_uint(&mut slice, &bytes, serializer)
    }
}

impl<'de> DeserializableUInt<'de> for u64 {
    fn deserialize_eth_uint<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut bytes = [0u8; 8];
        let wrote = impl_serde_macro::serialize::deserialize_check_len(
            deserializer,
            impl_serde_macro::serialize::ExpectedLen::Between(0, &mut bytes),
        )?;
        for i in 0..wrote {
            bytes[8 - wrote + i] = bytes[i];
            bytes[i] = 0;
        }
        Ok(Self::from_be_bytes(bytes))
    }
}
