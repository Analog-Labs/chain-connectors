use crate::{
    eth_hash::{H128, H256, H32, H64},
    eth_uint::U256,
    rstd::{
        default::Default, format, mem, option::Option, result::Result, string::String, vec::Vec,
    },
};
use impl_serde_macro::serialize::{deserialize_check_len, serialize_uint, ExpectedLen};
use num_rational::Rational64;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// serde functions for converting `u64` to and from hexadecimal string
pub mod uint_to_hex {
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

/// serde functions for converting `u64` to and from hexadecimal string
pub mod bytes_to_hex {
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

/// Deserialize that always returns `Vec<T>` regardless if the field is present or not
///
/// # Errors
/// returns an error if fails to deserialize T
#[cfg(feature = "serde")]
#[must_use]
pub const fn default_empty_vec<T>() -> Vec<T> {
    Vec::new()
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

impl SerializableNumber for H32 {
    fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = u32::from_be_bytes(self.0);
        <u32 as SerializableNumber>::serialize_eth_uint(&value, serializer)
    }
}

impl SerializableNumber for H64 {
    fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = u64::from_be_bytes(self.0);
        <u64 as SerializableNumber>::serialize_eth_uint(&value, serializer)
    }
}

impl SerializableNumber for H128 {
    fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = u128::from_be_bytes(self.0);
        <u128 as SerializableNumber>::serialize_eth_uint(&value, serializer)
    }
}

impl SerializableNumber for H256 {
    fn serialize_eth_uint<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = U256::from_big_endian(&self.0);
        <U256 as Serialize>::serialize(&value, serializer)
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
        impl_serde_macro::serialize::serialize(self.as_ref(), serializer)
    }
}

impl<const N: usize> SerializableBytes for [u8; N] {
    fn serialize_bytes<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        impl_serde_macro::serialize::serialize(self.as_ref(), serializer)
    }
}

impl SerializableBytes for [u8] {
    fn serialize_bytes<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        impl_serde_macro::serialize::serialize(self, serializer)
    }
}

impl SerializableBytes for u32 {
    fn serialize_bytes<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        impl_serde_macro::serialize::serialize(&self.to_be_bytes(), serializer)
    }
}

impl SerializableBytes for u64 {
    fn serialize_bytes<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        impl_serde_macro::serialize::serialize(&self.to_be_bytes(), serializer)
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
        impl_serde_macro::serialize::deserialize(deserializer)
    }
}

impl<'de, const N: usize> DeserializableBytes<'de> for [u8; N] {
    fn deserialize_bytes<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut output = [0u8; N];
        let bytes = impl_serde_macro::serialize::deserialize(deserializer)?;
        if bytes.len() != N {
            return Err(serde::de::Error::custom(format!("invalid length: {}", bytes.len())));
        }
        output.copy_from_slice(bytes.as_ref());
        Ok(output)
    }
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
                    return Err(serde::de::Error::custom(format!(
                        "invalid length: {}",
                        bytes.len()
                    )));
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

#[inline]
const fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

#[inline]
const fn parse_digit(c: u8) -> Option<u8> {
    if is_digit(c) {
        Some(c - b'0')
    } else {
        None
    }
}

#[inline]
const fn next_digit(chars: &[u8]) -> Option<(u8, &[u8])> {
    let Some((digit, rest)) = chars.split_first() else {
        return None;
    };
    if let Some(num) = parse_digit(*digit) {
        Some((num, rest))
    } else {
        None
    }
}

#[inline]
const fn next_int(chars: &[u8], initial: i64) -> Option<(i64, &[u8])> {
    let Some((value, mut chars)) = next_digit(chars) else {
        return None;
    };
    let mut value = (initial * 10) + value as i64;
    while let Some((d, rest)) = next_digit(chars) {
        chars = rest;
        value *= 10;
        value += d as i64;
    }
    Some((value, chars))
}

macro_rules! parse_num {
    ($body: expr, $val: expr) => {
        match next_int($body, $val) {
            Some(v) => v,
            None => return None,
        }
    };
}

const fn parse_numeric(chars: &[u8]) -> Option<(i64, usize)> {
    match chars {
        [b'-', rest @ ..] => match parse_num!(rest, 0) {
            (num, []) => Some((-num, 0)),
            (num, [b'.', fract @ ..]) => match parse_num!(fract, num) {
                (den, []) => Some((den, fract.len())),
                _ => None,
            },
            _ => None,
        },
        [b'0', rest @ ..] => match rest {
            [b'.', fract @ ..] => match parse_num!(rest, 0) {
                (den, []) => Some((den, fract.len())),
                _ => None,
            },
            _ => None,
        },
        rest => match parse_num!(rest, 0) {
            (num, []) => Some((num, 1)),
            (num, [b'.', fract @ ..]) => match parse_num!(fract, num) {
                (den, []) => Some((den, fract.len())),
                _ => None,
            },
            _ => None,
        },
    }
}

