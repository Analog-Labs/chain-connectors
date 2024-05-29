#![allow(clippy::large_futures)]

#[allow(clippy::ignored_unit_patterns)]
#[cfg(test)]
mod tests {
    use alloy_sol_types::{sol, SolCall};
    use anyhow::Result;
    use ethers::types::H256;

    use ethers_solc::{artifacts::Source, CompilerInput, EvmVersion, Solc};
    use hex_literal::hex;
    use rosetta_client::Wallet;
    use rosetta_config_ethereum::{AtBlock, CallResult};
    use rosetta_core::BlockchainClient;
    use rosetta_server_polkadot::PolkadotClient;
    use sha3::Digest;
    use std::{collections::BTreeMap, future::Future, path::Path};

    const FUNDING_ACCOUNT_PRIVATE_KEY: [u8; 32] =
        hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");

    /// humanode rpc url
    const HUMANODE_RPC_WS_URL: &str = "ws://127.0.0.1:9944";

    sol! {
        interface TestContract {
            event AnEvent();
            function emitEvent() external;

            function identity(bool a) external view returns (bool);
        }
    }

    /// Run the test in another thread while sending txs to force humanode to mine new blocks
    /// # Panic
    /// Panics if the future panics
    async fn run_test<Fut: Future<Output = ()> + Send + 'static>(future: Fut) {
        // Guarantee that only one test is incrementing blocks at a time
        static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

        // Run the test in another thread
        let test_handler: tokio::task::JoinHandle<()> = tokio::spawn(future);

        // Acquire Lock
        let guard = LOCK.lock().await;

        // Check if the test is finished after acquiring the lock
        if test_handler.is_finished() {
            // Release lock
            drop(guard);

            // Now is safe to panic
            if let Err(err) = test_handler.await {
                std::panic::resume_unwind(err.into_panic());
            }
            return;
        }

        // Now is safe to panic
        if let Err(err) = test_handler.await {
            // Resume the panic on the main task
            std::panic::resume_unwind(err.into_panic());
        }
    }

    #[tokio::test]
    async fn network_status() {
        run_test(async move {
            let client = PolkadotClient::new("humanode-dev", HUMANODE_RPC_WS_URL)
                .await
                .expect("Error creating client");
            // Check if the genesis is consistent
            let genesis_block = client.genesis_block();
            assert_eq!(genesis_block.index, 0);

            // Check if the current block is consistent
            let current_block = client.current_block().await.unwrap();
            if current_block.index > 0 {
                assert_ne!(current_block.hash, genesis_block.hash);
            } else {
                assert_eq!(current_block.hash, genesis_block.hash);
            }

            // Check if the finalized block is consistent
            let finalized_block = client.finalized_block().await.unwrap();
            assert!(finalized_block.index >= genesis_block.index);
        })
        .await;
    }

    #[tokio::test]
    async fn test_account() {
        run_test(async move {
            let client = PolkadotClient::new("humanode-dev", HUMANODE_RPC_WS_URL)
                .await
                .expect("Error creating HumanodeClient");
            let wallet = Wallet::from_config(
                client.config().clone(),
                HUMANODE_RPC_WS_URL,
                None,
                Some(FUNDING_ACCOUNT_PRIVATE_KEY),
            )
            .await
            .unwrap();
            let value = 100 * u128::pow(10, client.config().currency_decimals);

            let _ = wallet.faucet(value).await;
            println!(" ::::: {:?} \n", wallet.balance().await);

            let amount = wallet.balance().await.unwrap();
            assert_eq!(amount, value);
        })
        .await;
    }

    #[tokio::test]
    async fn construction() {
        run_test(async move {
            let client = PolkadotClient::new("humanode-dev", HUMANODE_RPC_WS_URL)
                .await
                .expect("Error creating HumanodeClient");

            let faucet = 100 * u128::pow(10, client.config().currency_decimals);
            let value = u128::pow(10, client.config().currency_decimals);
            let alice =
                Wallet::from_config(client.config().clone(), HUMANODE_RPC_WS_URL, None, None)
                    .await
                    .unwrap();
            let bob = Wallet::from_config(client.config().clone(), HUMANODE_RPC_WS_URL, None, None)
                .await
                .unwrap();
            assert_ne!(alice.public_key(), bob.public_key());

            // Alice and bob have no balance
            let balance = alice.balance().await.unwrap();
            assert_eq!(balance, 0);
            let balance = bob.balance().await.unwrap();
            assert_eq!(balance, 0);

            // Transfer faucets to alice
            alice.faucet(faucet).await.unwrap();
            let balance = alice.balance().await.unwrap();
            assert_eq!(balance, faucet);

            // Alice transfers to bob
            alice.transfer(bob.account(), value, None, None).await.unwrap();
            let amount = bob.balance().await.unwrap();
            assert_eq!(amount, value);
        })
        .await;
    }
}
