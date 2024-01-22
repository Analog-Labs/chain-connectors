use rosetta_core::traits::Member;
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData};
// use rosetta_config_polkadot::metadata::westend::dev as westend_dev_metadata;
use subxt::{
    config::{ExtrinsicParams, Hasher, Header},
    ext::{codec::Encode, scale_decode::DecodeAsType, scale_encode::EncodeAsType},
    utils::{AccountId32, MultiAddress},
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

    /// These parameters can be provided to the constructor along with
    /// some default parameters that `subxt` understands, in order to
    /// help construct your [`Self::ExtrinsicParams`] object.
    type OtherParams: Default + Send + Sync + 'static;

    /// This type defines the extrinsic extra and additional parameters.
    type ExtrinsicParams: ExtrinsicParams<SubxtConfigAdapter<Self>, OtherParams = Self::OtherParams>;

    /// This is used to identify an asset in the `ChargeAssetTxPayment` signed extension.
    type AssetId: Debug + Clone + Encode + DecodeAsType + EncodeAsType;

    type AccountInfo: Member + subxt::metadata::DecodeWithMetadata;

    type TransferKeepAlive: Member
        + subxt::blocks::StaticExtrinsic
        + subxt::ext::scale_encode::EncodeAsFields;

    type Pair: subxt::tx::Signer<SubxtConfigAdapter<Self>> + Send + Sync + 'static;

    fn account_info(
        account: impl Borrow<AccountId32>,
    ) -> ::subxt::storage::address::Address<
        ::subxt::storage::address::StaticStorageMapKey,
        Self::AccountInfo,
        ::subxt::storage::address::Yes,
        ::subxt::storage::address::Yes,
        (),
    >;

    fn transfer_keep_alive(
        dest: MultiAddress<AccountId32, ()>,
        value: u128,
    ) -> ::subxt::tx::Payload<Self::TransferKeepAlive>;

    fn other_params(
    ) -> <Self::ExtrinsicParams as ExtrinsicParams<SubxtConfigAdapter<Self>>>::OtherParams;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageQuery {
    /// Version of the runtime specification.
    spec_version: Option<u32>,
    // Raw storage-key
    address: Vec<u8>,
}

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
