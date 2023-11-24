// use codec::{Decode, Encode};
use crate::rstd::{
    collections::btree_set::BTreeSet,
    iter::{DoubleEndedIterator, IntoIterator},
    vec::Vec,
};
use hash_db::{HashDB, Hasher};

// Note that `LayoutV1` usage here (proof compaction) is compatible
// with `LayoutV0`.
// use super::Layout;

type MemoryDB<H> = memory_db::MemoryDB<H, memory_db::HashKey<H>, trie_db::DBValue>;

/// A proof that some set of key-value pairs are included in the storage trie. The proof contains
/// the storage values so that the partial storage backend can be reconstructed by a verifier that
/// does not already have access to the key-value pairs.
///
/// The proof consists of the set of serialized nodes in the storage trie accessed when looking up
/// the keys covered by the proof. Verifying the proof requires constructing the partial trie from
/// the serialized nodes and performing the key lookups.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StorageProof {
    trie_nodes: BTreeSet<Vec<u8>>,
}

impl StorageProof {
    /// Constructs a storage proof from a subset of encoded trie nodes in a storage backend.
    pub fn new(trie_nodes: impl IntoIterator<Item = Vec<u8>>) -> Self {
        Self { trie_nodes: BTreeSet::from_iter(trie_nodes) }
    }

    /// Returns a new empty proof.
    ///
    /// An empty proof is capable of only proving trivial statements (ie. that an empty set of
    /// key-value pairs exist in storage).
    pub const fn empty() -> Self {
        Self { trie_nodes: BTreeSet::new() }
    }

    /// Returns whether this is an empty proof.
    pub fn is_empty(&self) -> bool {
        self.trie_nodes.is_empty()
    }

    /// Convert into an iterator over encoded trie nodes in lexicographical order constructed
    /// from the proof.
    pub fn into_iter_nodes(self) -> impl Sized + DoubleEndedIterator<Item = Vec<u8>> {
        self.trie_nodes.into_iter()
    }

    /// Create an iterator over encoded trie nodes in lexicographical order constructed
    /// from the proof.
    pub fn iter_nodes(&self) -> impl Sized + DoubleEndedIterator<Item = &Vec<u8>> {
        self.trie_nodes.iter()
    }

    /// Convert into plain node vector.
    pub fn into_nodes(self) -> BTreeSet<Vec<u8>> {
        self.trie_nodes
    }

    /// Creates a [`MemoryDB`](crate::MemoryDB) from `Self`.
    pub fn into_memory_db<H: Hasher>(self) -> MemoryDB<H> {
        self.into()
    }

    /// Creates a [`MemoryDB`](crate::MemoryDB) from `Self` reference.
    pub fn to_memory_db<H: Hasher>(&self) -> MemoryDB<H> {
        self.into()
    }

    /// Merges multiple storage proofs covering potentially different sets of keys into one proof
    /// covering all keys. The merged proof output may be smaller than the aggregate size of the
    /// input proofs due to deduplication of trie nodes.
    pub fn merge(proofs: impl IntoIterator<Item = Self>) -> Self {
        let trie_nodes = proofs
            .into_iter()
            .flat_map(Self::into_iter_nodes)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        Self { trie_nodes }
    }

    // /// Encode as a compact proof with default trie layout.
    // pub fn into_compact_proof<H: Hasher>(
    // 	self,
    // 	root: H::Out,
    // ) -> Result<CompactProof, crate::CompactProofError<H::Out, crate::Error<H::Out>>> {
    // 	let db = self.into_memory_db();
    // 	crate::encode_compact::<Layout<H>, MemoryDB<H>>(&db, &root)
    // }

    // /// Encode as a compact proof with default trie layout.
    // pub fn to_compact_proof<H: Hasher>(
    // 	&self,
    // 	root: H::Out,
    // ) -> Result<CompactProof, crate::CompactProofError<H::Out, crate::Error<H::Out>>> {
    // 	let db = self.to_memory_db();
    // 	crate::encode_compact::<Layout<H>, MemoryDB<H>>(&db, &root)
    // }

    // /// Returns the estimated encoded size of the compact proof.
    // ///
    // /// Running this operation is a slow operation (build the whole compact proof) and should
    // only /// be in non sensitive path.
    // ///
    // /// Return `None` on error.
    // pub fn encoded_compact_size<H: Hasher>(self, root: H::Out) -> Option<usize> {
    // 	let compact_proof = self.into_compact_proof::<H>(root);
    // 	compact_proof.ok().map(|p| p.encoded_size())
    // }
}

impl<H: Hasher> From<StorageProof> for MemoryDB<H> {
    fn from(proof: StorageProof) -> Self {
        From::from(&proof)
    }
}

impl<H: Hasher> From<&StorageProof> for MemoryDB<H> {
    fn from(proof: &StorageProof) -> Self {
        use hash_db::EMPTY_PREFIX;
        use rlp::NULL_RLP;
        let mut db = Self::from_null_node(&NULL_RLP, NULL_RLP.as_ref().into());
        proof.iter_nodes().for_each(|n| {
            db.insert(EMPTY_PREFIX, n);
        });
        db
    }
}

/*
/// Storage proof in compact form.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CompactProof {
    pub encoded_nodes: Vec<Vec<u8>>,
}

impl CompactProof {
    /// Return an iterator on the compact encoded nodes.
    pub fn iter_compact_encoded_nodes(&self) -> impl Iterator<Item = &[u8]> {
        self.encoded_nodes.iter().map(Vec::as_slice)
    }

    /// Decode to a full storage_proof.
    pub fn to_storage_proof<H: Hasher>(
        &self,
        expected_root: Option<&H::Out>,
    ) -> Result<(StorageProof, H::Out), crate::CompactProofError<H::Out, crate::Error<H::Out>>> {
        let mut db = crate::MemoryDB::<H>::new(&[]);
        let root = crate::decode_compact::<Layout<H>, _, _>(
            &mut db,
            self.iter_compact_encoded_nodes(),
            expected_root,
        )?;
        Ok((
            StorageProof::new(db.drain().into_iter().filter_map(|kv| {
                if (kv.1).1 > 0 {
                    Some((kv.1).0)
                } else {
                    None
                }
            })),
            root,
        ))
    }

    /// Convert self into a [`MemoryDB`](crate::MemoryDB).
    ///
    /// `expected_root` is the expected root of this compact proof.
    ///
    /// Returns the memory db and the root of the trie.
    pub fn to_memory_db<H: Hasher>(
        &self,
        expected_root: Option<&H::Out>,
    ) -> Result<(crate::MemoryDB<H>, H::Out), crate::CompactProofError<H::Out, crate::Error<H::Out>>>
    {
        let mut db = crate::MemoryDB::<H>::new(&[]);
        let root = crate::decode_compact::<Layout<H>, _, _>(
            &mut db,
            self.iter_compact_encoded_nodes(),
            expected_root,
        )?;

        Ok((db, root))
    }
}
*/
