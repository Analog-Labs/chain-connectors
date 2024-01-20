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

#[allow(clippy::ignored_unit_patterns)]
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
    use rosetta_core::{types::PartialBlockIdentifier, BlockchainClient};
    use rosetta_server_ethereum::MaybeWsEthereumClient;
    use sha3::Digest;
    use std::{
        collections::BTreeMap,
        future::Future,
        path::Path,
        sync::atomic::{AtomicU64, Ordering},
        time::Duration,
    };

    sol! {
        interface TestContract {
            event AnEvent();
            function emitEvent() external;

            function identity(bool a) external view returns (bool);
        }
    }

    macro_rules! create_account {
        ($name: literal, $value: expr) => {{
            use ethers::core::k256::ecdsa::SigningKey;
            let private_key: [u8; 32] =
                sha3::Keccak256::digest(concat!(module_path!(), "::", $name)).into();
            let address = ::ethers::utils::secret_key_to_address(
                &SigningKey::from_bytes(private_key.as_ref().into()).unwrap(),
            );
            sync_send_funds(address, { $value }).await.unwrap();
            private_key
        }};
    }

    /// Arbitrum faucet account private key.
    const FAUCET_ACCOUNT_PRIVATE_KEY: [u8; 32] =
        hex!("8aab161e2a1e57367b60bd870861e3042c2513f8a856f9fee014e7b96e0a2a36");

    /// Account used exclusively to continuously sending tx to mine new blocks.
    const BLOCK_MANEGER_PRIVATE_KEY: [u8; 32] =
        hex!("b6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659");
    // const BLOCK_MANEGER_ADDRESS: H160 = H160(hex!("3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E"));

    /// Arbitrum rpc url
    const ARBITRUM_RPC_URL: &str = "http://localhost:8547";

    /// Send funds to the provided account
    async fn sync_send_funds<I: Into<U256> + Send>(dest: H160, amount: I) -> Result<()> {
        // Guarantee the faucet nonce is incremented is sequentially
        static NONCE: std::sync::OnceLock<std::sync::atomic::AtomicU64> =
            std::sync::OnceLock::new();

        let amount = amount.into();
        // Connect to the provider
        let wallet = LocalWallet::from_bytes(&FAUCET_ACCOUNT_PRIVATE_KEY)?;
        let provider =
            ethers::providers::Provider::<ethers::providers::Http>::try_from(ARBITRUM_RPC_URL)
                .context("Failed to create HTTP provider")?
                .interval(Duration::from_secs(1));

        // retrieve the current nonce
        let nonce = provider
            .get_transaction_count(wallet.address(), Some(BlockId::Number(BlockNumber::Latest)))
            .await?
            .as_u64();
        let nonce = NONCE.get_or_init(|| AtomicU64::new(nonce));
        let nonce = nonce.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Retrieve chain id
        let chain_id = provider.get_chainid().await?.as_u64();

        // Create a transaction request
        let transaction_request = TransactionRequest {
            from: None,
            to: Some(ethers::types::NameOrAddress::Address(dest)),
            value: Some(amount),
            gas: Some(U256::from(210_000)),
            gas_price: Some(U256::from(500_000_000)),
            nonce: Some(U256::from(nonce)),
            data: None,
            chain_id: Some(chain_id.into()),
        };

        // Sign and send the transaction
        let tx: TypedTransaction = transaction_request.into();
        let signature = wallet.sign_transaction(&tx).await?;
        let tx = tx.rlp_signed(&signature);
        let receipt = provider
            .send_raw_transaction(tx)
            .await?
            .confirmations(1)
            .await?
            .context("failed to retrieve tx receipt")?;

        // Verify if the tx reverted
        if !matches!(receipt.status, Some(U64([1]))) {
            anyhow::bail!("Transaction reverted: {:?}", receipt.transaction_hash);
        }
        Ok(())
    }

    /// Run the test in another thread and while sending txs to force arbitrum to mine new blocks
    /// Panics if the test panics
    async fn run_test<Fut: Future<Output = ()> + Send + 'static>(future: Fut) {
        static TEST_ID_MANAGER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        static CURRENT_TEST: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        static NONCE: std::sync::OnceLock<std::sync::atomic::AtomicU64> =
            std::sync::OnceLock::new();

        // Create a unique id for this test context
        let test_id = TEST_ID_MANAGER.fetch_add(1, Ordering::SeqCst);

        let wallet = LocalWallet::from_bytes(&BLOCK_MANEGER_PRIVATE_KEY).unwrap();
        let provider =
            ethers::providers::Provider::<ethers::providers::Http>::try_from(ARBITRUM_RPC_URL)
                .expect("Failed to create HTTP provider")
                .interval(Duration::from_secs(1));
        let address = H160(hex!("8Db77D3B019a52788bD3804724f5653d7C9Cf0b6"));

        let nonce = provider
            .get_transaction_count(wallet.address(), None)
            .await
            .expect("failed to retrieve account nonce")
            .as_u64();

        let chain_id = provider.get_chainid().await.unwrap().as_u64();

        let nonce = NONCE.get_or_init(|| AtomicU64::new(nonce));

        // Run the test in another thread
        let handler = tokio::spawn(future);

        loop {
            let current_test = CURRENT_TEST.load(std::sync::atomic::Ordering::SeqCst);
            if current_test == 0 {
                let result =
                    CURRENT_TEST.compare_exchange(0, test_id, Ordering::Acquire, Ordering::Relaxed);
                if result.is_ok() {
                    break;
                }
            } else if current_test == test_id {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(4000)).await;
        }

        // Force arbitrum to mine a new block by sending a transaction until the test finishes
        while !handler.is_finished() {
            let next_nonce = nonce.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            // Create a transaction request
            let transaction_request = TransactionRequest {
                from: None,
                to: Some(ethers::types::NameOrAddress::Address(address)),
                value: Some(U256::from(1)),
                gas: Some(U256::from(210_000)),
                gas_price: Some(U256::from(500_000_000)),
                nonce: Some(U256::from(next_nonce)),
                data: None,
                chain_id: Some(chain_id.into()),
            };

            // Sign and send the transaction
            let tx: TypedTransaction = transaction_request.into();
            let signature = match wallet.sign_transaction(&tx).await {
                Ok(signature) => signature,
                Err(err) => {
                    CURRENT_TEST.store(0, std::sync::atomic::Ordering::SeqCst);
                    panic!("{err}");
                },
            };
            let tx: ethers::types::Bytes = tx.rlp_signed(&signature);
            let pending_tx = match provider.send_raw_transaction(tx).await {
                Ok(tx) => tx,
                Err(err) => {
                    CURRENT_TEST.store(0, std::sync::atomic::Ordering::SeqCst);
                    panic!("{err}");
                },
            };

            // Wait 500ms for the tx to be mined
            match pending_tx.confirmations(1).await {
                Ok(Some(_)) => {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                },
                Ok(None) => {
                    CURRENT_TEST.store(0, std::sync::atomic::Ordering::SeqCst);
                    panic!("no tx receipt");
                },
                Err(err) => {
                    CURRENT_TEST.store(0, std::sync::atomic::Ordering::SeqCst);
                    panic!("{err}");
                },
            };
        }
        if CURRENT_TEST.load(Ordering::SeqCst) == test_id {
            CURRENT_TEST.store(0, Ordering::SeqCst);
        }

        // Now is safe to panic
        if let Err(err) = handler.await {
            // Resume the panic on the main task
            std::panic::resume_unwind(err.into_panic());
        }
    }

    #[tokio::test]
    // #[sequential]
    async fn network_status() {
        run_test(async {
            let private_key = create_account!("network_status", 20 * u128::pow(10, 18));
            let client = MaybeWsEthereumClient::new(
                "arbitrum",
                "dev",
                "ws://127.0.0.1:8548",
                Some(private_key),
            )
            .await
            .expect("Error creating client");
            // Check if the genesis is consistent
            let expected_genesis = client.genesis_block().clone();
            let actual_genesis = client
                .block(&PartialBlockIdentifier { index: Some(0), hash: None })
                .await
                .unwrap()
                .block_identifier;

            assert_eq!(expected_genesis, actual_genesis);
            // Check if the current block is consistent
            let expected_current = client.current_block().await.unwrap();
            let actual_current = client
                .block(&PartialBlockIdentifier { index: None, hash: Some(expected_current.hash) })
                .await
                .unwrap();
            assert_eq!(expected_current, actual_current.block_identifier);

            // Check if the finalized block is consistent
            let expected_finalized = client.finalized_block().await.unwrap();
            let actual_finalized = client
                .block(&PartialBlockIdentifier { index: None, hash: Some(expected_finalized.hash) })
                .await
                .unwrap();
            assert_eq!(expected_finalized, actual_finalized.block_identifier);
        })
        .await;
    }

    #[tokio::test]
    // #[sequential]
    async fn test_account() {
        run_test(async {
            let private_key = create_account!("test_account", 20 * u128::pow(10, 18));
            let client = MaybeWsEthereumClient::new(
                "arbitrum",
                "dev",
                "ws://127.0.0.1:8548",
                Some(private_key),
            )
            .await
            .expect("Error creating ArbitrumClient");
            let wallet = Wallet::from_config(
                client.config().clone(),
                "ws://127.0.0.1:8548",
                None,
                Some(private_key),
            )
            .await
            .unwrap();
            let value = 10 * u128::pow(10, client.config().currency_decimals);
            let _ = wallet.faucet(value).await;
            let amount = wallet.balance().await.unwrap();
            assert_eq!((amount.value), (value).to_string());
            assert_eq!(amount.currency, client.config().currency());
            assert!(amount.metadata.is_none());
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
    // #[sequential]
    async fn test_smart_contract() {
        run_test(async {
            let private_key = create_account!("test_smart_contract", 20 * u128::pow(10, 18));
            let client = MaybeWsEthereumClient::new(
                "arbitrum",
                "dev",
                "ws://127.0.0.1:8548",
                Some(private_key),
            )
            .await
            .expect("Error creating ArbitrumClient");
            let faucet = 10 * u128::pow(10, client.config().currency_decimals);
            let wallet = Wallet::from_config(
                client.config().clone(),
                "ws://127.0.0.1:8548",
                None,
                Some(private_key),
            )
            .await
            .unwrap();
            wallet.faucet(faucet).await.unwrap();

            let bytes = compile_snippet(
                r"
                event AnEvent();
                function emitEvent() public {
                    emit AnEvent();
                }
                ",
            )
            .unwrap();
            let tx_hash = wallet.eth_deploy_contract(bytes).await.unwrap();
            let receipt = wallet.eth_transaction_receipt(tx_hash).await.unwrap().unwrap();
            let contract_address = receipt.contract_address.unwrap();
            let tx_hash = {
                let call = TestContract::emitEventCall {};
                wallet.eth_send_call(contract_address.0, call.abi_encode(), 0).await.unwrap()
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
    // #[sequential]
    async fn test_smart_contract_view() {
        run_test(async move {
            let private_key = create_account!("test_smart_contract_view", 20 * u128::pow(10, 18));
            let client = MaybeWsEthereumClient::new(
                "arbitrum",
                "dev",
                "ws://127.0.0.1:8548",
                Some(private_key),
            )
            .await
            .expect("Error creating ArbitrumClient");
            let faucet = 10 * u128::pow(10, client.config().currency_decimals);
            let wallet = Wallet::from_config(
                client.config().clone(),
                "ws://127.0.0.1:8548",
                None,
                Some(private_key),
            )
            .await
            .unwrap();
            wallet.faucet(faucet).await.unwrap();
            let bytes = compile_snippet(
                r"
                function identity(bool a) public view returns (bool) {
                    return a;
                }
                ",
            )
            .unwrap();
            let tx_hash = wallet.eth_deploy_contract(bytes).await.unwrap();
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
