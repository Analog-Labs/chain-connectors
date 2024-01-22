#![allow(dead_code)]
use crate::types::{BlockIdentifier, ClientConfig, SubxtConfigAdapter};
use std::{borrow::Borrow, future::Future, sync::Arc};
use subxt::{
    backend::{
        rpc::{RpcClient, RpcClientT},
        RuntimeVersion,
    },
    blocks::BlockRef,
    metadata::Metadata,
    utils::AccountId32,
};

type OnlineClient<T> = subxt::OnlineClient<SubxtConfigAdapter<T>>;
type LegacyRpcMethods<T> = subxt::backend::legacy::LegacyRpcMethods<SubxtConfigAdapter<T>>;
type LegacyBackend<T> = subxt::backend::legacy::LegacyBackend<SubxtConfigAdapter<T>>;
type PairSigner<T> = subxt::tx::PairSigner<SubxtConfigAdapter<T>, <T as ClientConfig>::Pair>;
type Block<T> = subxt::backend::legacy::rpc_methods::Block<SubxtConfigAdapter<T>>;
type BlockDetails<T> = subxt::backend::legacy::rpc_methods::BlockDetails<SubxtConfigAdapter<T>>;

pub struct SubstrateClient<T: ClientConfig> {
    client: OnlineClient<T>,
    rpc_methods: LegacyRpcMethods<T>,
}

impl<T: ClientConfig> SubstrateClient<T> {
    /// Creates a new polkadot client using the provided `config` and connects to `addr`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn from_client<C: RpcClientT>(client: C) -> anyhow::Result<Self> {
        let rpc_client = RpcClient::new(client);
        let rpc_methods = LegacyRpcMethods::<T>::new(rpc_client.clone());
        let backend = LegacyBackend::<T>::new(rpc_client);
        let client = OnlineClient::<T>::from_backend(Arc::new(backend)).await?;
        Ok(Self { client, rpc_methods })
    }

    pub const fn rpc_methods(&self) -> &LegacyRpcMethods<T> {
        &self.rpc_methods
    }

    pub const fn client(&self) -> &OnlineClient<T> {
        &self.client
    }

    pub fn account_info(
        &self,
        account: impl Borrow<AccountId32>,
        block_ref: BlockRef<T::Hash>,
    ) -> impl Future<Output = anyhow::Result<T::AccountInfo>> + Sized + Send + '_ {
        let account = account.borrow();
        let tx = T::account_info(account);
        async move {
            let account = self.client.storage().at(block_ref).fetch_or_default(&tx).await?;
            Ok(account)
        }
    }

    pub async fn block(
        &self,
        block_identifier: BlockIdentifier<T::Hash>,
    ) -> anyhow::Result<Option<BlockDetails<T>>> {
        use subxt::backend::legacy::rpc_methods::BlockNumber;
        let block_hash = match block_identifier {
            BlockIdentifier::Hash(block_hash) => block_hash,
            BlockIdentifier::Number(block_number) => {
                let Some(block_hash) = self
                    .rpc_methods
                    .chain_get_block_hash(Some(BlockNumber::Number(block_number)))
                    .await?
                else {
                    return Ok(None);
                };
                block_hash
            },
        };
        self.rpc_methods
            .chain_get_block(Some(block_hash))
            .await
            .map_err(anyhow::Error::from)
    }

    async fn faucet(
        &self,
        signer: T::Pair,
        dest: subxt::utils::MultiAddress<AccountId32, ()>,
        value: u128,
    ) -> anyhow::Result<T::Hash> {
        let tx = T::transfer_keep_alive(dest, value);
        let hash = self
            .client
            .tx()
            .sign_and_submit_then_watch(&tx, &signer, T::other_params())
            .await?
            .wait_for_finalized_success()
            .await?
            .extrinsic_hash();
        Ok(hash)
    }

    fn runtime_version(&self) -> RuntimeVersion {
        self.client.runtime_version()
    }

    fn metadata(&self) -> Metadata {
        self.client.metadata()
    }

    fn genesis_hash(&self) -> T::Hash {
        self.client.genesis_hash()
    }
}
