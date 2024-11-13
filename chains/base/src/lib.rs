#![allow(clippy::large_futures)]

//! # Base Testnet Rosetta Server
//!
//! This module contains the production test for an Base Rosetta server implementation
//! specifically designed for interacting with the Base Nitro Testnet. The code includes
//! tests for network status, account management, and smart contract interaction.
//!
//! ## Features
//!
//! - Network status tests to ensure proper connection and consistency with the Base Nitro Testnet.
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
//! - `rosetta_server_base`: Custom client implementation for interacting with Base.
//! - `sha3`: SHA-3 (Keccak) implementation for hashing.
//! - `tokio`: Asynchronous runtime for running async functions.
//!
//! ## Usage
//!
//! To run the tests, execute the following command:
//!
//! ```sh
//! cargo test --package rosetta-testing-base --lib -- tests --nocapture
//! ```
//!
//! Note: The code assumes a local Base Nitro Testnet node running on `ws://127.0.0.1:8548` and
//! a local Ethereum node on `http://localhost:8545`. Ensure that these endpoints are configured correctly.

#[allow(clippy::ignored_unit_patterns, clippy::pub_underscore_fields)]
#[cfg(test)]
mod tests {
    use alloy_sol_types::{sol, SolCall};
    use anyhow::Result;
    use ethers::types::H256;
    use ethers_solc::{artifacts::Source, CompilerInput, EvmVersion, Solc};
    use hex_literal::hex;
    use rosetta_chain_testing::run_test;
    use rosetta_client::Wallet;
    use rosetta_config_ethereum::{AtBlock, CallResult};
    use rosetta_core::BlockchainClient;
    use rosetta_server_ethereum::MaybeWsEthereumClient;
    use serial_test::serial;
    use sha3::Digest;
    use std::{collections::BTreeMap, path::Path};

    /// Account used to fund other testing accounts.
    const FUNDING_ACCOUNT_PRIVATE_KEY: [u8; 32] =
        hex!("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");

    /// Base rpc url
    const BASE_RPC_WS_URL: &str = "http://127.0.0.1:8545";

    sol! {
        interface TestContract {
            event AnEvent();
            function emitEvent() external;

            function identity(bool a) external view returns (bool);
        }
    }

    #[tokio::test]
    #[serial]
    async fn network_status() {
        run_test(async move {
            let client = MaybeWsEthereumClient::new("base", "dev", BASE_RPC_WS_URL, None)
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
    #[serial]
    async fn test_account() {
        run_test(async move {
            let client = MaybeWsEthereumClient::new(
                "base",
                "dev",
                BASE_RPC_WS_URL,
                Some(FUNDING_ACCOUNT_PRIVATE_KEY),
            )
            .await
            .expect("Error creating BaseClient");
            let wallet = Wallet::from_config(
                client.config().clone(),
                BASE_RPC_WS_URL,
                None,
                Some(FUNDING_ACCOUNT_PRIVATE_KEY),
            )
            .await
            .unwrap();
            let value = 10 * u128::pow(10, client.config().currency_decimals);
            let _ = wallet.faucet(value, Some(25_000_000_000)).await;
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
    #[serial]
    async fn test_smart_contract() {
        run_test(async move {
            let client = MaybeWsEthereumClient::new(
                "base",
                "dev",
                BASE_RPC_WS_URL,
                Some(FUNDING_ACCOUNT_PRIVATE_KEY),
            )
            .await
            .expect("Error creating BaseClient");
            let faucet = 10 * u128::pow(10, client.config().currency_decimals);
            let wallet = Wallet::from_config(
                client.config().clone(),
                BASE_RPC_WS_URL,
                None,
                Some(FUNDING_ACCOUNT_PRIVATE_KEY),
            )
            .await
            .unwrap();
            wallet.faucet(faucet, Some(50_000_000_000)).await.unwrap();

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
    #[serial]
    async fn test_smart_contract_view() {
        run_test(async move {
            let client = MaybeWsEthereumClient::new(
                "base",
                "dev",
                BASE_RPC_WS_URL,
                Some(FUNDING_ACCOUNT_PRIVATE_KEY),
            )
            .await
            .expect("Error creating BaseClient");
            let faucet = 10 * u128::pow(10, client.config().currency_decimals);
            let wallet = Wallet::from_config(
                client.config().clone(),
                BASE_RPC_WS_URL,
                None,
                Some(FUNDING_ACCOUNT_PRIVATE_KEY),
            )
            .await
            .unwrap();
            wallet.faucet(faucet, Some(25_000_000_000)).await.unwrap();
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
