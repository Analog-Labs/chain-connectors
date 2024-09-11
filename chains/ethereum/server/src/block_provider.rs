use super::client::BlockFinalityStrategy;
use std::sync::Arc;
use tokio::{
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::utils::PartialBlock;
use auto_impl::auto_impl;
use futures_util::{future::BoxFuture, Future, FutureExt};
use rosetta_config_ethereum::{
    ext::types::{crypto::DefaultCrypto, rpc::RpcBlock, BlockIdentifier, SealedBlock},
    AtBlock, H256,
};
use rosetta_ethereum_backend::{jsonrpsee::Adapter, EthereumRpc};

/// Block Provider trait provides an interface to query blocks from an Ethereum node using
/// different finality strategies.
#[auto_impl(&, Box, Arc)]
pub trait BlockProvider {
    /// Error type
    type Error: Send;
    /// Future type when querying a block by hash or number
    type BlockAtFut: Future<Output = Result<Option<Arc<PartialBlock>>, Self::Error>>
        + Unpin
        + Send
        + 'static;
    /// Future type when querying the latest block
    type LatestFut: Future<Output = Result<Arc<PartialBlock>, Self::Error>> + Unpin + Send + 'static;
    /// Future type when querying the latest finalized block
    type FinalizedFut: Future<Output = Result<Arc<PartialBlock>, Self::Error>>
        + Unpin
        + Send
        + 'static;

    /// Get block by identifier
    fn block_at(&self, block_ref: BlockIdentifier) -> Self::BlockAtFut;
    /// Retrieve the latest block
    fn latest(&self) -> Self::LatestFut;
    /// Retrieve the latest finalized block
    fn finalized(&self) -> Self::FinalizedFut;
}

/// Block Provider Error
#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum BlockProviderError<ERR> {
    #[error("{0}")]
    Rpc(ERR),
    #[error("latest block not found")]
    LatestBlockNotFound,
    #[error("finalized block not found")]
    FinalizedBlockNotFound,
}

/// Converts a `RpcBlock` into a `Arc<SealedBlock>`.
fn into_sealed_block<ERR>(
    block: Result<Option<RpcBlock<H256>>, ERR>,
) -> Result<Option<Arc<SealedBlock<H256>>>, BlockProviderError<ERR>> {
    let Some(block) = block.map_err(BlockProviderError::Rpc)? else {
        return Ok(None);
    };

    let block_number = block.header.number;
    let block = if let Some(hash) = block.hash {
        block.seal(hash)
    } else {
        // OBS: this should never happen, except for pending blocks, a block should
        // always have a hash.
        let sealed_block = block.seal_slow::<DefaultCrypto>();
        let block_hash = sealed_block.header().hash();
        tracing::error!(
            "[report this bug] api returned the block {block_number} without hash, hash was computed locally: {block_hash}."
        );
        sealed_block
    };
    Ok(Some(Arc::new(block)))
}

impl<RPC> BlockProvider for Adapter<RPC>
where
    RPC: EthereumRpc + Send + Clone + 'static,
    RPC::Error: Send,
{
    type Error = BlockProviderError<<RPC as EthereumRpc>::Error>;
    type BlockAtFut = BoxFuture<'static, Result<Option<Arc<PartialBlock>>, Self::Error>>;
    type LatestFut = BoxFuture<'static, Result<Arc<PartialBlock>, Self::Error>>;
    type FinalizedFut = BoxFuture<'static, Result<Arc<PartialBlock>, Self::Error>>;

    fn block_at(&self, block_ref: BlockIdentifier) -> Self::BlockAtFut {
        let rpc = self.clone();
        async move {
            let maybe_block = <RPC as EthereumRpc>::block(&rpc.0, block_ref.into())
                .map(into_sealed_block)
                .await?;
            Ok(maybe_block)
        }
        .boxed()
    }

    fn latest(&self) -> Self::LatestFut {
        let rpc = self.clone();
        async move {
            let Some(latest_block) = <RPC as EthereumRpc>::block(&rpc.0, AtBlock::Latest)
                .map(into_sealed_block)
                .await?
            else {
                return Err(BlockProviderError::LatestBlockNotFound);
            };
            Ok(latest_block)
        }
        .boxed()
    }

    fn finalized(&self) -> Self::FinalizedFut {
        let rpc = self.clone();
        async move {
            let Some(best_block) = <RPC as EthereumRpc>::block(&rpc.0, AtBlock::Finalized)
                .map(into_sealed_block)
                .await?
            else {
                return Err(BlockProviderError::FinalizedBlockNotFound);
            };
            Ok(best_block)
        }
        .boxed()
    }
}

#[derive(Clone)]
pub struct RpcBlockProvider<RPC>(Arc<InnerState<RPC>>);

