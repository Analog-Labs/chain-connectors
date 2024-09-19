#![allow(clippy::large_futures)]

//! # Arbitrum Nitro Testnet Rosetta Server
//!
//! This module contains the production test for an Arbitrum Rosetta server implementation
//! specifically designed for interacting with the Arbitrum Nitro Testnet. The code includes
//! tests for network status, account management, and smart contract interaction.
//!
//! ## Features
//!
//! - Network status tests to ensure proper connection and consistency with the Arbitrum Nitro
//!   Testnet.
//! - Account tests, including faucet funding, balance retrieval, and error handling.
//! - Smart contract tests covering deployment, event emission, and view function calls.
//!
//! ## Dependencies
//!
//! - `anyhow`: For flexible error handling.
//! - `alloy_sol_types`: Custom types and macros for interacting with Solidity contracts.
//! - `ethers`: Ethereum library for interaction with Ethereum clients.
//! - `ethers_solc`: Integration for compiling Solidity code using the Solc compiler.
//! - `rosetta_client`: Client library for Rosetta API interactions.
//! - `rosetta_config_ethereum`: Configuration for Ethereum Rosetta server.
//! - `rosetta_server_arbitrum`: Custom client implementation for interacting with Arbitrum.
//! - `sha3`: SHA-3 (Keccak) implementation for hashing.
//! - `tokio`: Asynchronous runtime for running async functions.
//!
//! ## Usage
//!
//! To run the tests, execute the following command:
//!
//! ```sh
//! cargo test --package rosetta-testing-arbitrum --lib -- tests --nocapture
//! ```
//!
//! Note: The code assumes a local Arbitrum Nitro Testnet node running on `ws://127.0.0.1:8548` and
//! a local Ethereum node on `http://localhost:8545`. Ensure that these endpoints are configured correctly.

#[allow(clippy::ignored_unit_patterns, clippy::pub_underscore_fields)]
#[cfg(test)]
mod tests {
    use alloy_sol_types::{sol, SolCall};
    use anyhow::{Context, Result};
    use ethers::{
        providers::Middleware,
        signers::{LocalWallet, Signer},
        types::{
            transaction::eip2718::TypedTransaction, BlockId, BlockNumber, TransactionRequest, H160,
            H256, U256, U64,
        },
    };
    use ethers_solc::{artifacts::Source, CompilerInput, EvmVersion, Solc};
    use hex_literal::hex;
    use rosetta_client::Wallet;
    use rosetta_config_ethereum::{AtBlock, CallResult};
    use rosetta_core::BlockchainClient;
    use rosetta_server_ethereum::MaybeWsEthereumClient;
    use sha3::Digest;
    use std::{collections::BTreeMap, future::Future, path::Path, time::Duration};

    /// Account used to fund other testing accounts.
    const FUNDING_ACCOUNT_PRIVATE_KEY: [u8; 32] =
        hex!("b6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659");

    /// Account used exclusively to continuously sending tx to mine new blocks.
    const BLOCK_INCREMENTER_PRIVATE_KEY: [u8; 32] =
        hex!("b6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659");

    /// Arbitrum rpc url
    const ARBITRUM_RPC_HTTP_URL: &str = "http://127.0.0.1:8547";
    const ARBITRUM_RPC_WS_URL: &str = "ws://127.0.0.1:8548";

    type WsProvider = ethers::providers::Provider<ethers::providers::Ws>;

    sol! {
        interface TestContract {
            event AnEvent();
            function emitEvent() external;

            function identity(bool a) external view returns (bool);
        }
    }

