// ignore clippy warnings in `construct_fixed_hash!` macro.
#![allow(
    clippy::pedantic,
    clippy::reversed_empty_ranges,
    clippy::assign_op_pattern,
    clippy::incorrect_clone_impl_on_copy_type
)]
use fixed_hash::*;
#[cfg(feature = "with-codec")]
use impl_codec_macro::impl_fixed_hash_codec;
#[cfg(feature = "with-rlp")]
use impl_rlp_macro::impl_fixed_hash_rlp;
#[cfg(feature = "with-serde")]
use impl_serde_macro::impl_fixed_hash_serde;
pub use primitive_types::{H128, H160, H256, H384, H512, H768};

// Aliases for Ethereum types.
pub type Address = H160;
pub type TxHash = H256;
pub type Secret = H256;
pub type Public = H512;
pub type Signature = H520;

macro_rules! impl_hash {
    ($hash: ident, $n_bytes: expr) => {
        construct_fixed_hash! { pub struct $hash($n_bytes); }

        #[cfg(feature = "with-codec")]
        impl_fixed_hash_codec!($hash, $n_bytes);
        #[cfg(feature = "with-rlp")]
        impl_fixed_hash_rlp!($hash, $n_bytes);
        #[cfg(feature = "with-serde")]
        impl_fixed_hash_serde!($hash, $n_bytes);

        #[cfg(feature = "with-codec")]
        impl ::scale_info::TypeInfo for $hash {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                use ::scale_info::{build::Fields, Path, Type};

                Type::builder()
                    .path(Path::new(stringify!($hash), module_path!()))
                    .type_params(Vec::new())
                    .composite(Fields::unnamed().field(|f| {
                        f.ty::<[u8; $n_bytes]>().type_name(concat!("[u8; ", $n_bytes, "]"))
                    }))
            }
        }
    };
}

impl_hash!(H32, 4);
impl_hash!(H64, 8);
impl_hash!(H520, 65);
