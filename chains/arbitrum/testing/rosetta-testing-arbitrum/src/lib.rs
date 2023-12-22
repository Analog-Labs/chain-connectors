use anyhow::Result;

use std::{error::Error, fmt, process::Command};

// Define a custom error type for your application
#[derive(Debug)]
pub struct ArbitrumEnvError {
    message: String,
}

impl ArbitrumEnvError {
    #[allow(clippy::use_self)]
    fn new(message: &str) -> ArbitrumEnvError {
        ArbitrumEnvError { message: message.to_string() }
    }
}

impl Error for ArbitrumEnvError {}

impl fmt::Display for ArbitrumEnvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

#[derive(Debug)]
pub struct ArbitrumEnv {}

impl ArbitrumEnv {
    /// Starts a new arbitrum testnet
    #[allow(clippy::use_self, clippy::missing_errors_doc, clippy::unused_async)]
    pub async fn new() -> Result<Self, ArbitrumEnvError> {
        // You can start your Bash script here

        //when running the test , get the folder path, recive as perms
        let script_path = "../nitro-testnode/test-node.bash"; // Replace with the actual path to binary
        let output = Command::new(script_path)
            .arg("--detach")
            .output()
            .map_err(|e| ArbitrumEnvError::new(&format!("Failed to start Bash script: {e}")))?;
        println!("Standard Output:\n{}", String::from_utf8_lossy(&output.stdout));
        println!("Standard Error:\n{}", String::from_utf8_lossy(&output.stderr));

        //Check output status if status is success means chain is up
        if output.status.success() {
            // Your implementation here
            Ok(ArbitrumEnv {})
        } else {
            Err(ArbitrumEnvError::new("failed to run nitro-testnode"))
        }
    }

    /// Stop the arbitrum testnet and cleanup dependencies
    /// ex: stop docker containers, delete temporary files, etc
    #[allow(clippy::missing_errors_doc, clippy::unused_async)]
    pub async fn cleanup() -> Result<(), ArbitrumEnvError> {
        let output = Command::new("sh")
            .arg("-c")
            .arg("docker ps -a -q -f name=nitro-testnode | xargs docker rm -fv")
            .output()
            .map_err(|e| {
                ArbitrumEnvError::new(&format!("Failed to run docker-compose command: {e}"))
            })?;

        if output.status.success() {
            tracing::info!("Docker Compose down successful!");
            Ok(())
        } else {
            let error_message = format!(
                "Docker Compose down failed! \nOutput: {:?} \nError: {:?}",
                output.stdout, output.stderr
            );
            tracing::error!("{}", error_message);
            Err(ArbitrumEnvError::new(&error_message))
        }
    }
}

#[allow(clippy::ignored_unit_patterns)]
#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_types::{sol, SolCall};
    use ethers::{types::H256, utils::hex};
    use ethers_solc::{artifacts::Source, CompilerInput, EvmVersion, Solc};
    use rosetta_client::Wallet;
    use rosetta_config_ethereum::{AtBlock, CallResult};
    use rosetta_core::{types::PartialBlockIdentifier, BlockchainClient};
    use rosetta_server_arbitrum::ArbitrumClient;
    use sequential_test::sequential;
    use sha3::Digest;
    use std::{collections::BTreeMap, path::Path, thread, time::Duration};

    sol! {
        interface TestContract {
            event AnEvent();
            function emitEvent() external;

            function identity(bool a) external view returns (bool);
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
                thread::sleep(Duration::from_secs(10));
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
    }

    #[allow(clippy::needless_raw_string_hashes)]
    #[tokio::test]
    #[sequential]
    async fn test_smart_contract_view() {
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
    }
}
