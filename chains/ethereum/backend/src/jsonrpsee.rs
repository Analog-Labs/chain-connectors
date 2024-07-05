use crate::{
    rstd::{
        boxed::Box,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        marker::Send,
        ops::{Deref, DerefMut},
        string::ToString,
        vec::Vec,
    },
    BlockRange, MaybeDeserializeOwned,
};
use async_trait::async_trait;
use futures_core::future::BoxFuture;

use crate::{AccessListWithGasUsed, AtBlock, CallRequest, EthereumPubSub, EthereumRpc, ExitReason};
pub use jsonrpsee_core as core;
use jsonrpsee_core::{
    client::{ClientT, SubscriptionClientT},
    rpc_params, ClientError as Error,
};
use rosetta_ethereum_types::{
    rpc::{RpcBlock, RpcTransaction},
    Address, BlockIdentifier, Bytes, EIP1186ProofResponse, FeeHistory, Log, SealedHeader,
    TransactionReceipt, TxHash, H256, U256,
};

/// Adapter for [`ClientT`] to [`EthereumRpc`].
#[repr(transparent)]
pub struct Adapter<T>(pub T);

impl<T> Adapter<T>
where
    T: ClientT + Send + Sync,
{
    #[must_use]
    pub const fn inner(&self) -> &T {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.0
    }

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
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}

impl<T> Debug for Adapter<T>
where
    T: ClientT + Send + Sync + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Adapter").field(&self.0).finish()
    }
}

impl<T> Display for Adapter<T>
where
    T: ClientT + Send + Sync + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        <T as Display>::fmt(&self.0, f)
    }
}

