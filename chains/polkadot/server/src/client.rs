use crate::types::{BlockIdentifier, ClientConfig, SubxtConfigAdapter};
use anyhow::Context;
use std::{borrow::Borrow, future::Future, sync::Arc};
use subxt::{
    backend::rpc::{RpcClient, RpcClientT},
    blocks::BlockRef,
    client::RuntimeVersion,
    metadata::Metadata,
    utils::AccountId32,
};

type Config<T> = SubxtConfigAdapter<T>;
type OnlineClient<T> = subxt::OnlineClient<Config<T>>;
type LegacyRpcMethods<T> = subxt::backend::legacy::LegacyRpcMethods<Config<T>>;
type BlockDetails<T> = subxt::backend::legacy::rpc_methods::BlockDetails<Config<T>>;

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
        let backend = subxt::backend::legacy::LegacyBackendBuilder::new().build(rpc_client);
        let client = OnlineClient::<T>::from_backend(Arc::new(backend)).await?;
        Ok(Self { client, rpc_methods })
    }

    pub const fn client(&self) -> &OnlineClient<T> {
        &self.client
    }

    async fn block_identifier_to_hash(
        &self,
        block_identifier: BlockIdentifier<T::Hash>,
    ) -> anyhow::Result<T::Hash> {
        use subxt::backend::legacy::rpc_methods::BlockNumber;
        let block_hash = match block_identifier {
            BlockIdentifier::Hash(block_hash) => block_hash,
            BlockIdentifier::Number(block_number) => {
                let Some(block_hash) = self
                    .rpc_methods
                    .chain_get_block_hash(Some(BlockNumber::Number(block_number)))
                    .await?
                else {
                    anyhow::bail!("block not found: {block_identifier:?}");
                };
                block_hash
            },
            BlockIdentifier::Latest => self
                .rpc_methods
                .chain_get_block_hash(None)
                .await?
                .context("latest block not found")?,
            BlockIdentifier::Finalized => self.rpc_methods.chain_get_finalized_head().await?,
        };
        Ok(block_hash)
    }

    pub fn account_info(
        &self,
        account: impl Borrow<AccountId32>,
        block_identifier: impl Into<BlockIdentifier<T::Hash>>,
    ) -> impl Future<Output = anyhow::Result<T::AccountInfo>> + Sized + Send + '_ {
        let account = account.borrow();
        let tx = T::account_info(account);
        let block_identifier = block_identifier.into();
        async move {
            let block_hash = self.block_identifier_to_hash(block_identifier).await?;
            let account = self
                .client
                .storage()
                .at(BlockRef::from_hash(block_hash))
                .fetch_or_default(&tx)
                .await?;
            Ok(account)
        }
    }

    // pub fn block(
    //     &self,
    //     block_identifier: impl Into<BlockIdentifier<T::Hash>> + Send,
    // ) -> impl Future<Output = anyhow::Result<Block<T>>> + Sized + Send + '_ {
    //     let block_identifier = block_identifier.into();
    //     async move {
    //         let block_hash = self.block_identifier_to_hash(block_identifier).await?;
    //         let block = self.client.blocks().at(BlockRef::from_hash(block_hash)).await?;
    //         Ok(block)
    //     }
    // }

    pub fn block_details(
        &self,
        block_identifier: impl Into<BlockIdentifier<T::Hash>> + Send,
    ) -> impl Future<Output = anyhow::Result<Option<BlockDetails<T>>>> + Sized + Send + '_ {
        let block_identifier = block_identifier.into();
        async move {
            let block_hash = self.block_identifier_to_hash(block_identifier).await?;
            self.rpc_methods
                .chain_get_block(Some(block_hash))
                .await
                .map_err(anyhow::Error::from)
        }
    }

    pub async fn faucet(
        &self,
        signer: T::Pair,
        dest: subxt::ext::subxt_core::utils::MultiAddress<AccountId32, ()>,
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

    pub fn runtime_version(&self) -> RuntimeVersion {
        self.client.runtime_version()
    }

    pub fn metadata(&self) -> Metadata {
        self.client.metadata()
    }

    pub fn genesis_hash(&self) -> T::Hash {
        self.client.genesis_hash()
    }
}
