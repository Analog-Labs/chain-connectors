use ::core::{
    future::Future,
    marker::Send,
    ops::{Deref, DerefMut},
    pin::Pin,
};

use crate::{AccessListWithGasUsed, AtBlock, EthereumRpc, ExitReason, TransactionCall};
use alloc::boxed::Box;
pub use jsonrpsee_core as core;
use jsonrpsee_core::{
    client::{ClientT, SubscriptionClientT},
    rpc_params, Error,
};
use rosetta_ethereum_primitives::{
    Address, Block, BlockIdentifier, Bytes, EIP1186ProofResponse, TransactionReceipt, TxHash, H256,
    U256, U64,
};

/// Adapter for [`ClientT`] to [`EthereumRpc`].
#[repr(transparent)]
pub struct Adapter<T: ClientT + Send + Sync>(pub T);

impl<T> Adapter<T>
where
    T: ClientT + Send + Sync,
{
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Adapter<T>
where
    T: ClientT + Send + Sync,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> AsRef<T> for Adapter<T>
where
    T: ClientT + Send + Sync,
{
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Adapter<T>
where
    T: ClientT + Send + Sync,
{
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Deref for Adapter<T>
where
    T: ClientT + Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Adapter<T>
where
    T: ClientT + Send + Sync,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Clone for Adapter<T>
where
    T: ClientT + Send + Sync + Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}

impl<T> alloc::fmt::Debug for Adapter<T>
where
    T: ClientT + Send + Sync + alloc::fmt::Debug,
{
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        f.debug_tuple("Adapter").field(&self.0).finish()
    }
}

impl<T> alloc::fmt::Display for Adapter<T>
where
    T: ClientT + Send + Sync + alloc::fmt::Display,
{
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        <T as alloc::fmt::Display>::fmt(&self.0, f)
    }
}

#[async_trait::async_trait]
impl<T> EthereumRpc for Adapter<T>
where
    T: ClientT + Send + Sync,
{
    type Error = Error;

    /// Returns the balance of the account.
    async fn get_balance(&self, account: Address, at: AtBlock) -> Result<U256, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_getBalance", rpc_params![account, at]).await
    }

    /// Returns the number of transactions sent from an address.
    async fn get_transaction_count(
        &self,
        account: Address,
        at: AtBlock,
    ) -> Result<U64, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_getTransactionCount", rpc_params![account, at]).await
    }

    /// Returns code at a given account
    async fn get_code(&self, account: Address, at: AtBlock) -> Result<Bytes, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_getCode", rpc_params![account, at]).await
    }

    /// Executes a new message call immediately without creating a transaction on the blockchain.
    async fn call(&self, tx: &TransactionCall, at: AtBlock) -> Result<ExitReason, Self::Error> {
        <T as ClientT>::request::<Bytes, _>(&self.0, "eth_call", rpc_params![tx, at])
            .await
            .map(ExitReason::Succeed)
    }

    /// Returns an estimate of how much gas is necessary to allow the transaction to complete.
    async fn estimate_gas(&self, tx: &TransactionCall, at: AtBlock) -> Result<U256, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_estimateGas", rpc_params![tx, at]).await
    }

    /// Returns the current gas price in wei.
    async fn gas_price(&self) -> Result<U256, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_gasPrice", rpc_params![]).await
    }

    /// Submits a pre-signed transaction for broadcast to the Ethereum network.
    async fn send_raw_transaction(&self, tx: Bytes) -> Result<TxHash, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_sendRawTransaction", rpc_params![tx]).await
    }

    /// Returns the receipt of a transaction by transaction hash.
    async fn transaction_receipt(
        &self,
        tx: TxHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_getTransactionReceipt", rpc_params![tx]).await
    }

    /// Creates an EIP-2930 access list that you can include in a transaction.
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    async fn create_access_list(
        &self,
        tx: &TransactionCall,
        at: AtBlock,
    ) -> Result<AccessListWithGasUsed, Error> {
        <T as ClientT>::request(&self.0, "eth_createAccessList", rpc_params![tx, at]).await
    }

    /// Returns the account and storage values, including the Merkle proof, of the specified
    /// account.
    async fn get_proof(
        &self,
        address: Address,
        storage_keys: &[H256],
        at: AtBlock,
    ) -> Result<EIP1186ProofResponse, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_getProof", rpc_params![address, storage_keys, at])
            .await
    }

    /// Get storage value of address at index.
    async fn storage(
        &self,
        address: Address,
        index: H256,
        at: AtBlock,
    ) -> Result<H256, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_getStorageAt", rpc_params![address, index, at]).await
    }

    /// Returns information about a block.
    async fn block(&self, at: AtBlock) -> Result<Option<Block<H256>>, Self::Error> {
        let block = if let AtBlock::At(BlockIdentifier::Hash(block_hash)) = at {
            <T as ClientT>::request::<Block<TxHash>, _>(
                &self.0,
                "eth_getBlockByHash",
                rpc_params![block_hash, false],
            )
            .await?
        } else {
            <T as ClientT>::request::<Block<TxHash>, _>(
                &self.0,
                "eth_getBlockByNumber",
                rpc_params![at, false],
            )
            .await?
        };
        Ok(Some(block))
    }

    /// Returns the currently configured chain ID, a value used in replay-protected
    /// transaction signing as introduced by EIP-155.
    async fn chain_id(&self) -> Result<U64, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_chainId", rpc_params![]).await
    }
}

