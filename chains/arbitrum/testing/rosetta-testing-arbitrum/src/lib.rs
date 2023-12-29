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
//! - `sequential_test`: Macro for ensuring sequential execution of asynchronous tests.
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
        types::{transaction::eip2718::TypedTransaction, TransactionRequest, H160, H256, U256},
        utils::hex,
    };
    use ethers_solc::{artifacts::Source, CompilerInput, EvmVersion, Solc};
    use rosetta_client::Wallet;
    use rosetta_config_ethereum::{AtBlock, CallResult};
    use rosetta_core::{types::PartialBlockIdentifier, BlockchainClient};
    use rosetta_server_arbitrum::ArbitrumClient;
    use sequential_test::sequential;
    use sha3::Digest;
    use std::{collections::BTreeMap, future::Future, path::Path, str::FromStr};
    use tokio::sync::oneshot::{error::TryRecvError, Receiver};

    sol! {
        interface TestContract {
            event AnEvent();
            function emitEvent() external;

            function identity(bool a) external view returns (bool);
        }
    }

    async fn run_test<Fut: Future + Send>(_future: Fut, mut stop_rx: Receiver<()>) {
        loop {
            if matches!(stop_rx.try_recv(), Ok(()) | Err(TryRecvError::Closed)) {
                break;
            }
            let hex_string = "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659";
            let hex_string = &hex_string[2..];
            let mut private_key_result = [0; 32];
            let bytes = hex::decode(hex_string).expect("Failed to decode hex string");
            private_key_result.copy_from_slice(&bytes);
            let result =
                ArbitrumClient::new("dev", "ws://127.0.0.1:8548", Some(private_key_result)).await;
            assert!(result.is_ok(), "Error creating ArbitrumClient");
            let wallet = LocalWallet::from_bytes(&private_key_result).unwrap();
            let provider = ethers::providers::Provider::<ethers::providers::Http>::try_from(
                "http://localhost:8547",
            )
            .expect("Failed to create HTTP provider");
            let address = H160::from_str("0x8Db77D3B019a52788bD3804724f5653d7C9Cf0b6").unwrap();
            let nonce = provider
                .get_transaction_count(
                    H160::from_str("0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E").unwrap(),
                    None,
                )
                .await
                .unwrap();
            let chain_id = provider.get_chainid().await.unwrap().as_u64();
            // Create a transaction request
            let transaction_request = TransactionRequest {
                from: None,
                to: Some(ethers::types::NameOrAddress::Address(address)),
                value: Some(U256::from(1)),
                gas: Some(U256::from(210_000)),
                gas_price: Some(U256::from(500_000_000)),
                nonce: Some(nonce),
                data: None,
                chain_id: Some(chain_id.into()),
            };
            let tx: TypedTransaction = transaction_request.into();
            let signature = wallet.sign_transaction(&tx).await.unwrap();
            let tx = tx.rlp_signed(&signature);
            let _ = provider
                .send_raw_transaction(tx)
                .await
                .unwrap()
                .confirmations(1)
                .await
                .unwrap()
                .context("failed to retrieve tx receipt")
                .unwrap()
                .transaction_hash
                .0
                .to_vec();
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    #[tokio::test]
    #[sequential]
    async fn network_status() {
        let hex_string = "0x8aab161e2a1e57367b60bd870861e3042c2513f8a856f9fee014e7b96e0a2a36";
        // Remove the "0x" prefix
        let hex_string = &hex_string[2..];
        let mut result = [0; 32];
        // Parse the hexadecimal string into a Vec<u8>
        let bytes = hex::decode(hex_string).expect("Failed to decode hex string");
        result.copy_from_slice(&bytes);

        match ArbitrumClient::new("dev", "ws://127.0.0.1:8548", Some(result)).await {
            Ok(client) => {
                // The client was successfully created, continue with the rest of the function
                // ...
                println!("Client created successfully");
                // Check if the genesis is consistent
                let expected_genesis = client.genesis_block().clone();
                tracing::info!("expected_genesis=> {expected_genesis:?}");
                let actual_genesis = client
                    .block(&PartialBlockIdentifier { index: Some(0), hash: None })
                    .await
                    .unwrap()
                    .block_identifier;

                tracing::info!("actual_genesis=> {actual_genesis:?}");
                assert_eq!(expected_genesis, actual_genesis);
                // Check if the current block is consistent
                let expected_current = client.current_block().await.unwrap();
                tracing::info!("expected_current=> {expected_current:?}");
                let actual_current = client
                    .block(&PartialBlockIdentifier {
                        index: None,
                        hash: Some(expected_current.hash.clone()),
                    })
                    .await;
                match actual_current {
                    Ok(block) => {
                        tracing::info!("actual_current=> {:?}", block.block_identifier);
                        assert_eq!(expected_current, block.block_identifier);
                    },
                    Err(error) => {
                        tracing::error!("{error:?}");
                    },
                }

                // Check if the finalized block is consistent
                let expected_finalized = client.finalized_block().await.unwrap();
                tracing::info!("expected_finalized=> {expected_finalized:?}");
                let actual_finalized = client
                    .block(&PartialBlockIdentifier {
                        index: None,
                        hash: Some(expected_finalized.hash.clone()),
                    })
                    .await;

                match actual_finalized {
                    Ok(block) => {
                        tracing::info!("actual_finalized=> {:?}", block.block_identifier);
                        assert_eq!(expected_finalized, block.block_identifier);
                    },
                    Err(error) => {
                        tracing::error!("ad{error:?}");
                    },
                }

                tracing::info!("Arbitrum network is up and running");
            },
            Err(err) => {
                // An error occurred while creating the client, handle the error here
                eprintln!("Error creating client: {err:?}");
            },
        }
    }

    #[tokio::test]
    #[sequential]
    async fn test_account() {
        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
        let handler = tokio::spawn(async move {
            run_test(async {}, stop_rx).await;
        });
        let hex_string = "0x8aab161e2a1e57367b60bd870861e3042c2513f8a856f9fee014e7b96e0a2a36";
        // Remove the "0x" prefix
        let hex_string = &hex_string[2..];
        let mut private_key_result = [0; 32];
        // Parse the hexadecimal string into a Vec<u8>
        let bytes = hex::decode(hex_string).expect("Failed to decode hex string");
        private_key_result.copy_from_slice(&bytes);
        let result =
            ArbitrumClient::new("dev", "ws://127.0.0.1:8548", Some(private_key_result)).await;
        assert!(result.is_ok(), "Error creating ArbitrumClient");
        let client = result.unwrap();
        let value = 100 * u128::pow(10, client.config().currency_decimals);
        let wallet = Wallet::from_config(
            client.config().clone(),
            "ws://127.0.0.1:8548",
            None,
            Some(private_key_result),
        )
        .await;
        match wallet {
            Ok(w) => {
                let _ = w.faucet(value).await;
                let amount = w.balance().await.unwrap();
                assert_eq!((amount.value), (value).to_string());
                assert_eq!(amount.currency, client.config().currency());
                assert!(amount.metadata.is_none());
            },
            Err(e) => {
                println!("Error : {e:?}");
            },
        }
        stop_tx.send(()).expect("Failed to send stop signal");
        handler.await.expect("Failed to join the background task");
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

    #[allow(clippy::needless_raw_string_hashes)]
    #[tokio::test]
    #[sequential]
    async fn test_smart_contract() {
        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
        let handler = tokio::spawn(async move {
            run_test(async {}, stop_rx).await;
        });
        let hex_string = "0x8aab161e2a1e57367b60bd870861e3042c2513f8a856f9fee014e7b96e0a2a36";
        // Remove the "0x" prefix
        let hex_string = &hex_string[2..];
        let mut private_key_result = [0; 32];
        // Parse the hexadecimal string into a Vec<u8>
        let bytes = hex::decode(hex_string).expect("Failed to decode hex string");
        private_key_result.copy_from_slice(&bytes);
        let result =
            ArbitrumClient::new("dev", "ws://127.0.0.1:8548", Some(private_key_result)).await;
        assert!(result.is_ok(), "Error creating ArbitrumClient");

        let client = result.unwrap();

        let faucet = 100 * u128::pow(10, client.config().currency_decimals);
        let wallet = Wallet::from_config(
            client.config().clone(),
            "ws://127.0.0.1:8548",
            None,
            Some(private_key_result),
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
        stop_tx.send(()).expect("Failed to send stop signal");
        handler.await.expect("Failed to join the background task");
    }

    #[allow(clippy::needless_raw_string_hashes)]
    #[tokio::test]
    #[sequential]
    async fn test_smart_contract_view() {
        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
        let handler = tokio::spawn(async move {
            run_test(async {}, stop_rx).await;
        });
        let hex_string = "0x8aab161e2a1e57367b60bd870861e3042c2513f8a856f9fee014e7b96e0a2a36";
        // Remove the "0x" prefix
        let hex_string = &hex_string[2..];
        let mut private_key_result = [0; 32];
        // Parse the hexadecimal string into a Vec<u8>
        let bytes = hex::decode(hex_string).expect("Failed to decode hex string");
        private_key_result.copy_from_slice(&bytes);
        let result =
            ArbitrumClient::new("dev", "ws://127.0.0.1:8548", Some(private_key_result)).await;
        assert!(result.is_ok(), "Error creating ArbitrumClient");
        let client = result.unwrap();
        let faucet = 100 * u128::pow(10, client.config().currency_decimals);
        let wallet = Wallet::from_config(
            client.config().clone(),
            "ws://127.0.0.1:8548",
            None,
            Some(private_key_result),
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
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 1
                ]
                .to_vec()
            )
        );
        stop_tx.send(()).expect("Failed to send stop signal");
        handler.await.expect("Failed to join the background task");
    }
}
