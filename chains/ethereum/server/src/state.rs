use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, RwLock},
};

use crate::multi_block::{BlockRef, MultiBlock};
use fork_tree::FinalizationResult;
use hashbrown::{hash_map::Entry, HashMap};
use rosetta_config_ethereum::ext::types::H256;

type ForkTree = fork_tree::ForkTree<BlockRef, u64, H256>;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("block not found: {0}")]
    BlockNotFound(H256),
}

#[derive(Debug, Clone)]
pub struct State {
    inner: Arc<RwLock<StateInner>>,
}

impl State {
    pub fn new<B: Into<MultiBlock>>(best_finalized_block: B) -> Self {
        Self { inner: Arc::new(RwLock::new(StateInner::new(best_finalized_block))) }
    }

    pub fn import<B: Into<MultiBlock>>(&self, block: B) -> Result<(), fork_tree::Error<Error>> {
        #[allow(clippy::unwrap_used)]
        self.inner.write().unwrap().import(block)
    }

    pub fn finalize<B: Into<BlockRef>>(
        &self,
        finalized_block_ref: B,
    ) -> Result<Vec<MultiBlock>, fork_tree::Error<Error>> {
        let finalized_block_ref = finalized_block_ref.into();
        #[allow(clippy::unwrap_used)]
        self.inner.write().unwrap().finalize(finalized_block_ref)
    }
}

/// Manages the client state
#[derive(Debug, PartialEq)]
struct StateInner {
    /// Map of block hashes to their full block data
    blocks: HashMap<BlockRef, MultiBlock>,
    /// Maps an orphan block to missing block
    orphans: HashMap<BlockRef, BlockRef>,
    /// Maps a missing block to a list of orphan blocks
    missing: HashMap<BlockRef, BTreeMap<BlockRef, MultiBlock>>,
    /// Tree-like ordered blocks, used to track and remove orphan blocks
    fork_tree: ForkTree,
    /// List of finalized finalized blocks
    finalized_blocks: VecDeque<BlockRef>,
    /// latest known block
    latest_block: BlockRef,
}

impl StateInner {
    fn new<B: Into<MultiBlock>>(best_finalized_block: B) -> Self {
        let best_finalized_block = best_finalized_block.into();
        let best_finalized_block_ref = best_finalized_block.as_block_ref();
        let best_finalized_block_parent = best_finalized_block.parent_hash();

        // Initialize the state with the best finalized block
        let mut blocks = HashMap::with_capacity(1024);
        blocks.insert(best_finalized_block_ref, best_finalized_block);
        let mut finalized_blocks = VecDeque::with_capacity(512);
        finalized_blocks.push_back(best_finalized_block_ref);
        let mut fork_tree = ForkTree::new();

        #[allow(clippy::expect_used)]
        {
            fork_tree
                .import(
                    best_finalized_block_ref,
                    best_finalized_block_ref.number,
                    best_finalized_block_parent,
                    &|_base, _block| Result::<bool, Error>::Ok(true),
                )
                .expect("qed: best_finalized_block is valid");
            fork_tree
                .finalize(
                    &best_finalized_block_ref,
                    best_finalized_block_ref.number,
                    &|_base, _block| Result::<bool, Error>::Ok(true),
                )
                .expect("qed: best_finalized_block is valid");
        }

        Self {
            blocks,
            orphans: HashMap::new(),
            missing: HashMap::new(),
            fork_tree,
            finalized_blocks,
            latest_block: best_finalized_block_ref,
        }
    }

