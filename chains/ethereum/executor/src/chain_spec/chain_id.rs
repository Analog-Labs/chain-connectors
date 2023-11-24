#[cfg(feature = "with-serde")]
use rosetta_ethereum_primitives::serde_utils::{deserialize_uint, serialize_uint};

/// Either a named or chain id or the actual id value
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
pub struct ChainId(pub u64);

impl Default for ChainId {
    fn default() -> Self {
        Self::MAINNET
    }
}

impl ChainId {
    pub const MAINNET: Self = Self(1);
    pub const MORDEN: Self = Self(2);
    pub const ROPSTEN: Self = Self(3);
    pub const RINKEBY: Self = Self(4);
    pub const GOERLI: Self = Self(5);
    pub const KOVAN: Self = Self(42);
    pub const HOLESKY: Self = Self(17000);
    pub const SEPOLIA: Self = Self(11_155_111);

    pub const OPTIMISM: Self = Self(10);
    pub const OPTIMISM_KOVAN: Self = Self(69);
    pub const OPTIMISM_GOERLI: Self = Self(420);

    pub const BASE: Self = Self(8453);
    pub const BASE_GOERLI: Self = Self(84531);

    pub const ARBITRUM: Self = Self(42161);
    pub const ARBITRUM_TESTNET: Self = Self(421_611);
    pub const ARBITRUM_GOERLI: Self = Self(421_613);
    pub const ARBITRUM_NOVA: Self = Self(42_170);

    pub const BINANCE_SMART_CHAIN: Self = Self(56);
    pub const BINANCE_SMART_CHAIN_TESTNET: Self = Self(97);

    pub const DEV: Self = Self(1337);

    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns true if the chain contains Optimism configuration.
    #[must_use]
    pub const fn is_optimism(self) -> bool {
        matches!(
            self,
            Self::OPTIMISM |
                Self::OPTIMISM_GOERLI |
                Self::OPTIMISM_KOVAN |
                Self::BASE |
                Self::BASE_GOERLI
        )
    }

    #[must_use]
    pub const fn is_dev(self) -> bool {
        matches!(
            self,
            // Local testnet
            Self::DEV |

            // Public testnet
            Self::MORDEN |
            Self::RINKEBY |
            Self::GOERLI |
            Self::KOVAN |
            Self::HOLESKY |
            Self::SEPOLIA |

            // Arbitrum testnet
            Self::ARBITRUM_GOERLI |
            Self::ARBITRUM_TESTNET |
            Self::ARBITRUM_NOVA |

            // Optimism testnet
            Self::OPTIMISM_GOERLI |
            Self::OPTIMISM_KOVAN |
            Self::BASE |
            Self::BASE_GOERLI
        )
    }

    /// The id of the chain
    #[must_use]
    pub const fn id(self) -> u64 {
        self.0
    }
}

#[cfg(feature = "with-serde")]
impl serde::Serialize for ChainId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize_uint(&self.0, serializer)
    }
}

#[cfg(feature = "with-serde")]
impl<'de> serde::Deserialize<'de> for ChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = deserialize_uint::<u64, D>(deserializer)?;
        Ok(Self(value))
    }
}
