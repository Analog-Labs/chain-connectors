use anyhow::Result;
use dioxus::prelude::*;
use fermi::*;
use lazy_static::lazy_static;
use rosetta_client::BlockchainConfig;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Chain {
    pub blockchain: &'static str,
    pub network: &'static str,
}

impl Chain {
    pub const BTC: Self = Self::new("bitcoin", "regtest");
    pub const ETH: Self = Self::new("ethereum", "dev");
    pub const DOT: Self = Self::new("polkadot", "dev");
    pub const CHAINS: &'static [Self] = &[Self::BTC, Self::ETH, Self::DOT];

    pub const fn new(blockchain: &'static str, network: &'static str) -> Self {
        Self {
            blockchain,
            network,
        }
    }

    pub fn from_str(blockchain: &str, network: &str) -> Result<Self> {
        for chain in Self::CHAINS {
            if chain.blockchain == blockchain && chain.network == network {
                return Ok(*chain);
            }
        }
        anyhow::bail!("unsupported network")
    }

    pub fn config(self) -> BlockchainConfig {
        rosetta_client::create_config(self.blockchain, self.network).unwrap()
    }
}

pub fn use_chain_from_route(cx: &ScopeState) -> &'static ChainHandle {
    let route = dioxus_router::use_route(cx);
    let blockchain = route.segment("blockchain").unwrap();
    let network = route.segment("network").unwrap();
    let chain = Chain::from_str(blockchain, network).unwrap();
    CHAINS.get(&chain).unwrap()
}

#[derive(Clone)]
pub struct ChainHandle {
    info: ChainInfo,
    state: AtomRef<ChainState>,
}

impl ChainHandle {
    pub fn info(&self) -> &ChainInfo {
        &self.info
    }

    pub fn use_state<'a>(&self, cx: &'a ScopeState) -> &'a UseAtomRef<ChainState> {
        use_atom_ref(cx, self.state)
    }
}

#[derive(Clone)]
pub struct ChainInfo {
    pub chain: Chain,
    pub config: BlockchainConfig,
    pub icon: &'static Path,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ChainState {
    pub account: String,
    pub balance: String,
}

static BTC: AtomRef<ChainState> = |_| ChainState::default();
static ETH: AtomRef<ChainState> = |_| ChainState::default();
static DOT: AtomRef<ChainState> = |_| ChainState::default();

lazy_static! {
    pub static ref CHAINS: BTreeMap<Chain, ChainHandle> = {
        let data = [
            (Chain::BTC, img!("btc.png"), BTC),
            (Chain::ETH, img!("eth.png"), ETH),
            (Chain::DOT, img!("dot.png"), DOT),
        ];

        let mut chains = BTreeMap::new();
        for (chain, icon, state) in data {
            let config = chain.config();
            let info = ChainInfo {
                chain,
                config,
                icon: icon.as_ref(),
            };
            chains.insert(chain, ChainHandle { info, state });
        }
        chains
    };
}
