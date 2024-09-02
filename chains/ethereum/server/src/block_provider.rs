use super::client::BlockFinalityStrategy;
use std::{
    fmt::{Debug, Display},
    sync::Arc,
};
use tokio::{
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::utils::PartialBlock;
use futures_util::{future::BoxFuture, Future, FutureExt};
use rosetta_config_ethereum::{
    ext::types::{crypto::DefaultCrypto, BlockIdentifier},
    AtBlock,
};
use rosetta_ethereum_backend::EthereumRpc;

/// Block Provider
pub trait BlockProvider: Unpin {
    /// Error type
    type Error: Unpin + Send + Sync + 'static;
    /// Future type
    type BlockAtFut: Future<Output = Result<Option<Arc<PartialBlock>>, Self::Error>>
        + Unpin
        + Send
        + 'static;
    /// Future type
    type LatestFut: Future<Output = Result<Arc<PartialBlock>, Self::Error>> + Unpin + Send + 'static;
    /// Future type
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
pub enum BlockProviderError<ERR> {
    Rpc(ERR),
    LatestBlockNotFound,
    FinalizedBlockNotFound,
}

impl<ERR> Unpin for BlockProviderError<ERR> where ERR: Unpin {}
unsafe impl<ERR> Send for BlockProviderError<ERR> where ERR: Send {}
unsafe impl<ERR> Sync for BlockProviderError<ERR> where ERR: Sync {}

impl<ERR> std::error::Error for BlockProviderError<ERR>
where
    ERR: std::error::Error + Display + Debug,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Rpc(err) => err.source(),
            Self::LatestBlockNotFound | Self::FinalizedBlockNotFound => None,
        }
    }
}

impl<ERR> Display for BlockProviderError<ERR>
where
    ERR: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rpc(err) => Display::fmt(err, f),
            Self::LatestBlockNotFound => write!(f, "latest block not found"),
            Self::FinalizedBlockNotFound => write!(f, "finalized block not found"),
        }
    }
}

impl<ERR> Debug for BlockProviderError<ERR>
where
    ERR: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rpc(err) => f.debug_tuple("Rpc").field(err).finish(),
            Self::LatestBlockNotFound => write!(f, "LatestBlockNotFound"),
            Self::FinalizedBlockNotFound => write!(f, "FinalizedBlockNotFound"),
        }
    }
}

impl<ERR> PartialEq for BlockProviderError<ERR>
where
    ERR: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Rpc(err0), Self::Rpc(err1)) => err0.eq(err1),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
impl<ERR> Eq for BlockProviderError<ERR> where ERR: Eq {}

impl<ERR> Clone for BlockProviderError<ERR>
where
    ERR: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Rpc(err) => Self::Rpc(err.clone()),
            Self::LatestBlockNotFound => Self::LatestBlockNotFound,
            Self::FinalizedBlockNotFound => Self::FinalizedBlockNotFound,
        }
    }
}

async fn retrieve_sealed_block<RPC>(
    rpc: RPC,
    at: AtBlock,
) -> Result<Option<PartialBlock>, RPC::Error>
where
    RPC: EthereumRpc + Unpin + Send + Sync + 'static,
    RPC::Error: std::error::Error + Unpin + Send + Sync + 'static,
{
    let Some(block) = EthereumRpc::block(&rpc, at).await? else {
        return Ok(None);
    };

    let block = if let Some(hash) = block.hash {
        block.seal(hash)
    } else {
        // OBS: this should never happen, except for pending blocks, a block should always have a
        // hash.
        tracing::warn!(
            "[report this bug] api returned a block without hash, computing the hash locally..."
        );
        block.seal_slow::<DefaultCrypto>()
    };
    Ok(Some(block))
}

