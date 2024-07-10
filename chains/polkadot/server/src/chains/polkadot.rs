use crate::types::{ClientConfig, SubxtConfigAdapter};
use rosetta_config_polkadot::metadata::polkadot::dev;
use std::borrow::Borrow;
use subxt::{
    config::polkadot,
    ext::subxt_core::{
        storage::address::{StaticAddress, StaticStorageKey},
        tx::payload::StaticPayload,
        utils::{AccountId32, MultiAddress, Yes},
    },
};

pub type Config = SubxtConfigAdapter<PolkadotConfig>;
pub type ExtrinsicParams = polkadot::PolkadotExtrinsicParams<Config>;
pub type OtherParams = <ExtrinsicParams as subxt::config::ExtrinsicParams<Config>>::Params;
pub type PairSigner = subxt::tx::PairSigner<Config, sp_keyring::sr25519::sr25519::Pair>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolkadotConfig;

impl ClientConfig for PolkadotConfig {
    type Hash = <polkadot::PolkadotConfig as subxt::Config>::Hash;
    type AccountId = <polkadot::PolkadotConfig as subxt::Config>::AccountId;
    type Address = <polkadot::PolkadotConfig as subxt::Config>::Address;
    type Signature = <polkadot::PolkadotConfig as subxt::Config>::Signature;
    type Hasher = <polkadot::PolkadotConfig as subxt::Config>::Hasher;
    type Header = <polkadot::PolkadotConfig as subxt::Config>::Header;
    type OtherParams = OtherParams;
    type ExtrinsicParams = ExtrinsicParams;
    type AssetId = <polkadot::PolkadotConfig as subxt::Config>::AssetId;

    type AccountInfo = dev::runtime_types::frame_system::AccountInfo<
        u32,
        dev::runtime_types::pallet_balances::types::AccountData<u128>,
    >;

    type TransferKeepAlive = dev::balances::calls::types::TransferKeepAlive;

    type Pair = PairSigner;

    fn account_info(
        account: impl Borrow<AccountId32>,
    ) -> StaticAddress<StaticStorageKey<Self::AccountId>, Self::AccountInfo, Yes, Yes, ()> {
        dev::storage().system().account(account)
    }

    fn transfer_keep_alive(
        dest: MultiAddress<AccountId32, ()>,
        value: u128,
    ) -> StaticPayload<Self::TransferKeepAlive> {
        dev::tx().balances().transfer_keep_alive(dest, value)
    }

    fn other_params() -> OtherParams {
        OtherParams::default()
    }
}
