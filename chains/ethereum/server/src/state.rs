use rosetta_config_ethereum::{BlockFull, ext::types::H256};
use hashbrown::HashMap;
use rosetta_core::traits::{Block, Header};
use fork_tree::FinalizationResult;

type ForkTree = fork_tree::ForkTree<H256, u64, H256>;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("block not found: {0}")]
    BlockNotFound(H256),
}

/// Manages the client state
pub struct State {
    /// Map of block hashes to their full block data
    blocks: HashMap<H256, BlockFull>,
    /// Tree-like ordered blocks, used to track and remove orphan blocks
    fork_tree: ForkTree,
    /// Latest known finalized block
    best_finalized_block: Option<H256>,
}

impl State {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            fork_tree: ForkTree::new(),
            best_finalized_block: None,
        }
    }

    pub fn import(&mut self, block: BlockFull) -> Result<(), fork_tree::Error<Error>> {
        let hash = block.hash().0;
        let block_number = block.header().number();
        let parent_hash = block.header().0.header().parent_hash;
        self.blocks.insert(hash, block);

        let blocks = &self.blocks;
        self.fork_tree.import(hash, block_number, parent_hash, &|base, block| {
            is_descendent_of(blocks, *base, *block)
        })?;
        self.fork_tree.rebalance();
        Ok(())
    }

    pub fn finalize(&mut self, block_hash: H256) -> Result<Vec<BlockFull>, fork_tree::Error<Error>> {
        let Some(block) = self.blocks.get(&block_hash).map(BlockFull::header) else {
            return Err(fork_tree::Error::Client(Error::BlockNotFound(block_hash)));
        };
        let block_number = block.number();
        let result = self.fork_tree.finalize(&block_hash, block_number, &|base, block| {
            is_descendent_of(&self.blocks, *base, *block)
        })?;

        match result {
            FinalizationResult::Changed(_) => {},
            FinalizationResult::Unchanged => return Ok(Vec::new()),
        }

        // Remove orphan blocks from cache
        let removed = self.blocks.extract_if(|current, block| {
            // Skip finalized blocks
            if current == &block_hash || block.header().header().number < block_number {
                return false;
            }
            // Check if the block exists in the fork tree
            !self.fork_tree.iter().any(|(hash, _, _)| hash == current)
        }).map(|(_, block)| block).collect::<Vec<_>>();
        Ok(removed)
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

fn is_descendent_of(blocks: &HashMap<H256, BlockFull>, base: H256, block: H256) -> Result<bool, Error> {
    let Some(block) = blocks.get(&block).map(BlockFull::header) else {
        return Err(Error::BlockNotFound(block));
    };
    let Some(base) = blocks.get(&base).map(BlockFull::header) else {
        return Err(Error::BlockNotFound(base));
    };
    #[allow(clippy::cast_possible_wrap)]
    let mut diff = (block.number() as i64) - (base.number() as i64);
    
    if diff <= 0 || usize::try_from(diff).map(|diff| diff > blocks.len()).unwrap_or(true) {
        // base and block have the same number, so they can't be descendents
        return Ok(false);
    }

    // Walk up the chain until we find the block imediatly after base hash
    let mut parent_hash = block.0.header().parent_hash;
    while diff > 1 {
        let Some(parent) = blocks.get(&parent_hash).map(BlockFull::header) else {
            return Err(Error::BlockNotFound(parent_hash));
        };
        parent_hash = parent.0.header().parent_hash;
        diff -= 1;
    }
    Ok(parent_hash == base.hash().0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rosetta_config_ethereum::{BlockFull, ext::types::{H256, BlockBody, TypedTransaction, SignedTransaction, SealedHeader, SealedBlock, Header, crypto::DefaultCrypto}};

    fn create_block(parent_hash: H256, number: u64, nonce: u64) -> BlockFull {
        let body = BlockBody::<SignedTransaction<TypedTransaction>, SealedHeader> {
            transactions: Vec::new(),
            total_difficulty: None,
            seal_fields: Vec::new(),
            uncles: Vec::new(),
            size: None,
        };
        let header = Header {
            parent_hash,
            number,
            nonce,
            ..Header::default()
        };
        let header = header.seal_slow::<DefaultCrypto>();
        BlockFull(SealedBlock::new(header, body))
    }

    #[test]
    fn basic_test() {
		//     +---B-c-C---D---E
		//     |
		//     |   +---G
		//     |   | 
		// 0---A---F---H---I
		//     |       |
		//     |       +---L-m-M---N
		//     |           |
		//     |           +---O
		//     +---J---K
		//
		// (where N is not a part of fork tree)
        let mut state = State::new();
        let block_a = create_block(H256::zero(), 1, 1);
        let block_b = create_block(block_a.hash().0, 2, 2);
        let block_c = create_block(block_b.hash().0, 3, 3);
        let block_d = create_block(block_c.hash().0, 4, 4);
        let block_e = create_block(block_d.hash().0, 5, 5);
        let block_f = create_block(block_a.hash().0, 2, 6);
        let block_g = create_block(block_f.hash().0, 3, 7);
        let block_h = create_block(block_f.hash().0, 3, 8);
        let block_i = create_block(block_h.hash().0, 4, 9);
        let block_j = create_block(block_a.hash().0, 2, 10);
        let block_k = create_block(block_j.hash().0, 3, 11);
        let block_l = create_block(block_h.hash().0, 4, 12);
        let block_m = create_block(block_l.hash().0, 5, 13);
        let block_o = create_block(block_l.hash().0, 5, 15);

        let blocks = [
            block_a.clone(),
            block_b.clone(),
            block_c.clone(),
            block_d.clone(),
            block_e.clone(),
            block_f.clone(),
            block_g,
            block_h,
            block_i,
            block_l,
            block_m,
            block_o,
            block_j.clone(),
            block_k.clone(),
        ];

        // Import all blocks
        for block in blocks {
            state.import(block).unwrap();
        }

        // Finalize block A
        let retracted = state.finalize(block_a.hash().0).unwrap();
        assert!(retracted.is_empty());

        // Finalize block F
        let retracted = state.finalize(block_f.hash().0).unwrap();
        let expect_retracted = vec![block_b, block_c, block_d, block_e, block_j, block_k];
        assert!(expect_retracted.iter().all(|hash| retracted.contains(hash)));
        assert_eq!(retracted.len(), expect_retracted.len());
    }
}