    /// Send funds from funding account to the provided account.
    /// This function is can be called concurrently.
    async fn sync_send_funds(dest: H160, amount: u128) -> Result<()> {
        // Guarantee the funding account nonce is incremented atomically
        static NONCE: tokio::sync::Mutex<u64> = tokio::sync::Mutex::const_new(0);

        // Connect to the provider
        let wallet = LocalWallet::from_bytes(&FUNDING_ACCOUNT_PRIVATE_KEY)?;
        let provider =
            ethers::providers::Provider::<ethers::providers::Http>::try_from(ARBITRUM_RPC_HTTP_URL)
                .context("Failed to create HTTP provider")?
                .interval(Duration::from_secs(1));

        // Retrieve chain id
        let chain_id = provider.get_chainid().await?.as_u64();

        // Acquire nonce lock
        let mut nonce_lock = NONCE.lock().await;

        // Initialize nonce if necessary
        if *nonce_lock == 0 {
            // retrieve the current nonce, used to initialize the nonce if necessary, once
            // `OnceLock` doesn't support async functions
            let current_nonce = provider
                .get_transaction_count(wallet.address(), Some(BlockId::Number(BlockNumber::Latest)))
                .await?
                .as_u64();
            *nonce_lock = current_nonce;
        }

        // Create a transaction request
        let transaction_request = TransactionRequest {
            from: None,
            to: Some(ethers::types::NameOrAddress::Address(dest)),
            value: Some(U256::from(amount)),
            gas: Some(U256::from(210_000)),
            gas_price: Some(U256::from(500_000_000)),
            nonce: Some(U256::from(*nonce_lock)),
            data: None,
            chain_id: Some(chain_id.into()),
        };

        // Sign and send the transaction
        let tx: TypedTransaction = transaction_request.into();
        let signature = wallet.sign_transaction(&tx).await?;
        let tx = tx.rlp_signed(&signature);
        let pending_tx = provider.send_raw_transaction(tx).await?;

        // Increment and release nonce lock
        // increment only after successfully send the tx to avoid nonce gaps
        *nonce_lock += 1;
        drop(nonce_lock);

        // Verify if the tx reverted
        let receipt =
            pending_tx.confirmations(1).await?.context("failed to retrieve tx receipt")?;
        if !matches!(receipt.status, Some(U64([1]))) {
            anyhow::bail!("Transaction reverted: {:?}", receipt.transaction_hash);
        }
        Ok(())
    }

    /// Creates a random account and send funds to it
    async fn create_test_account(initial_balance: u128) -> Result<[u8; 32]> {
        use ethers::core::k256::ecdsa::SigningKey;
        use rand_core::OsRng;
        let signing_key = SigningKey::random(&mut OsRng);
        let address = ::ethers::utils::secret_key_to_address(&signing_key);
        sync_send_funds(address, initial_balance).await?;
        Ok(signing_key.to_bytes().into())
    }

    /// Run the test in another thread while sending txs to force arbitrum to mine new blocks
    /// # Panic
    /// Panics if the future panics
    async fn run_test<Fut: Future<Output = ()> + Send + 'static>(future: Fut) {
        // Guarantee that only one test is incrementing blocks at a time
        static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

        // Run the test in another thread
        let test_handler = tokio::spawn(future);

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

        // Connect to arbitrum node
        let wallet = LocalWallet::from_bytes(&BLOCK_INCREMENTER_PRIVATE_KEY).unwrap();
        let provider = WsProvider::connect(ARBITRUM_RPC_WS_URL)
            .await
            .map(|provider| provider.interval(Duration::from_millis(500)))
            .unwrap();

        // Retrieve chain id
        let chain_id = provider.get_chainid().await.unwrap().as_u64();

        // Retrieve current nonce
        let mut nonce = provider
            .get_transaction_count(wallet.address(), None)
            .await
            .expect("failed to retrieve account nonce")
            .as_u64();

        // Create a transaction request
        let transaction_request = TransactionRequest {
            from: None,
            to: Some(wallet.address().into()),
            value: None,
            gas: Some(U256::from(210_000)),
            gas_price: Some(U256::from(500_000_000)),
            nonce: None,
            data: None,
            chain_id: Some(chain_id.into()),
        };
        let mut tx: TypedTransaction = transaction_request.into();

        // Mine a new block by sending a transaction until the test finishes
        while !test_handler.is_finished() {
            // Set tx nonce
            tx.set_nonce(nonce);

            // Increment nonce
            nonce += 1;

            // Sign and send the transaction
            let signature = wallet.sign_transaction(&tx).await.expect("failed to sign tx");
            let tx: ethers::types::Bytes = tx.rlp_signed(&signature);
            let receipt = provider
                .send_raw_transaction(tx)
                .await
                .unwrap()
                .confirmations(1)
                .await
                .unwrap()
                .expect("tx receipt not found");

            // Verify if the tx reverted
            assert!(receipt.status.unwrap().as_u64() == 1, "Transaction reverted: {receipt:?}");

            // Wait 500ms for the tx to be mined
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        // Release lock
        drop(guard);

        // Now is safe to panic
        if let Err(err) = test_handler.await {
            // Resume the panic on the main task
            std::panic::resume_unwind(err.into_panic());
        }
    }

