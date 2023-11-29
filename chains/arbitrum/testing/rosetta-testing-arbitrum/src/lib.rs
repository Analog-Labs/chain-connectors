use anyhow::Result;
use rosetta_client::Wallet;
use rosetta_config_ethereum::{EthereumMetadata, EthereumMetadataParams};
use rosetta_core::{BlockchainClient, BlockchainConfig};
use rosetta_server_ethereum::MaybeWsEthereumClient;
use rosetta_client::crypto::{
    address::Address as crypto_Address, bip32::DerivedSecretKey, bip44::ChildNumber,
};
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

// Define a custom error type for your application
#[derive(Debug)]
pub struct ArbitrumEnvError {
    message: String,
}

impl ArbitrumEnvError {
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

/// All settings necessary to configure an arbitrum testnet
pub struct Config {
    /// Base Directory for store temporary files (chain config files, docker volumes, etc).
    pub base_directory: PathBuf,

    /// Port where the L2 Arbitrum node is listening, if none pick a random port
    pub arbitrum_port: Option<u16>,

    /// Port where the L1 ethereum node is listening, if none pick a random port
    pub ethereum_port: Option<u16>,
    // /// unlocked accounts which will receive funds.
    // pub main_account: Vec<(ethereum_types::Address, ethereum_types::U256)>,

    // ...
}

#[derive(Debug)]
pub struct ArbitrumEnv {
    _start: u8,
}

impl ArbitrumEnv {
    /// Starts a new arbitrum testnet
    pub async fn new() -> Result<Self, ArbitrumEnvError> {
        // You can start your Bash script here

        //when running the test , get the folder path, recive as perms
        let script_path = "/home/daino/nitro-testnode/test-node.bash"; // Replace with the actual path to binary
        let output = Command::new(script_path)
            .arg("--detach")
            .output()
            .map_err(|e| ArbitrumEnvError::new(&format!("Failed to start Bash script: {}", e)))?;
        println!("Standard Output:\n{}", String::from_utf8_lossy(&output.stdout));
        println!("Standard Error:\n{}", String::from_utf8_lossy(&output.stderr));

        //Check output status if status is success means chain is up
        if output.status.success() {
            // Your implementation here
            Ok(ArbitrumEnv { _start: 1 })
        } else {
            Err(ArbitrumEnvError::new("failed to run nitro-testnode"))
        }
    }