#[async_trait]
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
    ) -> Result<u64, Self::Error> {
        let tx_count = <T as ClientT>::request::<U256, _>(
            &self.0,
            "eth_getTransactionCount",
            rpc_params![account, at],
        )
        .await?;
        u64::try_from(tx_count).map_err(|_| {
            Error::Custom(
                "invalid tx count, see https://eips.ethereum.org/EIPS/eip-2681".to_string(),
            )
        })
    }

    /// Returns code at a given account
    async fn get_code(&self, account: Address, at: AtBlock) -> Result<Bytes, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_getCode", rpc_params![account, at]).await
    }

    /// Returns an array of all the logs matching the given filter object
    async fn get_logs(&self, range: BlockRange) -> Result<Vec<Log>, Self::Error> {
        <T as ClientT>::request::<Vec<Log>, _>(&self.0, "eth_getLogs", rpc_params![range]).await
    }

    /// Executes a new message call immediately without creating a transaction on the blockchain.
    fn call<'life0, 'life1, 'async_trait>(
        &'life0 self,
        tx: &'life1 CallRequest,
        at: AtBlock,
    ) -> BoxFuture<'async_trait, Result<ExitReason, Self::Error>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let params = rpc_params![tx, at];
        Box::pin(async move {
            match <T as ClientT>::request::<Bytes, _>(&self.0, "eth_call", params).await {
                Ok(data) => Ok(ExitReason::Succeed(data)),
                Err(Error::Call(msg)) => {
                    if let Some(raw_value) = msg.data() {
                        if let Ok(revert_data) = serde_json::from_str::<Bytes>(raw_value.get()) {
                            // If the error is `Error(string)` or the message contains "revert", we
                            // assume it's a revert.
                            let revert = ExitReason::Revert(revert_data);
                            if revert.revert_msg().is_some() || msg.message().contains("revert") {
                                return Ok(revert);
                            }
                        }
                    } else if msg.message().contains("overflow") ||
                        msg.message().contains("underflow")
                    {
                        // we assume it's an stack overflow or underflow error.
                        return Ok(ExitReason::Error(msg.message().to_string().into()));
                    }
                    // otherwise we assume it's an RPC error (rate limit, invalid credentials, etc).
                    Err(Error::Call(msg))
                },
                Err(err) => Err(err),
            }
        })
    }

    /// Returns an estimate of how much gas is necessary to allow the transaction to complete.
    fn estimate_gas<'life0, 'life1, 'async_trait>(
        &'life0 self,
        tx: &'life1 CallRequest,
        at: AtBlock,
    ) -> BoxFuture<'async_trait, Result<U256, Self::Error>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let params = rpc_params![tx, at];
        <T as ClientT>::request(&self.0, "eth_estimateGas", params)
    }

    /// Returns the current gas price in wei.
    async fn gas_price(&self) -> Result<U256, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_gasPrice", rpc_params![]).await
    }

    /// Submits a pre-signed transaction for broadcast to the Ethereum network.
    async fn send_raw_transaction(&self, tx: Bytes) -> Result<TxHash, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_sendRawTransaction", rpc_params![tx]).await
    }

    /// Submits an unsigned transaction which will be signed by the node
    fn send_transaction<'life0, 'life1, 'async_trait>(
        &'life0 self,
        tx: &'life1 CallRequest,
    ) -> BoxFuture<'async_trait, Result<TxHash, Self::Error>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let params = rpc_params![tx];
        <T as ClientT>::request::<TxHash, _>(&self.0, "eth_sendTransaction", params)
    }

    /// Returns the receipt of a transaction by transaction hash.
    async fn transaction_receipt(
        &self,
        tx: TxHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_getTransactionReceipt", rpc_params![tx]).await
    }

    /// Returns information about a transaction for a given hash.
    async fn transaction_by_hash(&self, tx: TxHash) -> Result<Option<RpcTransaction>, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_getTransactionByHash", rpc_params![tx]).await
    }

    /// Creates an EIP-2930 access list that you can include in a transaction.
    /// [EIP-2930]: <https://eips.ethereum.org/EIPS/eip-2930>
    fn create_access_list<'life0, 'life1, 'async_trait>(
        &'life0 self,
        tx: &'life1 CallRequest,
        at: AtBlock,
    ) -> BoxFuture<'async_trait, Result<AccessListWithGasUsed, Self::Error>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let params = rpc_params![tx, at];
        <T as ClientT>::request(&self.0, "eth_createAccessList", params)
    }

    /// Returns the account and storage values, including the Merkle proof, of the specified
    /// account.
    fn get_proof<'life0, 'life1, 'async_trait>(
        &'life0 self,
        address: Address,
        storage_keys: &'life1 [H256],
        at: AtBlock,
    ) -> BoxFuture<'async_trait, Result<EIP1186ProofResponse, Self::Error>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let params = rpc_params![address, storage_keys, at];
        <T as ClientT>::request(&self.0, "eth_getProof", params)
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
    async fn block(&self, at: AtBlock) -> Result<Option<RpcBlock<H256, H256>>, Self::Error> {
        let maybe_block = if let AtBlock::At(BlockIdentifier::Hash(block_hash)) = at {
            <T as ClientT>::request::<Option<RpcBlock<H256, H256>>, _>(
                &self.0,
                "eth_getBlockByHash",
                rpc_params![block_hash, false],
            )
            .await?
        } else {
            <T as ClientT>::request::<Option<RpcBlock<H256, H256>>, _>(
                &self.0,
                "eth_getBlockByNumber",
                rpc_params![at, false],
            )
            .await?
        };
        Ok(maybe_block)
    }

    /// Returns information about a block.
    async fn block_full<TX>(&self, at: AtBlock) -> Result<Option<RpcBlock<TX, H256>>, Self::Error>
    where
        TX: MaybeDeserializeOwned + Send,
    {
        if let AtBlock::At(BlockIdentifier::Hash(block_hash)) = at {
            <T as ClientT>::request::<Option<RpcBlock<TX, H256>>, _>(
                &self.0,
                "eth_getBlockByHash",
                rpc_params![block_hash, true],
            )
            .await
        } else {
            <T as ClientT>::request::<Option<RpcBlock<TX, H256>>, _>(
                &self.0,
                "eth_getBlockByNumber",
                rpc_params![at, true],
            )
            .await
        }
    }

    /// Returns the current latest block number.
    async fn block_number(&self) -> Result<u64, Self::Error> {
        let res =
            <T as ClientT>::request::<U256, _>(&self.0, "eth_blockNumber", rpc_params![]).await?;
        u64::try_from(res)
            .map_err(|_| Error::Custom("invalid block number, it exceeds 2^64-1".to_string()))
    }

    /// Returns information about a uncle of a block given the block hash and the uncle index
    /// position.
    async fn uncle_by_blockhash(
        &self,
        block_hash: H256,
        index: u32,
    ) -> Result<Option<SealedHeader>, Self::Error> {
        let index = U256::from(index);
        <T as ClientT>::request::<Option<SealedHeader>, _>(
            &self.0,
            "eth_getUncleByBlockHashAndIndex",
            rpc_params![block_hash, index],
        )
        .await
    }

    /// Returns the currently configured chain ID, a value used in replay-protected
    /// transaction signing as introduced by EIP-155.
    async fn chain_id(&self) -> Result<u64, Self::Error> {
        let res = <T as ClientT>::request::<U256, _>(&self.0, "eth_chainId", rpc_params![]).await?;
        u64::try_from(res)
            .map_err(|_| Error::Custom("invalid chain_id, it exceeds 2^64-1".to_string()))
    }

    /// Returns a list of addresses owned by client.
    async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error> {
        <T as ClientT>::request(&self.0, "eth_accounts", rpc_params![]).await
    }

    /// Returns historical gas information, allowing you to track trends over time.
    async fn fee_history(
        &self,
        block_count: u64,
        last_block: AtBlock,
        reward_percentiles: &[f64],
    ) -> Result<FeeHistory, Self::Error> {
        let block_count = U256::from(block_count);
        let params = rpc_params![block_count, last_block, reward_percentiles];
        <T as ClientT>::request::<FeeHistory, _>(&self.0, "eth_feeHistory", params).await
    }
}

