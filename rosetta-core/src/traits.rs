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
pub trait Member: Debug + Send + Sync + Sized + PartialEq + Eq + Clone + 'static {}
impl<T: Debug + Send + Sync + Sized + PartialEq + Eq + Clone + 'static> Member for T {}

/// Super trait with all the attributes for a hashing output.
pub trait HashOutput:
    Member + Display + FromStr + core::hash::Hash + AsRef<[u8]> + AsMut<[u8]> + Copy + Ord + Default
{
}

pub type BlockNumber = u64;

pub trait Header: Clone + Send + Sync + Eq + Debug + 'static {
    /// Header hash type
    type Hash: HashOutput;

    /// Returns the header block number.
    fn number(&self) -> BlockNumber;

    /// Returns the hash of the header.
    fn hash(&self) -> Self::Hash;
}

/// Something that acts like a [`SignaturePayload`](Extrinsic::SignaturePayload) of an
/// [`Transaction`].
pub trait SignaturePayload {
    /// The type of the address that signed the extrinsic.
    ///
    /// Particular to a signed extrinsic.
    type SignatureAddress;

    /// The signature type of the extrinsic.
    ///
    /// Particular to a signed extrinsic.
    type Signature;

    /// The additional data that is specific to the signed extrinsic.
    ///
    /// Particular to a signed extrinsic.
    type SignatureExtra;
}

impl SignaturePayload for () {
    type SignatureAddress = ();
    type Signature = ();
    type SignatureExtra = ();
}

/// Something that acts like an `Extrinsic`.
pub trait Transaction: Sized {
    /// The function call.
    type Call;

    /// The payload we carry for signed transactions.
    ///
    /// Usually it will contain a `Signature` and
    /// may include some additional data that are specific to signed
    /// transaction.
    type SignaturePayload: SignaturePayload;

    /// Is this `Extrinsic` signed?
    /// If no information are available about signed/unsigned, `None` should be returned.
    fn is_signed(&self) -> Option<bool> {
        None
    }

    /// Create new instance of the extrinsic.
    ///
    /// Extrinsics can be split into:
    /// 1. Inherents (no signature; created by validators during block production)
    /// 2. Unsigned Transactions (no signature; represent "system calls" or other special kinds of
    /// calls) 3. Signed Transactions (with signature; a regular transactions with known origin)
    fn new(_call: Self::Call, _signed_data: Option<Self::SignaturePayload>) -> Option<Self> {
        None
    }
}

pub trait Block: Clone + Send + Sync + Eq + Debug + 'static {
    /// Type for extrinsics.
    type Transaction: Member + Transaction;
    /// Header type.
    type Header: Header<Hash = Self::Hash>;
    /// Block hash type.
    type Hash: HashOutput;

    /// Returns a reference to the header.
    fn header(&self) -> &Self::Header;

    /// Returns a reference to the list of transactions.
    fn transactions(&self) -> &[Self::Transaction];

    /// Returns the hash of the block.
    fn hash(&self) -> Self::Hash;
}

pub trait BlockchainConfig {
    const NAME: &'static str;
    const SYMBOL: &'static str;
    const BIP44: u32;
    const DEV: bool;

    const CHECKPOINT: Option<(BlockNumber, <<Self::Block as Block>::Header as Header>::Hash)>;

    type Block: Block;
    type Transaction: Clone + Send + Sync + 'static;
    type UnsignedTransaction: Clone + Send + Sync + 'static;
}

pub trait Query: Member {
    type Result: Member;
}

impl Query for () {
    type Result = ();
}

// pub enum QueryEnum<T: BlockchainConfig, CUSTOM> {
//     BlockHeaderByHash(<T::Block as Block>::Hash),
//     BlockByHash(T::Block),
//     Query(CUSTOM),
// }

// pub enum SubscribeEnum<T: BlockchainConfig, CUSTOM> {
//     NewHeads,
//     FinalizedHeads,
//     Subscribe(CUSTOM),
// }

// pub trait BlockchainClient {
//     type Config: BlockchainConfig;
//     type Error: std::error::Error + Send + Sync + 'static;
//     type TxParams: Send + Sync + 'static;

//     type QueryFuture<'a, Q>: core::future::Future<Output = Result<Q::Result, Self::Error>>
//         + Send
//         + 'a
//     where
//         Q: Query,
//         Self: 'a;

//     fn query<Q: Query>(&self, params: Q::Params) -> Self::QueryFuture<'_, Q>;

//     fn build_transaction(
//         &self,
//         params: Self::TxParams,
//     ) -> <Self::Config as BlockchainConfig>::Transaction;

//     fn submit_transaction(
//         &self,
//         tx: <Self::Config as BlockchainConfig>::Transaction,
//     ) -> Result<(), Self::Error>;

//     fn poll_next_event(
//         &self,
//         tx: <Self::Config as BlockchainConfig>::Transaction,
//     ) -> Result<(), Self::Error>;
// }