/// serde functions for converting numeric value to and from rational number
pub mod numeric_to_rational {
    use super::{DeserializableRational, SerializableRational};
    use serde::{Deserializer, Serializer};

    /// # Errors
    /// Returns `Err` if the value cannot be encoded as bytes
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: SerializableRational,
        S: Serializer,
    {
        <T as SerializableRational>::serialize_as_rational(value, serializer)
    }

    /// # Errors
    /// Returns `Err` source is not a valid hexadecimal string
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: DeserializableRational<'de>,
        D: Deserializer<'de>,
    {
        <T as DeserializableRational<'de>>::deserialize_as_rational(deserializer)
    }
}

pub trait SerializableRational {
    #[allow(clippy::missing_errors_doc)]
    fn serialize_as_rational<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

pub trait DeserializableRational<'de>: Sized {
    /// Deserialize a rational value
    /// # Errors
    /// should never fails
    fn deserialize_as_rational<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl SerializableRational for Rational64 {
    fn serialize_as_rational<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = RationalNumber(*self);
        <RationalNumber as Serialize>::serialize(&value, serializer)
    }
}

impl SerializableRational for Vec<Rational64> {
    fn serialize_as_rational<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Safety: `Vec<Rational64>` and `Vec<RationalNumber>` have the same memory layout
        #[allow(clippy::transmute_undefined_repr)]
        let value = unsafe { &*core::ptr::from_ref::<Self>(self).cast::<Vec<RationalNumber>>() };
        <Vec<RationalNumber> as Serialize>::serialize(value, serializer)
    }
}

impl<'de> DeserializableRational<'de> for Rational64 {
    fn deserialize_as_rational<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ratio = <RationalNumber as Deserialize<'de>>::deserialize(deserializer)?;
        Ok(ratio.0)
    }
}

impl<'de> DeserializableRational<'de> for Vec<Rational64> {
    fn deserialize_as_rational<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ratio = <Vec<RationalNumber> as Deserialize<'de>>::deserialize(deserializer)?;
        // Safety: `Vec<Rational64>` and `Vec<RationalNumber>` have the same memory layout
        #[allow(clippy::transmute_undefined_repr)]
        let ratio = unsafe { mem::transmute::<Vec<RationalNumber>, Self>(ratio) };
        Ok(ratio)
    }
}

#[repr(transparent)]
struct RationalNumber(pub Rational64);

impl Serialize for RationalNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use num_traits::ToPrimitive;
        let float = self.0.to_f64().unwrap_or(f64::NAN);
        <f64 as Serialize>::serialize(&float, serializer)
    }
}

impl<'de> Deserialize<'de> for RationalNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let numeric = <StringifiedNumeric as Deserialize<'de>>::deserialize(deserializer)?;
        let ratio = match numeric {
            StringifiedNumeric::String(s) => {
                let Some((num, decimals)) = parse_numeric(s.as_bytes()) else {
                    return Err(serde::de::Error::custom("invalid character"));
                };
                #[allow(clippy::cast_possible_truncation)]
                Rational64::new(num, 10i64.pow(decimals as u32))
            },
            StringifiedNumeric::Num(numer) => Rational64::new(numer, 1),
            StringifiedNumeric::Float(float) => {
                let Some(ratio) = num_rational::Rational64::approximate_float(float) else {
                    return Err(serde::de::Error::custom("invalid fraction"));
                };
                ratio
            },
        };
        Ok(Self(ratio))
    }
}

/// Helper type to parse numeric strings, `u64` and `U256`
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum StringifiedNumeric {
    String(String),
    Num(i64),
    Float(f64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::ToPrimitive;

    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    const TEST_CASES: [f64; 5] = [
        0.529_074_766_666_666_6,
        0.492_404_533_333_333_34,
        0.461_557_6,
        0.494_070_833_333_333_35,
        0.466_905_3,
    ];

    #[test]
    fn deserialize_rational_works() {
        for float in TEST_CASES {
            let ratio = serde_json::from_str::<RationalNumber>(&float.to_string()).unwrap().0;
            let value = ratio.to_f64().unwrap();
            assert!((value - float).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn serialize_rational_works() {
        for float in TEST_CASES {
            let ratio = RationalNumber(Rational64::approximate_float(float).unwrap());
            assert_eq!(serde_json::to_string(&ratio).unwrap(), float.to_string());
        }
    }
}