impl<T> ClientT for Adapter<T>
where
    T: ClientT + Send + Sync,
{
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn notification<'life0, 'life1, 'async_trait, Params>(
        &'life0 self,
        method: &'life1 str,
        params: Params,
    ) -> Pin<
        Box<
            dyn Future<Output = ::core::result::Result<(), ::jsonrpsee_core::Error>>
                + Send
                + 'async_trait,
        >,
    >
    where
        Params: ::jsonrpsee_core::traits::ToRpcParams + Send,
        Params: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        <T as ::jsonrpsee_core::client::ClientT>::notification(&self.0, method, params)
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn request<'life0, 'life1, 'async_trait, R, Params>(
        &'life0 self,
        method: &'life1 str,
        params: Params,
    ) -> Pin<
        Box<
            dyn Future<Output = ::core::result::Result<R, ::jsonrpsee_core::Error>>
                + Send
                + 'async_trait,
        >,
    >
    where
        R: ::serde::de::DeserializeOwned,
        Params: ::jsonrpsee_core::traits::ToRpcParams + Send,
        R: 'async_trait,
        Params: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        <T as ::jsonrpsee_core::client::ClientT>::request(&self.0, method, params)
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn batch_request<'a, 'life0, 'async_trait, R>(
        &'life0 self,
        batch: ::jsonrpsee_core::params::BatchRequestBuilder<'a>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = ::core::result::Result<
                        ::jsonrpsee_core::client::BatchResponse<'a, R>,
                        ::jsonrpsee_core::Error,
                    >,
                > + Send
                + 'async_trait,
        >,
    >
    where
        R: ::serde::de::DeserializeOwned + ::core::fmt::Debug + 'a,
        'a: 'async_trait,
        R: 'async_trait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        <T as ::jsonrpsee_core::client::ClientT>::batch_request(&self.0, batch)
    }
}

impl<T> SubscriptionClientT for Adapter<T>
where
    T: SubscriptionClientT + Send + Sync,
{
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn subscribe<'a, 'life0, 'async_trait, Notif, Params>(
        &'life0 self,
        subscribe_method: &'a str,
        params: Params,
        unsubscribe_method: &'a str,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = ::core::result::Result<
                        ::jsonrpsee_core::client::Subscription<Notif>,
                        ::jsonrpsee_core::Error,
                    >,
                > + Send
                + 'async_trait,
        >,
    >
    where
        Params: ::jsonrpsee_core::traits::ToRpcParams + Send,
        Notif: ::serde::de::DeserializeOwned,
        'a: 'async_trait,
        Notif: 'async_trait,
        Params: 'async_trait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        <T as ::jsonrpsee_core::client::SubscriptionClientT>::subscribe(
            &self.0,
            subscribe_method,
            params,
            unsubscribe_method,
        )
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn subscribe_to_method<'a, 'life0, 'async_trait, Notif>(
        &'life0 self,
        method: &'a str,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = ::core::result::Result<
                        ::jsonrpsee_core::client::Subscription<Notif>,
                        ::jsonrpsee_core::Error,
                    >,
                > + Send
                + 'async_trait,
        >,
    >
    where
        Notif: ::serde::de::DeserializeOwned,
        'a: 'async_trait,
        Notif: 'async_trait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        <T as ::jsonrpsee_core::client::SubscriptionClientT>::subscribe_to_method(&self.0, method)
    }
}
