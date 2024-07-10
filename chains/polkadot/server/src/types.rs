use rosetta_core::{traits::Member, types::PartialBlockIdentifier};
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData};
use subxt::{
    blocks::StaticExtrinsic,
    config::{ExtrinsicParams, Hasher, Header},
    ext::{
        codec::Encode,
        scale_decode::DecodeAsType,
        scale_encode::{EncodeAsFields, EncodeAsType},
        subxt_core::{
            config::BlockHash,
            metadata::DecodeWithMetadata,
            storage::address::{StaticAddress, StaticStorageKey},
            tx::{payload::StaticPayload, signer::Signer},
            utils::{AccountId32, MultiAddress, Yes},
        },
    },
    Config as SubxtConfig,
};

pub trait ClientConfig: Debug + Clone + PartialEq + Eq + Sized + Send + Sync + 'static {
    /// The output of the `Hasher` function.
    type Hash: BlockHash;

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
    type ExtrinsicParams: ExtrinsicParams<SubxtConfigAdapter<Self>, Params = Self::OtherParams>;

    /// This is used to identify an asset in the `ChargeAssetTxPayment` signed extension.
    type AssetId: Debug + Clone + Encode + DecodeAsType + EncodeAsType;

    type AccountInfo: Member + DecodeWithMetadata;

    type TransferKeepAlive: Member + StaticExtrinsic + EncodeAsFields;

    type Pair: Signer<SubxtConfigAdapter<Self>> + Send + Sync + 'static;

    fn account_info(
        account: impl Borrow<AccountId32>,
    ) -> StaticAddress<StaticStorageKey<Self::AccountId>, Self::AccountInfo, Yes, Yes, ()>;

    fn transfer_keep_alive(
        dest: MultiAddress<AccountId32, ()>,
        value: u128,
    ) -> StaticPayload<Self::TransferKeepAlive>;

    fn other_params() -> Self::OtherParams;
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
    Latest,
    Finalized,
}

impl<T> From<rosetta_core::types::BlockIdentifier> for BlockIdentifier<T>
where
    T: From<[u8; 32]>,
{
    fn from(block_identifier: rosetta_core::types::BlockIdentifier) -> Self {
        Self::Hash(T::from(block_identifier.hash))
    }
}

impl<T> From<PartialBlockIdentifier> for BlockIdentifier<T>
where
    T: From<[u8; 32]>,
{
    fn from(block_identifier: PartialBlockIdentifier) -> Self {
        match block_identifier {
            PartialBlockIdentifier { hash: Some(block_hash), .. } => {
                Self::Hash((block_hash).into())
            },
            PartialBlockIdentifier { index: Some(block_number), .. } => Self::Number(block_number),
            PartialBlockIdentifier { hash: None, index: None } => Self::Latest,
        }
    }
}

impl<T> From<&PartialBlockIdentifier> for BlockIdentifier<T>
where
    T: From<[u8; 32]>,
{
    fn from(block_identifier: &PartialBlockIdentifier) -> Self {
        match block_identifier {
            PartialBlockIdentifier { hash: Some(block_hash), .. } => {
                Self::Hash((*block_hash).into())
            },
            PartialBlockIdentifier { index: Some(block_number), .. } => Self::Number(*block_number),
            PartialBlockIdentifier { hash: None, index: None } => Self::Latest,
        }
    }
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
