use core::{
    default::Default,
    fmt::{Debug, Display},
    str::FromStr,
};

/// Macro for creating `Maybe*` marker traits.
///
/// Such a maybe-marker trait requires the given bound when `feature = std` and doesn't require
/// the bound on `no_std`. This is useful for situations where you require that a type implements
/// a certain trait with `feature = std`, but not on `no_std`.
///
/// # Example
///
/// ```
/// sp_core::impl_maybe_marker! {
///     /// A marker for a type that implements `Debug` when `feature = std`.
///     trait MaybeDebug: std::fmt::Debug;
///     /// A marker for a type that implements `Debug + Display` when `feature = std`.
///     trait MaybeDebugDisplay: std::fmt::Debug, std::fmt::Display;
/// }
/// ```
macro_rules! impl_maybe_marker {
	(
		$(
			$(#[$doc:meta] )+
			trait $trait_name:ident: $( $trait_bound:path ),+;
		)+
	) => {
		$(
			$(#[$doc])+
			#[cfg(feature = "std")]
			pub trait $trait_name: $( $trait_bound + )+ {}
			#[cfg(feature = "std")]
			impl<T: $( $trait_bound + )+> $trait_name for T {}

			$(#[$doc])+
			#[cfg(not(feature = "std"))]
			pub trait $trait_name {}
			#[cfg(not(feature = "std"))]
			impl<T> $trait_name for T {}
		)+
	}
}

impl_maybe_marker!(
    /// A type that implements Display when in std environment.
    trait MaybeDisplay: core::fmt::Display;

    /// A type that implements FromStr when in std environment.
    trait MaybeFromStr: core::str::FromStr;

    /// A type that implements Hash when in std environment.
    trait MaybeHash: core::hash::Hash;
);

/// A type that can be used in runtime structures.
pub trait Member: Send + Sync + Sized + Debug + Eq + PartialEq + Clone + 'static {}
impl<T: Send + Sync + Sized + Debug + Eq + PartialEq + Clone + 'static> Member for T {}

/// Super trait with all the attributes for a hashing output.
pub trait HashOutput:
    Member + Display + FromStr + core::hash::Hash + AsRef<[u8]> + AsMut<[u8]> + Copy + Ord + Default
{
}

pub trait Header: Clone + Send + Sync + Eq + Debug + 'static {
    /// Header number.
    type Number;

    /// Header hash type
    type Hash: HashOutput;
}

/// Something that acts like an `Extrinsic`.
pub trait Transaction: Sized {
    /// The function call.
    type Call;
}

pub trait Block {
    /// Type for extrinsics.
    type Transaction: Member + Transaction;
    /// Header type.
    type Header: Header<Hash = Self::Hash>;
    /// Block hash type.
    type Hash: HashOutput;
}

pub trait BlockchainPrimitives {
    type Block: Clone + Send + Sync + 'static;
    type Transaction: Clone + Send + Sync + 'static;
}
