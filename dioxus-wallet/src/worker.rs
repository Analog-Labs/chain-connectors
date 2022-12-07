use crate::components::alerts::{Alert, ALERTS};
use crate::state::{Chain, ChainState, CHAINS};
use anyhow::{Error, Result};
use dioxus::prelude::*;
use fermi::*;
use futures::channel::mpsc::UnboundedReceiver;
use rosetta_client::Wallet;
use std::time::Duration;

pub fn use_chain_workers(cx: &Scope) -> Result<()> {
    let signer = rosetta_client::create_signer(None)?;
    for (chain, _) in CHAINS.iter() {
        let state = State::new(cx, *chain);
        let wallet = Wallet::new(chain.url(), chain.config(), &signer)?;
        state.set_account(wallet.account().address.clone());
        use_coroutine(cx, |_: UnboundedReceiver<()>| chain_worker(state, wallet));
    }
    Ok(())
}

#[derive(Clone)]
struct State {
    chain: UseAtomRef<ChainState>,
    alerts: UseAtomRef<Vec<Alert>>,
}

impl State {
    fn new(cx: &Scope, chain: Chain) -> Self {
        let alerts = use_atom_ref(cx, ALERTS).clone();
        let handle = CHAINS.get(&chain).unwrap();
        let chain = handle.use_state(cx).clone();
        Self { alerts, chain }
    }

    fn set_balance(&self, balance: String) {
        self.chain.write().balance = balance;
    }

    fn set_account(&self, account: String) {
        self.chain.write().account = account;
    }

    fn add_error(&self, error: Error) {
        self.alerts.write().push(Alert::error(error.to_string()));
    }
}

async fn chain_worker(state: State, wallet: Wallet) {
    loop {
        if let Err(error) = fallible_chain_worker(&state, &wallet).await {
            state.add_error(error);
            async_std::task::sleep(Duration::from_secs(10)).await;
        }
    }
}

async fn fallible_chain_worker(state: &State, wallet: &Wallet) -> Result<()> {
    let mut synced = None;
    loop {
        let current = Some(wallet.status().await?);
        if current == synced {
            async_std::task::sleep(Duration::from_secs(10)).await;
            continue;
        }
        let amount = wallet.balance().await?;
        let balance = rosetta_client::amount_to_string(&amount)?;
        state.set_balance(balance);
        synced = current;
    }
}
