use rosetta_core::traits::Member;
use std::{fmt::Debug, marker::PhantomData};
// use rosetta_config_polkadot::metadata::westend::dev as westend_dev_metadata;
use subxt::{
    config::{ExtrinsicParams, Hasher, Header},
    ext::{codec::Encode, scale_decode::DecodeAsType, scale_encode::EncodeAsType},
    Config as SubxtConfig,
};

pub trait ClientConfig: Debug + Clone + PartialEq + Eq + Sized + Send + Sync + 'static {
    /// The output of the `Hasher` function.
    type Hash: subxt::config::BlockHash;

    /// The account ID type.
    type AccountId: Member + Encode;

    /// The address type.
    type Address: Member + Encode + From<Self::AccountId>;

    /// The signature type.
    type Signature: Member + Encode;

    /// The hashing system (algorithm) being used in the runtime (e.g. Blake2).
    type Hasher: Debug + Hasher<Output = Self::Hash>;

    /// The block header.
    type Header: Member + Header<Hasher = Self::Hasher> + Send + serde::de::DeserializeOwned;

    /// This type defines the extrinsic extra and additional parameters.
    type ExtrinsicParams: ExtrinsicParams<SubxtConfigAdapter<Self>>;

    /// This is used to identify an asset in the `ChargeAssetTxPayment` signed extension.
    type AssetId: Debug + Clone + Encode + DecodeAsType + EncodeAsType;

    type AccountInfo: Member;
}

pub struct SubxtConfigAdapter<T>(PhantomData<T>);

impl<T> SubxtConfig for SubxtConfigAdapter<T>
where
    T: ClientConfig,
{
    type Hash = <T as ClientConfig>::Hash;

    type AccountId = <T as ClientConfig>::AccountId;

    type Address = <T as ClientConfig>::Address;

    type Signature = <T as ClientConfig>::Signature;

    type Hasher = <T as ClientConfig>::Hasher;

    type Header = <T as ClientConfig>::Header;

    type ExtrinsicParams = <T as ClientConfig>::ExtrinsicParams;

    type AssetId = <T as ClientConfig>::AssetId;
}

// // westend_dev_metadata::runtime_types::westend_runtime::

// /// A concrete storage address. This can be created from static values (ie those generated
// /// via the `subxt` macro) or dynamic values via [`dynamic`].
// #[derive(Derivative)]
// pub struct Address<StorageKey, ReturnTy, Fetchable, Defaultable, Iterable> {
//     pallet_name: Cow<'static, str>,
//     entry_name: Cow<'static, str>,
//     storage_entry_keys: Vec<StorageKey>,
//     validation_hash: Option<[u8; 32]>,
//     _marker: std::marker::PhantomData<(ReturnTy, Fetchable, Defaultable, Iterable)>,
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageQuery {
    /// Version of the runtime specification.
    spec_version: Option<u32>,
    // Raw storage-key
    address: Vec<u8>,
}

// type AccountId = ::subxt::utils::AccountId32;
// type AccountData =
// westend_dev_metadata::runtime_types::pallet_balances::types::AccountData<u128>; type AccountInfo
// = westend_dev_metadata::runtime_types::frame_system::AccountInfo<u32, AccountData>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockIdentifier<BlockHash> {
    Number(u64),
    Hash(BlockHash),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query<T: ClientConfig> {
    AccountInfo(T::AccountId),
    GetBlock(BlockIdentifier<T::Hash>),
    Storage(StorageQuery),
}

impl<T: ClientConfig> rosetta_core::traits::Query for Query<T> {
    type Result = QueryResult<T>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryResult<T: ClientConfig> {
    AccountInfo(T::AccountInfo),
    GetBlock(u32),
    Storage(Vec<u8>),
}

// pub fn teste() {
//     let res = westend_dev_metadata::storage().balances().account(0);
//     westend_dev_metadata::apis().
//     ::subxt::storage::address::Address;

//     ::subxt::storage::address::StaticStorageMapKey;
// }
