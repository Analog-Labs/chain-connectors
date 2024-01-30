/// Creates a wrapper type around a type. Necessary for implementing traits on types that are not
/// defined in the current crate.
macro_rules! impl_wrapper {
    ( $(#[$attr:meta])* $visibility:vis struct $name:ident ($original:ty); ) => {
		$crate::util::impl_wrapper!{ @construct $(#[$attr])* $visibility struct $name ($original); }
	};

    ( @construct $(#[$attr:meta])* $visibility:vis struct $name:ident ( $original:ty ); ) => {
        #[repr(transparent)]
		$(#[$attr])*
		$visibility struct $name (pub $original);

        #[cfg(feature = "serde")]
        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> crate::rstd::result::Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                <$original as serde::Serialize>::serialize(&self.0, serializer)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> crate::rstd::result::Result<$name, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                <$original as serde::Deserialize<'de>>::deserialize(deserializer).map(Self)
            }
        }

        #[cfg(feature = "scale-info")]
        impl scale_info::TypeInfo for $name {
            type Identity = <$original as scale_info::TypeInfo>::Identity;

            fn type_info() -> scale_info::Type {
                <$original as scale_info::TypeInfo>::type_info()
            }
        }

        #[cfg(feature = "scale-codec")]
        impl parity_scale_codec::Encode for $name {
            fn size_hint(&self) -> usize {
                <$original as parity_scale_codec::Encode>::size_hint(&self.0)
            }

            fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, dest: &mut T) {
                <$original as parity_scale_codec::Encode>::encode_to(&self.0, dest);
            }

            fn encode(&self) -> Vec<u8> {
                <$original as parity_scale_codec::Encode>::encode(&self.0)
            }

            fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
                <$original as parity_scale_codec::Encode>::using_encoded(&self.0, f)
            }

            fn encoded_size(&self) -> usize {
                <$original as parity_scale_codec::Encode>::encoded_size(&self.0)
            }
        }

        #[cfg(feature = "scale-codec")]
        impl parity_scale_codec::Decode for $name {
            fn decode<I: parity_scale_codec::Input>(input: &mut I) -> crate::rstd::result::Result<Self, parity_scale_codec::Error> {
                let value = <$original as parity_scale_codec::Decode>::decode(input)?;
                Ok(Self(value))
            }

            fn decode_into<I: parity_scale_codec::Input>(input: &mut I, dst: &mut core::mem::MaybeUninit<Self>) -> crate::rstd::result::Result<parity_scale_codec::DecodeFinished, parity_scale_codec::Error> {
                // Safety: Self is repr(transparent) over the inner type, so have the same ABI and memory layout, we can safely cast it
                let dst = unsafe {
                    &mut *(dst as *mut core::mem::MaybeUninit<$name>).cast::<core::mem::MaybeUninit<$original>>()
                };
                <$original as parity_scale_codec::Decode>::decode_into(input, dst)
            }

            fn skip<I: parity_scale_codec::Input>(input: &mut I) -> crate::rstd::result::Result<(), parity_scale_codec::Error> {
                <$original as parity_scale_codec::Decode>::skip(input)
            }

            fn encoded_fixed_size() -> Option<usize> {
                <$original as parity_scale_codec::Decode>::encoded_fixed_size()
            }
        }

        impl From<$original> for $name {
            fn from(value: $original) -> Self {
                Self(value)
            }
        }

        impl <'a> From<&'a $original> for &'a $name {
            fn from(value: &'a $original) -> Self {
                unsafe {
                    // Safety: $name is repr(transparent) over the $original type, so have the same ABI as the original
                    // so can safely cast it
                    &*(value as *const $original).cast::<$name>()
                }
            }
        }

        impl <'a> From<&'a mut $original> for &'a mut $name {
            fn from(value: &'a mut $original) -> Self {
                unsafe {
                    // Safety: $name is repr(transparent) over the $original type, so have the same ABI as the original
                    // so can safely cast it
                    &mut *(value as *mut $original).cast::<$name>()
                }
            }
        }

        impl From<$name> for $original {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl <'a> From<&'a $name> for &'a $original {
            fn from(value: &'a $name) -> Self {
                unsafe {
                    // Safety: $name is repr(transparent) over the $original type, so have the same ABI as the original
                    // so can safely cast it
                    &*(value as *const $name).cast::<$original>()
                }
            }
        }

        impl <'a> From<&'a mut $name> for &'a mut $original {
            fn from(value: &'a mut $name) -> Self {
                unsafe {
                    // Safety: $name is repr(transparent) over the $original type, so have the same ABI as the original
                    // so can safely cast it
                    &mut *(value as *mut $name).cast::<$original>()
                }
            }
        }
    };
}

pub(crate) use impl_wrapper;