/// Implement `BlockProvider` for `EthereumRpc`
impl<RPC> BlockProvider for RPC
where
    RPC: EthereumRpc + Send + Sync + Clone + Unpin + 'static,
    RPC::Error: std::error::Error + Unpin + Send + Sync + 'static,
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
        let rpc = self.clone();
        async move {
            let maybe_block = retrieve_sealed_block(rpc, block_ref.into())
                .await
                .map_err(BlockProviderError::Rpc)?;
            Ok(maybe_block.map(Arc::new))
        }
        .boxed()
    }

    /// Retrieve the latest block
    fn latest(&self) -> Self::LatestFut {
        let rpc = self.clone();
        async move {
            let Some(latest_block) = retrieve_sealed_block(rpc, AtBlock::Latest)
                .await
                .map_err(BlockProviderError::Rpc)?
            else {
                return Err(BlockProviderError::LatestBlockNotFound);
            };
            Ok(Arc::new(latest_block))
        }
        .boxed()
    }

    /// Retrieve the latest finalized block
    fn finalized(&self) -> Self::FinalizedFut {
        let rpc = self.clone();
        async move {
            let Some(best_block) = retrieve_sealed_block(rpc, AtBlock::Finalized)
                .await
                .map_err(BlockProviderError::Rpc)?
            else {
                return Err(BlockProviderError::FinalizedBlockNotFound);
            };
            Ok(Arc::new(best_block))
        }
        .boxed()
    }
}

#[derive(Clone)]
pub struct RpcBlockProvider<RPC> {
    /// Ethereum RPC client
    inner: Arc<InnerState<RPC>>,
}

impl<RPC> RpcBlockProvider<RPC>
where
    RPC: EthereumRpc + Send + Sync + Clone + Unpin + 'static,
    RPC::Error: std::error::Error + Unpin + Send + Sync + 'static,
{
    pub async fn new(
        rpc: RPC,
        cache_timeout: Duration,
        finality_strategy: BlockFinalityStrategy,
    ) -> Result<Self, BlockProviderError<RPC::Error>> {
        // Retrieve the latest block
        let Some(latest_block) = retrieve_sealed_block(rpc.clone(), AtBlock::Latest)
            .await
            .map_err(BlockProviderError::Rpc)?
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
            let Some(best_block) = retrieve_sealed_block(rpc.clone(), at_block)
                .await
                .map_err(BlockProviderError::Rpc)?
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
            best_block: Mutex::new((Arc::new(best_block), now)),
            latest_block: Mutex::new((Arc::new(latest_block), now)),
        };

        // Return the block provider
        Ok(Self { inner: Arc::new(inner) })
    }
}

impl<RPC> AsRef<RPC> for RpcBlockProvider<RPC> {
    fn as_ref(&self) -> &RPC {
        &self.inner.rpc
    }
}

impl<RPC> BlockProvider for RpcBlockProvider<RPC>
where
    RPC: EthereumRpc + Send + Sync + Clone + Unpin + 'static,
    RPC::Error: std::error::Error + Unpin + Send + Sync + 'static,
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
        <RPC as BlockProvider>::block_at(&self.inner.rpc, block_ref)
    }

    /// Retrieve the latest block
    fn latest(&self) -> Self::LatestFut {
        let this = self.inner.clone();
        async move { this.latest_block().await }.boxed()
    }

    /// Retrieve the latest finalized block, following the specified finality strategy
    fn finalized(&self) -> Self::FinalizedFut {
        let this = self.inner.clone();
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
    RPC: EthereumRpc + Send + Sync + Clone + Unpin + 'static,
    RPC::Error: std::error::Error + Unpin + Send + Sync + 'static,
{
    /// Get the cached latest block, or refresh after `cache_timeout`.
    async fn latest_block<'a: 'b, 'b>(
        &'a self,
    ) -> Result<Arc<PartialBlock>, BlockProviderError<<RPC as EthereumRpc>::Error>> {
        let mut guard = self.latest_block.lock().await;

        // Check if the cache has expired
        if guard.1.elapsed() > self.cache_timeout {
            let Some(latest_block) = retrieve_sealed_block(self.rpc.clone(), AtBlock::Latest)
                .await
                .map_err(BlockProviderError::Rpc)?
            else {
                return Err(BlockProviderError::LatestBlockNotFound);
            };
            *guard = (Arc::new(latest_block), Instant::now());
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
            let Some(best_block) = retrieve_sealed_block(self.rpc.clone(), at_block)
                .await
                .map_err(BlockProviderError::Rpc)?
            else {
                return Err(BlockProviderError::FinalizedBlockNotFound);
            };
            *guard = (Arc::new(best_block), Instant::now());
        }

        Ok(guard.0.clone())
    }
}