impl<RPC> RpcBlockProvider<RPC>
where
    RPC: EthereumRpc + Send + Sync + Clone + 'static,
    RPC::Error: std::error::Error + Send,
{
    pub async fn new(
        rpc: RPC,
        cache_timeout: Duration,
        finality_strategy: BlockFinalityStrategy,
    ) -> Result<Self, BlockProviderError<RPC::Error>> {
        // Retrieve the latest block
        let Some(latest_block) = <RPC as EthereumRpc>::block(&rpc, AtBlock::Latest)
            .map(into_sealed_block)
            .await?
        else {
            return Err(BlockProviderError::LatestBlockNotFound);
        };

        // Retrieve the best block following the finality strategy
        let best_block = {
            let at_block = match finality_strategy {
                BlockFinalityStrategy::Finalized => AtBlock::Finalized,
                BlockFinalityStrategy::Confirmations(confirmations) => {
                    let latest_block_number = latest_block.header().number();
                    let best_block_number = latest_block_number.saturating_sub(confirmations);
                    AtBlock::At(BlockIdentifier::Number(best_block_number))
                },
            };
            let Some(best_block) =
                <RPC as EthereumRpc>::block(&rpc, at_block).map(into_sealed_block).await?
            else {
                return Err(BlockProviderError::FinalizedBlockNotFound);
            };
            best_block
        };
        let now = Instant::now();

        // Create the inner state
        let inner = InnerState {
            rpc,
            finality_strategy,
            cache_timeout,
            best_block: Mutex::new((best_block, now)),
            latest_block: Mutex::new((latest_block, now)),
        };

        // Return the block provider
        Ok(Self(Arc::new(inner)))
    }
}

impl<RPC> AsRef<RPC> for RpcBlockProvider<RPC> {
    fn as_ref(&self) -> &RPC {
        &self.0.rpc
    }
}

impl<RPC> BlockProvider for RpcBlockProvider<RPC>
where
    RPC: EthereumRpc + Send + Sync + 'static,
    RPC::Error: std::error::Error + Send,
{
    /// Error type
    type Error = BlockProviderError<RPC::Error>;
    /// Future type
    type BlockAtFut = BoxFuture<'static, Result<Option<Arc<PartialBlock>>, Self::Error>>;
    /// Future type
    type LatestFut = BoxFuture<'static, Result<Arc<PartialBlock>, Self::Error>>;
    /// Future type
    type FinalizedFut = BoxFuture<'static, Result<Arc<PartialBlock>, Self::Error>>;

    /// Get block by identifier
    fn block_at(&self, block_ref: BlockIdentifier) -> Self::BlockAtFut {
        let this = self.0.clone();
        async move {
            let maybe_block = <RPC as EthereumRpc>::block(&this.rpc, block_ref.into())
                .map(into_sealed_block)
                .await?;
            Ok(maybe_block)
        }
        .boxed()
    }

    /// Retrieve the latest block
    fn latest(&self) -> Self::LatestFut {
        let this = self.0.clone();
        async move { this.latest_block().await }.boxed()
    }

    /// Retrieve the latest finalized block, following the specified finality strategy
    fn finalized(&self) -> Self::FinalizedFut {
        let this = self.0.clone();
        async move { this.best_block().await }.boxed()
    }
}

struct InnerState<RPC> {
    /// Ethereum RPC client
    rpc: RPC,
    /// How to determine block finality
    finality_strategy: BlockFinalityStrategy,
    /// Duration to discard cached `best_block` and `latest_block`
    cache_timeout: Duration,
    /// Best finalized block number that we have seen
    best_block: Mutex<(Arc<PartialBlock>, Instant)>,
    /// Latest block number that we have seen
    latest_block: Mutex<(Arc<PartialBlock>, Instant)>,
}

impl<RPC> InnerState<RPC>
where
    RPC: EthereumRpc + Send + Sync + 'static,
    RPC::Error: std::error::Error + Send,
{
    /// Get the cached latest block, or refresh after `cache_timeout`.
    async fn latest_block<'a: 'b, 'b>(
        &'a self,
    ) -> Result<Arc<PartialBlock>, BlockProviderError<<RPC as EthereumRpc>::Error>> {
        let mut guard = self.latest_block.lock().await;

        // Check if the cache has expired
        if guard.1.elapsed() > self.cache_timeout {
            let Some(latest_block) = <RPC as EthereumRpc>::block(&self.rpc, AtBlock::Latest)
                .map(into_sealed_block)
                .await?
            else {
                return Err(BlockProviderError::LatestBlockNotFound);
            };
            *guard = (latest_block, Instant::now());
        }

        Ok(guard.0.clone())
    }

    /// Get the cached finalized block, or refresh after `cache_timeout`.
    async fn best_block<'a: 'b, 'b>(
        &'a self,
    ) -> Result<Arc<PartialBlock>, BlockProviderError<<RPC as EthereumRpc>::Error>> {
        let mut guard = self.best_block.lock().await;

        // Check if the cache has expired
        if guard.1.elapsed() > self.cache_timeout {
            let at_block = match self.finality_strategy {
                BlockFinalityStrategy::Finalized => AtBlock::Finalized,
                BlockFinalityStrategy::Confirmations(confirmations) => {
                    let latest_block_number = self.latest_block().await?.header().number();
                    let best_block_number = latest_block_number.saturating_sub(confirmations);
                    // If the best block number is the same, simply refresh the cache timestamp.
                    if best_block_number == guard.0.header().number() {
                        *guard = (guard.0.clone(), Instant::now());
                        return Ok(guard.0.clone());
                    }
                    AtBlock::At(BlockIdentifier::Number(best_block_number))
                },
            };
            let Some(best_block) =
                <RPC as EthereumRpc>::block(&self.rpc, at_block).map(into_sealed_block).await?
            else {
                return Err(BlockProviderError::FinalizedBlockNotFound);
            };
            *guard = (best_block, Instant::now());
        }

        Ok(guard.0.clone())
    }
}
