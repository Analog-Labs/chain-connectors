use crate::rstd::hash::{BuildHasher, Hasher};

#[cfg(not(feature = "std"))]
pub mod rand_state {
    use crate::{
        hasher::KeccakHasher,
        rstd::{
            mem,
            sync::atomic::{AtomicU64, Ordering},
        },
    };
    use hash_db::Hasher as DBHasher;
    use spin::Once;

    static RANDOM_STATE: Once<ahash::RandomState> = Once::INIT;

    static GLOBAL_STATE: AtomicU64 = AtomicU64::new(0);
    fn random_state_from(entropy: impl AsRef<[u8]>) -> ahash::RandomState {
        let mut out = unsafe {
            let hash = KeccakHasher::hash(entropy.as_ref()).0;
            mem::transmute::<[u8; 32], [u64; 4]>(hash)
        };
        {
            // Spin the state
            out[0] = GLOBAL_STATE.fetch_xor(out[0], Ordering::Relaxed);
        }
        ahash::RandomState::generate_with(out[0], out[1], out[2], out[3])
    }

    pub fn build_random_state() -> ahash::RandomState {
        let state = GLOBAL_STATE.fetch_add(1, Ordering::SeqCst);
        random_state_from(state.to_le_bytes().as_ref())
    }

    pub fn global_random_state() -> &'static ahash::RandomState {
        RANDOM_STATE.call_once(build_random_state)
    }
}

#[cfg(feature = "std")]
pub mod rand_state {
    lazy_static::lazy_static! {
        static ref RANDOM_STATE: ahash::RandomState = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            ahash::RandomState::generate_with(rng.gen(), rng.gen(), rng.gen(), rng.gen())
        };
    }

    pub fn build_random_state() -> ahash::RandomState {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        ahash::RandomState::generate_with(rng.gen(), rng.gen(), rng.gen(), rng.gen())
    }

    pub fn global_random_state() -> &'static ahash::RandomState {
        &RANDOM_STATE
    }
}

/// The default hash builder used by the LRU map.
#[derive(Debug, Clone)]
pub struct RandomState(ahash::RandomState);

impl RandomState {
    /// Constructs a new `RandomState`.
    #[inline]
    pub fn global_build_hasher() -> ahash::AHasher {
        let rng = rand_state::global_random_state();
        rng.build_hasher()
    }
}

impl Default for RandomState {
    #[inline]
    fn default() -> Self {
        Self(rand_state::build_random_state())
    }
}

impl BuildHasher for RandomState {
    type Hasher = DefaultHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        DefaultHasher(self.0.build_hasher())
    }
}

/// The default hasher used by the LRU map.
// Create a newtype to isolate the public API.
pub struct DefaultHasher(ahash::AHasher);

impl Hasher for DefaultHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }

    #[inline]
    fn write_u8(&mut self, value: u8) {
        self.0.write_u8(value);
    }

    #[inline]
    fn write_u16(&mut self, value: u16) {
        self.0.write_u16(value);
    }

    #[inline]
    fn write_u32(&mut self, value: u32) {
        self.0.write_u32(value);
    }

    #[inline]
    fn write_u128(&mut self, value: u128) {
        self.0.write_u128(value);
    }

    #[inline]
    fn write_usize(&mut self, value: usize) {
        self.0.write_usize(value);
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        self.0.write_u64(value);
    }
}
