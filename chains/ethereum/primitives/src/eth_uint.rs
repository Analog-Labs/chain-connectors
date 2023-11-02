// ignore clippy warnings from `construct_uint!` macro.
#![allow(
    clippy::pedantic,
    clippy::reversed_empty_ranges,
    clippy::assign_op_pattern,
    clippy::incorrect_clone_impl_on_copy_type
)]

#[cfg(feature = "with-codec")]
use impl_codec_macro::impl_uint_codec;
use impl_num_traits::impl_uint_num_traits;
#[cfg(feature = "with-rlp")]
use impl_rlp_macro::impl_uint_rlp;
#[cfg(feature = "with-serde")]
use impl_serde_macro::impl_uint_serde;
pub use primitive_types::{Error, U128, U256, U512};
pub use uint::{FromDecStrErr, FromStrRadixErr, FromStrRadixErrKind};

uint::construct_uint! {
    /// Unsigned 64-bit integer.
    pub struct U64(1);
}

impl U64 {
    /// Multiplies two 64-bit integers to produce full 128-bit integer.
    /// Overflow is not possible.
    #[inline(always)]
    pub fn full_mul(self, other: Self) -> primitive_types::U128 {
        primitive_types::U128(uint::uint_full_mul_reg!(U64, 1, self, other))
    }
}

impl_uint_num_traits!(U64, 1);
#[cfg(feature = "with-codec")]
impl_uint_codec!(U64, 1);
#[cfg(feature = "with-rlp")]
impl_uint_rlp!(U64, 1);
#[cfg(feature = "with-serde")]
impl_uint_serde!(U64, 1);

#[cfg(feature = "with-codec")]
impl ::scale_info::TypeInfo for U64 {
    type Identity = <u64 as ::scale_info::TypeInfo>::Identity;
    fn type_info() -> ::scale_info::Type {
        // Alias to u64 primitive type
        <u64 as ::scale_info::TypeInfo>::type_info()
    }
}

impl From<U64> for U128 {
    fn from(value: U64) -> Self {
        let U64(ref arr) = value;
        let mut ret = [0; 2];
        ret[0] = arr[0];
        Self(ret)
    }
}

impl<'a> From<&'a U64> for U128 {
    fn from(value: &'a U64) -> Self {
        let U64(ref arr) = value;
        let mut ret = [0; 2];
        ret[0] = arr[0];
        Self(ret)
    }
}

impl From<U64> for U256 {
    fn from(value: U64) -> Self {
        let U64(ref arr) = value;
        let mut ret = [0; 4];
        ret[0] = arr[0];
        Self(ret)
    }
}

impl<'a> From<&'a U64> for U256 {
    fn from(value: &'a U64) -> Self {
        let U64(ref arr) = value;
        let mut ret = [0; 4];
        ret[0] = arr[0];
        Self(ret)
    }
}

impl From<U64> for U512 {
    fn from(value: U64) -> Self {
        let U64(ref arr) = value;
        let mut ret = [0; 8];
        ret[0] = arr[0];
        Self(ret)
    }
}

impl<'a> From<&'a U64> for U512 {
    fn from(value: &'a U64) -> Self {
        let U64(ref arr) = value;
        let mut ret = [0; 8];
        ret[0] = arr[0];
        Self(ret)
    }
}