#[derive(serde::Serialize)]
struct LogsParams<'a> {
    address: Address,
    topics: &'a [H256],
}

#[async_trait]
impl<T> EthereumPubSub for Adapter<T>
where
    T: SubscriptionClientT + Send + Sync,
{
    type SubscriptionError = <Self as EthereumRpc>::Error;
    type NewHeadsStream<'a> = jsonrpsee_core::client::Subscription<RpcBlock<H256>> where Self: 'a;
    type LogsStream<'a> = jsonrpsee_core::client::Subscription<Log> where Self: 'a;

    /// Fires a notification each time a new header is appended to the chain, including chain
    /// reorganizations.
    /// Users can use the bloom filter to determine if the block contains logs that are interested
    /// to them. Note that if geth receives multiple blocks simultaneously, e.g. catching up after
    /// being out of sync, only the last block is emitted.
    fn new_heads<'a, 'async_trait>(
        &'a self,
    ) -> BoxFuture<'a, Result<Self::NewHeadsStream<'a>, Self::Error>>
    where
        'a: 'async_trait,
        Self: 'async_trait,
    {
        <T as SubscriptionClientT>::subscribe::<RpcBlock<H256>, _>(
            &self.0,
            "eth_subscribe",
            rpc_params!["newHeads"],
            "eth_unsubscribe",
        )
    }

    /// Returns logs that are included in new imported blocks and match the given filter criteria.
    /// In case of a chain reorganization previous sent logs that are on the old chain will be
    /// resent with the removed property set to true. Logs from transactions that ended up in
    /// the new chain are emitted. Therefore a subscription can emit logs for the same transaction
    /// multiple times.
    async fn logs<'a>(
        &'a self,
        contract: Address,
        topics: &[H256],
    ) -> Result<Self::LogsStream<'a>, Self::Error> {
        let params = LogsParams { address: contract, topics };
        <T as SubscriptionClientT>::subscribe(
            &self.0,
            "eth_subscribe",
            rpc_params!["logs", params],
            "eth_unsubscribe",
        )
        .await
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
    ) -> BoxFuture<'async_trait, Result<(), ::jsonrpsee_core::ClientError>>
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
    ) -> BoxFuture<'async_trait, Result<R, ::jsonrpsee_core::ClientError>>
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
    ) -> BoxFuture<
        'async_trait,
        Result<::jsonrpsee_core::client::BatchResponse<'a, R>, ::jsonrpsee_core::ClientError>,
    >
    where
        R: ::serde::de::DeserializeOwned + Debug + 'a,
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
    ) -> BoxFuture<
        'async_trait,
        Result<::jsonrpsee_core::client::Subscription<Notif>, ::jsonrpsee_core::ClientError>,
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
    ) -> BoxFuture<
        'async_trait,
        Result<::jsonrpsee_core::client::Subscription<Notif>, ::jsonrpsee_core::ClientError>,
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