    fn insert_block(&mut self, block: MultiBlock) -> Result<(), fork_tree::Error<Error>> {
        let block_ref = block.as_block_ref();
        let parent_hash = block.parent_hash();
        self.blocks.insert(block_ref, block);
        let blocks = &self.blocks;
        self.fork_tree
            .import(block_ref, block_ref.number, parent_hash, &|base, block| {
                is_descendent_of(blocks, *base, *block)
            })?;
        self.fork_tree.rebalance();
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn import<B: Into<MultiBlock>>(&mut self, block: B) -> Result<(), fork_tree::Error<Error>> {
        let block = block.into();

        // Check if the block is already in the cache, if so, update it
        if let Some(cached) = self.blocks.get_mut(&block.as_block_ref()) {
            cached.upgrade(block);
            return Ok(());
        }

        // Block number must be greater than the latest finalized block
        if let Some(best_block) =
            self.finalized_blocks.back().and_then(|hash| self.blocks.get(hash))
        {
            if block.number() <= best_block.number() {
                // Block is younger than the latest finalized block, so it can't be imported
                return Ok(());
            }
            if block.number() == (best_block.number() + 1) &&
                block.parent_hash() != best_block.hash()
            {
                // the block is not descendent of the best finalized block
                return Ok(());
            }
        }

        // Check if the block is in the missing list
        if let Some(mut children) = self.missing.remove(&block.as_block_ref()) {
            // Check if the new block is orphan
            if !self.blocks.contains_key(&block.parent_ref()) {
                // Add block to the orphan list
                let missing_ref =
                    if let Some(parent_ref) = self.orphans.get(&block.parent_ref()).copied() {
                        self.orphans.insert(block.as_block_ref(), parent_ref);
                        parent_ref
                    } else {
                        let parent_ref = block.parent_ref();
                        self.orphans.insert(block.as_block_ref(), parent_ref);
                        parent_ref
                    };

                // Update children missing references
                for child_ref in children.keys().copied() {
                    self.orphans.insert(child_ref, missing_ref);
                }

                // Add block to the orphan list
                match self.missing.entry(missing_ref) {
                    Entry::Occupied(mut entry) => {
                        let orphans = entry.get_mut();
                        if let Some(cached) = orphans.get_mut(&block.as_block_ref()) {
                            cached.upgrade(block);
                        } else {
                            orphans.insert(block.as_block_ref(), block);
                        }
                        orphans.extend(children);
                    },
                    Entry::Vacant(entry) => {
                        children.insert(block.as_block_ref(), block);
                        entry.insert(children);
                    },
                }
                return Ok(());
            }
            // Remove children from the orphan list
            for child_ref in children.keys() {
                self.orphans.remove(child_ref);
            }

            // Import blocks in order
            self.insert_block(block)?;
            for child in children.into_values() {
                self.insert_block(child)?;
            }
            return Ok(());
        }

        // Check if the block is orphan
        if !self.blocks.contains_key(&block.parent_ref()) {
            // Add block to the orphan list
            let missing_ref =
                if let Some(parent_ref) = self.orphans.get(&block.parent_ref()).copied() {
                    self.orphans.insert(block.as_block_ref(), parent_ref);
                    parent_ref
                } else {
                    let parent_ref = block.parent_ref();
                    self.orphans.insert(block.as_block_ref(), parent_ref);
                    parent_ref
                };

            match self.missing.entry(missing_ref) {
                Entry::Occupied(mut entry) => {
                    let orphans = entry.get_mut();
                    if let Some(cached) = orphans.get_mut(&block.as_block_ref()) {
                        cached.upgrade(block);
                    } else {
                        orphans.insert(block.as_block_ref(), block);
                    }
                },
                Entry::Vacant(entry) => {
                    let mut orphans = BTreeMap::new();
                    orphans.insert(block.as_block_ref(), block);
                    entry.insert(orphans);
                },
            }
            return Ok(());
        }
        self.insert_block(block)?;
        Ok(())
    }

    fn finalize(
        &mut self,
        finalized_block_ref: BlockRef,
    ) -> Result<Vec<MultiBlock>, fork_tree::Error<Error>> {
        // Check if the block was imported
        if !self.blocks.contains_key(&finalized_block_ref) {
            return Err(fork_tree::Error::Client(Error::BlockNotFound(finalized_block_ref.hash)));
        };

        // Check if the block is already finalized
        if self.finalized_blocks.contains(&finalized_block_ref) {
            return Ok(Vec::new());
        }

        // Check if the block is descendent of the latest finalized block
        if let Some(best_finalized_block) = self.finalized_blocks.back().copied() {
            debug_assert!(
                finalized_block_ref.number > best_finalized_block.number,
                "[report this bug] all blocks before {} should be descendent of the latest finalized block",
                best_finalized_block.number
            );
            if finalized_block_ref.number <= best_finalized_block.number {
                return Err(fork_tree::Error::Client(Error::BlockNotFound(
                    finalized_block_ref.hash,
                )));
            }
        }

        let result = self.fork_tree.finalize(
            &finalized_block_ref,
            finalized_block_ref.number,
            &|base, block| is_descendent_of(&self.blocks, *base, *block),
        )?;

        match result {
            FinalizationResult::Changed(_) => {},
            FinalizationResult::Unchanged => return Ok(Vec::new()),
        }

        // Add finalized block to the list
        self.finalized_blocks.push_back(finalized_block_ref);

        // Remove retracted blocks
        let finalized_blocks = &self.finalized_blocks;
        let mut removed = self
            .blocks
            .extract_if(|_, block| {
                // Skip finalized blocks
                if finalized_blocks.contains(&block.as_block_ref()) {
                    return false;
                }
                // Check if the block exists in the fork tree
                !self.fork_tree.iter().any(|(block_ref, _, _)| block_ref.hash == block.hash())
            })
            .map(|(_, block)| block)
            .collect::<Vec<_>>();

        // Remove orphan blocks
        let missing = self
            .missing
            .extract_if(|missing_ref, _| missing_ref.number <= finalized_block_ref.number)
            .flat_map(|(_, block)| block.into_values());
        for orphan in missing {
            self.orphans.remove(&orphan.as_block_ref());
            removed.push(orphan);
        }
        Ok(removed)
    }
}

fn is_descendent_of(
    blocks: &HashMap<BlockRef, MultiBlock>,
    base: BlockRef,
    block: BlockRef,
) -> Result<bool, Error> {
    let Some(block) = blocks.get(&block) else {
        return Err(Error::BlockNotFound(block.hash));
    };
    let Some(base) = blocks.get(&base) else {
        return Err(Error::BlockNotFound(base.hash));
    };
    let Ok(mut diff) = i64::try_from(i128::from(block.number()) - i128::from(base.number())) else {
        // block gap is greater than the number of blocks cached.
        return Ok(false);
    };

    if diff <= 0 || usize::try_from(diff).map(|diff| diff > blocks.len()).unwrap_or(true) {
        // base and block have the same number, so they can't be descendents
        return Ok(false);
    }

    // Walk up the chain until we find the block imediatly after base hash
    let mut parent_ref = block.parent_ref();
    while diff > 1 {
        let Some(parent) = blocks.get(&parent_ref) else {
            return Err(Error::BlockNotFound(parent_ref.hash));
        };
        parent_ref = parent.parent_ref();
        diff -= 1;
    }
    Ok(parent_ref.hash == base.hash())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rosetta_config_ethereum::ext::types::{
        crypto::DefaultCrypto, BlockBody, Header, SealedBlock, SealedHeader, SignedTransaction,
        TypedTransaction, H256,
    };

    fn create_block(parent_hash: H256, number: u64, nonce: u64) -> MultiBlock {
        let body = BlockBody::<SignedTransaction<TypedTransaction>, SealedHeader> {
            transactions: Vec::new(),
            total_difficulty: None,
            seal_fields: Vec::new(),
            uncles: Vec::new(),
            size: None,
        };
        let header = Header { parent_hash, number, nonce, ..Header::default() };
        let header = header.seal_slow::<DefaultCrypto>();
        SealedBlock::new(header, body).into()
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
        let block_a = create_block(H256::zero(), 1, 1);
        let state = State::new(block_a.clone());
        let block_b = create_block(block_a.hash(), 2, 2);
        let block_c = create_block(block_b.hash(), 3, 3);
        let block_d = create_block(block_c.hash(), 4, 4);
        let block_e = create_block(block_d.hash(), 5, 5);
        let block_f = create_block(block_a.hash(), 2, 6);
        let block_g = create_block(block_f.hash(), 3, 7);
        let block_h = create_block(block_f.hash(), 3, 8);
        let block_i = create_block(block_h.hash(), 4, 9);
        let block_j = create_block(block_a.hash(), 2, 10);
        let block_k = create_block(block_j.hash(), 3, 11);
        let block_l = create_block(block_h.hash(), 4, 12);
        let block_m = create_block(block_l.hash(), 5, 13);
        let block_o = create_block(block_l.hash(), 5, 15);

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
        let retracted = state.finalize(block_a.as_block_ref()).unwrap();
        assert!(retracted.is_empty());

        // Finalize block F
        let retracted = state.finalize(block_f.as_block_ref()).unwrap();
        let expect_retracted = vec![block_b, block_c, block_d, block_e, block_j, block_k];
        assert!(expect_retracted.iter().all(|hash| retracted.contains(hash)));
        assert_eq!(retracted.len(), expect_retracted.len());
    }

    #[test]
    fn orphan_blocks_test() {
        //     +---X---C---D---E---P
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
        let block_a = create_block(H256::zero(), 1, 1);
        let state = State::new(block_a.clone());
        let block_b = create_block(block_a.hash(), 2, 2);
        let block_c = create_block(block_b.hash(), 3, 3);
        let block_d = create_block(block_c.hash(), 4, 4);
        let block_e = create_block(block_d.hash(), 5, 5);
        let block_p = create_block(block_e.hash(), 6, 16);

        let block_f = create_block(block_a.hash(), 2, 6);
        let block_g = create_block(block_f.hash(), 3, 7);
        let block_h = create_block(block_f.hash(), 3, 8);
        let block_i = create_block(block_h.hash(), 4, 9);
        let block_j = create_block(block_a.hash(), 2, 10);
        let block_k = create_block(block_j.hash(), 3, 11);
        let block_l = create_block(block_h.hash(), 4, 12);
        let block_m = create_block(block_l.hash(), 5, 13);
        let block_o = create_block(block_l.hash(), 5, 15);

        let blocks = [
            block_a.clone(),
            // block_b.clone(),
            block_d.clone(),
            block_c.clone(),
            block_p.clone(),
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

        #[allow(clippy::significant_drop_tightening)]
        {
            let inner = state.inner.read().unwrap();
            assert_eq!(inner.orphans.len(), 4);
            assert_eq!(inner.missing.len(), 1);
            drop(inner);
        }

        // Finalize block A
        let retracted = state.finalize(block_a.as_block_ref()).unwrap();
        assert!(retracted.is_empty());

        // Finalize block F
        let retracted = state.finalize(block_f.as_block_ref()).unwrap();
        let expect_retracted = vec![block_c, block_d, block_e, block_p, block_j, block_k];
        for expect in &expect_retracted {
            assert!(retracted.contains(expect), "missing block: {expect:?}");
        }
        assert!(expect_retracted.iter().all(|hash| retracted.contains(hash)));
        assert_eq!(retracted.len(), expect_retracted.len());
    }
}
