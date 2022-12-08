use dioxus::prelude::*;
use fermi::*;
use lazy_static::lazy_static;
use rosetta_client::BlockchainConfig;
use std::collections::BTreeMap;
use std::path::Path;

pub use rosetta_client::Chain;

pub fn use_chain_from_route(cx: &ScopeState) -> &'static ChainHandle {
    let route = dioxus_router::use_route(cx);
    let segment = route.segment("chain").unwrap();
    let chain: Chain = segment.parse().unwrap();
    CHAINS.get(&chain).unwrap()
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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
            (Chain::Btc, img!("btc.png"), BTC),
            (Chain::Eth, img!("eth.png"), ETH),
            (Chain::Dot, img!("dot.png"), DOT),
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
