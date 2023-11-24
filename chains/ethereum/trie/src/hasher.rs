use hash256_std_hasher::Hash256StdHasher;
pub use hash_db::Hasher;
use primitive_types::H256;
use tiny_keccak::Keccak;

/// Concrete `Hasher` impl for the Keccak-256 hash
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeccakHasher;
impl Hasher for KeccakHasher {
    type Out = H256;
    type StdHasher = Hash256StdHasher;
    const LENGTH: usize = 32;
    fn hash(x: &[u8]) -> Self::Out {
        use tiny_keccak::Hasher as _;
        let mut keccak256 = Keccak::v256();
        keccak256.update(x);
        let mut out = [0; 32];
        keccak256.finalize(&mut out);
        H256(out)
    }
}