    #[tokio::test]
    async fn network_status() {
        let private_key = create_test_account(20 * u128::pow(10, 18)).await.unwrap();
        run_test(async move {
            let client = MaybeWsEthereumClient::new(
                "arbitrum",
                "dev",
                ARBITRUM_RPC_WS_URL,
                Some(private_key),
            )
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
        let private_key = create_test_account(20 * u128::pow(10, 18)).await.unwrap();
        run_test(async move {
            let client = MaybeWsEthereumClient::new(
                "arbitrum",
                "dev",
                ARBITRUM_RPC_WS_URL,
                Some(private_key),
            )
            .await
            .expect("Error creating ArbitrumClient");
            let wallet = Wallet::from_config(
                client.config().clone(),
                ARBITRUM_RPC_WS_URL,
                None,
                Some(private_key),
            )
            .await
            .unwrap();
            let value = 10 * u128::pow(10, client.config().currency_decimals);
            let _ = wallet.faucet(value, None).await;
            let amount = wallet.balance().await.unwrap();
            assert_eq!(amount, value);
        })
        .await;
    }

    fn compile_snippet(source: &str) -> Result<Vec<u8>> {
        let solc = Solc::default();
        let source = format!("contract Contract {{ {source} }}");
        let mut sources = BTreeMap::new();
        sources.insert(Path::new("contract.sol").into(), Source::new(source));
        let input = CompilerInput::with_sources(sources)[0]
            .clone()
            .evm_version(EvmVersion::Homestead);
        let output = solc.compile_exact(&input)?;
        let file = output.contracts.get("contract.sol").unwrap();
        let contract = file.get("Contract").unwrap();
        let bytecode = contract
            .evm
            .as_ref()
            .unwrap()
            .bytecode
            .as_ref()
            .unwrap()
            .object
            .as_bytes()
            .unwrap()
            .to_vec();
        Ok(bytecode)
    }

    #[tokio::test]
    async fn test_smart_contract() {
        let private_key = create_test_account(20 * u128::pow(10, 18)).await.unwrap();
        run_test(async move {
            let client = MaybeWsEthereumClient::new(
                "arbitrum",
                "dev",
                ARBITRUM_RPC_WS_URL,
                Some(private_key),
            )
            .await
            .expect("Error creating ArbitrumClient");
            let faucet = 10 * u128::pow(10, client.config().currency_decimals);
            let wallet = Wallet::from_config(
                client.config().clone(),
                ARBITRUM_RPC_WS_URL,
                None,
                Some(private_key),
            )
            .await
            .unwrap();
            wallet.faucet(faucet, None).await.unwrap();

            let bytes = compile_snippet(
                r"
                event AnEvent();
                function emitEvent() public {
                    emit AnEvent();
                }
                ",
            )
            .unwrap();
            let tx_hash = wallet.eth_deploy_contract(bytes).await.unwrap().tx_hash().0;
            let receipt = wallet.eth_transaction_receipt(tx_hash).await.unwrap().unwrap();
            let contract_address = receipt.contract_address.unwrap();
            let tx_hash = {
                let call = TestContract::emitEventCall {};
                wallet
                    .eth_send_call(contract_address.0, call.abi_encode(), 0, None, None)
                    .await
                    .unwrap()
                    .tx_hash()
                    .0
            };
            let receipt = wallet.eth_transaction_receipt(tx_hash).await.unwrap().unwrap();
            assert_eq!(receipt.logs.len(), 1);
            let topic = receipt.logs[0].topics[0];
            let expected = H256(sha3::Keccak256::digest("AnEvent()").into());
            assert_eq!(topic, expected);
        })
        .await;
    }

    #[tokio::test]
    async fn test_smart_contract_view() {
        let private_key = create_test_account(20 * u128::pow(10, 18)).await.unwrap();
        run_test(async move {
            let client = MaybeWsEthereumClient::new(
                "arbitrum",
                "dev",
                ARBITRUM_RPC_WS_URL,
                Some(private_key),
            )
            .await
            .expect("Error creating ArbitrumClient");
            let faucet = 10 * u128::pow(10, client.config().currency_decimals);
            let wallet = Wallet::from_config(
                client.config().clone(),
                ARBITRUM_RPC_WS_URL,
                None,
                Some(private_key),
            )
            .await
            .unwrap();
            wallet.faucet(faucet, None).await.unwrap();
            let bytes = compile_snippet(
                r"
                function identity(bool a) public view returns (bool) {
                    return a;
                }
                ",
            )
            .unwrap();
            let tx_hash = wallet.eth_deploy_contract(bytes).await.unwrap().tx_hash().0;
            let receipt = wallet.eth_transaction_receipt(tx_hash).await.unwrap().unwrap();
            let contract_address = receipt.contract_address.unwrap();

            let response = {
                let call = TestContract::identityCall { a: true };
                wallet
                    .eth_view_call(contract_address.0, call.abi_encode(), AtBlock::Latest)
                    .await
                    .unwrap()
            };
            assert_eq!(
                response,
                CallResult::Success(
                    hex!("0000000000000000000000000000000000000000000000000000000000000001")
                        .to_vec()
                )
            );
        })
        .await;
    }
}