    /// Stop the arbitrum testnet and cleanup dependencies
    /// ex: stop docker containers, delete temporary files, etc
    pub async fn cleanup() -> Result<(), ArbitrumEnvError> {
        let output = Command::new("sh")
            .arg("-c")
            .arg("docker ps -a -q -f name=nitro-testnode | xargs docker rm -fv")
            .output()
            .map_err(|e| {
                ArbitrumEnvError::new(&format!("Failed to run docker-compose command: {}", e))
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

#[cfg(test)]
mod tests {
    use ethers::{
        providers::{Http, Middleware, Provider},
        types::{Address, TransactionRequest, H160},
        utils::hex,
    };
    use ethers_solc::{artifacts::Source, CompilerInput, EvmVersion, Solc};
    use rosetta_core::types::PartialBlockIdentifier;
    use sha3::Digest;
    use std::{collections::BTreeMap, fmt::format, path::Path, str::FromStr};

    use super::*;

    #[tokio::test]
    async fn start_new() {
        match ArbitrumEnv::new().await {
            Ok(arbitrum_env) => {
                tracing::info!("Arbitrum chain is up {:?}", arbitrum_env);
            },
            Err(arbitrum_env_error) => {
                tracing::error!("Error: {:?}", arbitrum_env_error);
            },
        }
    }

    #[tokio::test]
    async fn cleanup_success() {
        // Assuming cleanup is successful
        let result = ArbitrumEnv::cleanup().await;
        assert!(result.is_ok(), "Cleanup failed: {:?}", result);
    }

    #[tokio::test]
    async fn cleanup_failure() {
        // Assuming cleanup fails
        let result = ArbitrumEnv::cleanup().await;
        assert!(result.is_err(), "Cleanup should have failed: {:?}", result);
    }

    #[tokio::test]
    async fn network_status() {
        match MaybeWsEthereumClient::new("arbitrum", "dev", "ws://127.0.0.1:8548").await {
            Ok(client) => {
                // The client was successfully created, continue with the rest of the function
                // ...
                println!("Client created successfully");
                // Check if the genesis is consistent
                let expected_genesis = client.genesis_block().clone();
                println!("expected_genesis=> {:?}", expected_genesis);
                let actual_genesis = client
                    .block(&PartialBlockIdentifier { index: Some(0), hash: None })
                    .await
                    .unwrap()
                    .block_identifier;

                println!("actual_genesis=> {:?}", expected_genesis);
                assert_eq!(expected_genesis, actual_genesis);
                // Check if the current block is consistent
                let expected_current = client.current_block().await.unwrap();
                let actual_current = client
                    .block(&PartialBlockIdentifier {
                        index: None,
                        hash: Some(expected_current.hash.clone()),
                    })
                    .await;
                match actual_current {
                    Ok(block) => {
                        println!("actual_current=> {:?}", block.block_identifier);
                        assert_eq!(expected_current, block.block_identifier);
                    },
                    Err(error) => {
                        println!("{:?}", error);
                    },
                }

                // Check if the finalized block is consistent
                let expected_finalized = client.finalized_block().await.unwrap();
                let actual_finalized = client
                    .block(&PartialBlockIdentifier {
                        index: None,
                        hash: Some(expected_finalized.hash.clone()),
                    })
                    .await;

                match actual_finalized {
                    Ok(block) => {
                        println!("actual_finalized=> {:?}", block.block_identifier);
                        assert_eq!(expected_finalized, block.block_identifier);
                    },
                    Err(error) => {
                        println!("ad{:?}", error);
                    },
                }

                tracing::info!("Arbitrum network is up and running");
            },
            Err(err) => {
                // An error occurred while creating the client, handle the error here
                eprintln!("Error creating client: {:?}", err);
            },
        }
    }

    async fn faucet_test(address: &str, value: u128) {
        let script_path = "/home/daino/nitro-testnode/test-node.bash";
        let to_address = format!("address_{address}");
        let value = &value.to_string();
        // The arguments for the script
        let args = vec!["script", "send-l2", "--to", &to_address, "--ethamount", value];

        // Execute the command
        let output = Command::new(script_path)
            .args(args)
            .output()
            .expect("Failed to execute command");

        // Print the output
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

        // Check the exit status
        if output.status.success() {
            println!("Command executed successfully");
        } else {
            println!("Command failed with exit code: {}", output.status);
        }
    }

    #[tokio::test]
    async fn test_account() {
        let result = MaybeWsEthereumClient::new("arbitrum", "dev", "ws://127.0.0.1:8548").await;
        assert!(result.is_ok(), "Error creating MaybeWsEthereumClient");
        let client = result.unwrap();

        let value = 2;
        let wallet =
            Wallet::from_config(client.config().clone(), "ws://127.0.0.1:8548", None).await;
        match wallet {
            Ok(w) => {
                let address = crypto_Address::new(
                    client.config().address_format,
                    w.account().address.clone(),
                );
                let _ = self::faucet_test(address.address(), value).await;
                let amount = w.balance().await.unwrap();

                assert_eq!(
                    (amount.value),
                    (value * u128::pow(10, client.config().currency_decimals)).to_string()
                );
                assert_eq!(amount.currency, client.config().currency());
                assert!(amount.metadata.is_none());
            },
            Err(e) => {
                println!("Error : {:?}", e);
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

    #[tokio::test]
    async fn test_smart_contract() -> Result<()> {
        let result = MaybeWsEthereumClient::new("arbitrum", "dev", "ws://127.0.0.1:8548").await;
        assert!(result.is_ok(), "Error creating MaybeWsEthereumClient");

        let client = result.unwrap();

        let faucet = 10;
        let wallet = Wallet::from_config(client.config().clone(), "ws://127.0.0.1:8548", None)
            .await
            .unwrap();
        let address =
            crypto_Address::new(client.config().address_format, wallet.account().address.clone());
        let _ = self::faucet_test(address.address(), faucet).await;

        // wallet.faucet(faucet).await?;

        let bytes = compile_snippet(
            r#"
            event AnEvent();
            function emitEvent() public {
                emit AnEvent();
            }
        "#,
        )?;
        let tx_hash = wallet.eth_deploy_contract(bytes).await?;
        let receipt = wallet.eth_transaction_receipt(&tx_hash).await?;
        let contract_address =
            receipt.get("contractAddress").and_then(serde_json::Value::as_str).unwrap();
        let tx_hash =
            wallet.eth_send_call(contract_address, "function emitEvent()", &[], 0).await?;
        let receipt = wallet.eth_transaction_receipt(&tx_hash).await?;
        let logs = receipt.get("logs").and_then(serde_json::Value::as_array).unwrap();
        assert_eq!(logs.len(), 1);
        let topic = logs[0]["topics"][0].as_str().unwrap();
        let expected = format!("0x{}", hex::encode(sha3::Keccak256::digest("AnEvent()")));
        assert_eq!(topic, expected);

        Ok(())
    }

    #[tokio::test]
    async fn test_smart_contract_view() -> Result<()> {
        let result = MaybeWsEthereumClient::new("arbitrum", "dev", "ws://127.0.0.1:8548").await;
        assert!(result.is_ok(), "Error creating MaybeWsEthereumClient");

        let client = result.unwrap();

        let faucet = 1;
        let wallet = Wallet::from_config(client.config().clone(), "ws://127.0.0.1:8548", None)
            .await
            .unwrap();
        // wallet.faucet(faucet).await?;

        let address =
            crypto_Address::new(client.config().address_format, wallet.account().address.clone());
        let _ = self::faucet_test(address.address(), faucet).await;

        let bytes = compile_snippet(
            r#"
            function identity(bool a) public view returns (bool) {
                return a;
            }
        "#,
        )?;
        let tx_hash = wallet.eth_deploy_contract(bytes).await?;
        let receipt = wallet.eth_transaction_receipt(&tx_hash).await?;
        let contract_address = receipt["contractAddress"].as_str().unwrap();

        let response = wallet
            .eth_view_call(
                contract_address,
                "function identity(bool a) returns (bool)",
                &["true".into()],
                None,
            )
            .await?;
        let result: Vec<String> = serde_json::from_value(response)?;
        assert_eq!(result[0], "true");
        Ok(())
    }
}
