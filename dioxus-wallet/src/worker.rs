use crate::components::alerts::{Alert, ALERTS};
use crate::state::{Chain, ChainState, CHAINS};
use anyhow::{Error, Result};
use dioxus::prelude::*;
use fermi::*;
use futures::channel::mpsc::UnboundedReceiver;
use futures::stream::StreamExt;
use rosetta_client::{Signer, Wallet};
use std::collections::btree_map::{BTreeMap, Entry};

pub enum Action {
    SyncAccount(Chain),
    SyncBalance(Chain),
}

pub fn use_worker(cx: &Scope) {
    let state = State::new(cx);
    use_coroutine(cx, |rx| worker(rx, state));
}

#[derive(Clone)]
struct State {
    chains: BTreeMap<Chain, UseAtomRef<ChainState>>,
    alerts: UseAtomRef<Vec<Alert>>,
}

impl State {
    fn new(cx: &Scope) -> Self {
        let alerts = use_atom_ref(cx, ALERTS).clone();
        let chains = CHAINS
            .iter()
            .map(|(chain, handle)| (*chain, handle.use_state(cx).clone()))
            .collect();
        Self { alerts, chains }
    }

    fn chain(&self, chain: Chain) -> &UseAtomRef<ChainState> {
        self.chains.get(&chain).unwrap()
    }

    fn set_balance(&self, chain: Chain, balance: String) {
        self.chain(chain).write().balance = balance;
    }

    fn set_account(&self, chain: Chain, account: String) {
        self.chain(chain).write().account = account;
    }

    fn add_error(&self, error: Error) {
        self.alerts.write().push(Alert::error(error.to_string()));
    }
}

struct Chains {
    signer: Signer,
    chains: BTreeMap<Chain, Wallet>,
}

impl Chains {
    fn new() -> Result<Self> {
        let keyfile = rosetta_client::default_keyfile()?;
        let signer = rosetta_client::open_or_create_keyfile(&keyfile)?;
        Ok(Self {
            signer,
            chains: Default::default(),
        })
    }

    async fn wallet(&mut self, chain: Chain) -> Result<&Wallet> {
        if let Entry::Vacant(entry) = self.chains.entry(chain) {
            let wallet = Wallet::new(chain.url(), chain.config(), &self.signer).await?;
            entry.insert(wallet);
        }
        Ok(self.chains.get(&chain).unwrap())
    }

    async fn account(&mut self, chain: Chain) -> Result<String> {
        let wallet = self.wallet(chain).await?;
        Ok(wallet.account().address.clone())
    }

    async fn balance(&mut self, chain: Chain) -> Result<String> {
        let wallet = self.wallet(chain).await?;
        let amount = wallet.balance().await?;
        rosetta_client::amount_to_string(&amount)
    }
}

async fn worker(mut rx: UnboundedReceiver<Action>, state: State) {
    let mut chains = Chains::new().unwrap();
    while let Some(action) = rx.next().await {
        match action {
            Action::SyncBalance(chain) => match chains.balance(chain).await {
                Ok(balance) => state.set_balance(chain, balance),
                Err(error) => state.add_error(error),
            },
            Action::SyncAccount(chain) => match chains.account(chain).await {
                Ok(account) => state.set_account(chain, account),
                Err(error) => state.add_error(error),
            },
        }
    }
}
