use crate::components::alerts::{Alert, ALERTS};
use crate::components::token_list::{Token, TOKENS};
use anyhow::Result;
use dioxus::prelude::*;
use fermi::*;
use futures::channel::mpsc::UnboundedReceiver;
use futures::stream::StreamExt;
use rosetta_client::Chain;

pub enum Action {
    SyncBalances,
    SyncBalance(Chain),
}

pub fn use_worker(cx: &Scope) {
    let state = State::new(&cx);
    use_coroutine(&cx, |rx| worker(rx, state));
}

#[derive(Clone)]
struct State {
    tokens: UseAtomRef<Vec<Token>>,
    alerts: UseAtomRef<Vec<Alert>>,
}

impl State {
    pub fn new(cx: &Scope) -> Self {
        let alerts = use_atom_ref(cx, ALERTS).clone();
        let tokens = use_atom_ref(cx, TOKENS).clone();
        Self { alerts, tokens }
    }

    fn set_balance(&self, chain: Chain, balance: String) {
        self.tokens
            .write()
            .iter_mut()
            .find(|token| token.chain() == chain)
            .map(move |token| token.set_balance(balance.into()));
    }

    fn add_error(&self, error: String) {
        self.alerts.write().push(Alert::error(error));
    }
}

async fn worker(mut rx: UnboundedReceiver<Action>, state: State) {
    while let Some(action) = rx.next().await {
        match action {
            Action::SyncBalances => {
                futures::future::join_all([
                    sync_balance(&state, Chain::Btc),
                    sync_balance(&state, Chain::Eth),
                    sync_balance(&state, Chain::Dot),
                ])
                .await;
            }
            Action::SyncBalance(chain) => {
                sync_balance(&state, chain).await;
            }
        }
    }
}

async fn sync_balance(state: &State, chain: Chain) {
    match fetch_balance(chain).await {
        Ok(balance) => state.set_balance(chain, balance),
        Err(error) => state.add_error(error.to_string()),
    }
}

async fn fetch_balance(chain: Chain) -> Result<String> {
    let wallet = rosetta_client::create_wallet(chain, None, None).await?;
    let amount = wallet.balance().await?;
    let balance = rosetta_client::amount_to_string(&amount)?;
    Ok(balance)
}
