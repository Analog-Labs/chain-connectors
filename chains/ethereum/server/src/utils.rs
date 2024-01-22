use ethers::{prelude::*, providers::Middleware, types::H256};
use rosetta_core::types::BlockIdentifier;
use std::sync::Arc;

/// A block that is not pending, so it must have a valid hash and number.
/// This allow skipping duplicated checks in the code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonPendingBlock {
    pub hash: H256,
    pub number: u64,
    pub identifier: BlockIdentifier,
    pub block: ethers::types::Block<H256>,
}

impl TryFrom<ethers::types::Block<H256>> for NonPendingBlock {
    type Error = anyhow::Error;

    fn try_from(block: ethers::types::Block<H256>) -> std::result::Result<Self, Self::Error> {
        let Some(number) = block.number else { anyhow::bail!("block number is missing") };
        let Some(hash) = block.hash else { anyhow::bail!("block hash is missing") };
        Ok(Self {
            hash,
            number: number.as_u64(),
            identifier: BlockIdentifier::new(number.as_u64(), hash.0),
            block,
        })
    }
}

// Retrieve a non-pending block
pub async fn get_non_pending_block<C, ID>(
    client: Arc<Provider<C>>,
    block_id: ID,
) -> anyhow::Result<Option<NonPendingBlock>>
where
    C: JsonRpcClient + 'static,
    ID: Into<BlockId> + Send + Sync,
{
    let block_id = block_id.into();
    if matches!(block_id, BlockId::Number(BlockNumber::Pending)) {
        anyhow::bail!("request a pending block is not allowed");
    }
    let Some(block) = client.get_block(block_id).await? else {
        return Ok(None);
    };
    // The block is not pending, it MUST have a valid hash and number
    let Ok(block) = NonPendingBlock::try_from(block) else {
        anyhow::bail!(
            "[RPC CLIENT BUG] the rpc client returned an invalid non-pending block at {block_id:?}"
        );
    };
    Ok(Some(block))
}
