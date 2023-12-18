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

#[allow(clippy::used_underscore_binding)]
#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_types::{sol, SolCall};
    use ethers::{
        providers::{Http, Middleware, Provider},
        signers::{LocalWallet, Signer},
        types::{
            transaction::eip2718::TypedTransaction, Bytes, TransactionRequest, H160, H256, U256,
            U64,
        },
    };
    use ethers_solc::{artifacts::Source, CompilerInput, EvmVersion, Solc};
    use rosetta_client::Wallet;
    use rosetta_config_ethereum::{AtBlock, CallResult};
    use rosetta_core::{types::PartialBlockIdentifier, BlockchainClient};
    use rosetta_server_arbitrum::ArbitrumClient;
    use sha3::Digest;
    use std::{collections::BTreeMap, path::Path, str::FromStr, thread, time::Duration};
    use url::Url;

    sol! {
        interface TestContract {
            event AnEvent();
            function emitEvent() external;

            function identity(bool a) external view returns (bool);
        }
    }

    //Test for start the arbitrum default node (nitro-testnode)
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

    //must run this test before running below tests.
    #[tokio::test]
    pub async fn for_incress_blocknumber() -> Result<()> {
        let rpc_url_str = "http://localhost:8547";
        let rpc_url = Url::parse(rpc_url_str).expect("Invalid URL");
        let http = Http::new(rpc_url);
        let provider = Provider::<Http>::new(http);
        let chain_id = provider.get_chainid().await?;
        let private_key = "0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659";
        let result = ArbitrumClient::new("dev", "ws://127.0.0.1:8548").await;
        assert!(result.is_ok(), "Error creating ArbitrumClient");
        let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id.as_u64());
        loop {
            let nonce = provider
                .get_transaction_count(
                    ethers::types::NameOrAddress::Address(
                        H160::from_str("0x3f1eae7d46d88f08fc2f8ed27fcb2ab183eb2d0e").unwrap(),
                    ),
                    None,
                )
                .await
                .unwrap(); //public key of faucet account
                           // Create a transaction request
            let transaction_request = TransactionRequest {
                from: None,
                to: Some(ethers::types::NameOrAddress::Address(
                    H160::from_str("0xc109c36fd5d730d7f9a14dB2597B2d9eDd991719").unwrap(),
                )),
                value: Some(U256::from(1_000_000_000)), // Specify the amount you want to send
                gas: Some(U256::from(210_000)),         // Adjust gas values accordingly
                gas_price: Some(U256::from(500_000_000)), // Adjust gas price accordingly
                nonce: Some(nonce),                     // Nonce will be automatically determined
                data: None,
                chain_id: Some(U64::from(412_346)), // Replace with your desired chain ID
            };
            let tx: TypedTransaction = transaction_request.into();
            let signature = wallet.sign_transaction(&tx).await.unwrap();
            let tx: Bytes = tx.rlp_signed(&signature);
            let _ = provider.send_raw_transaction(tx).await;
            thread::sleep(Duration::from_secs(1));
        }
    }

    #[tokio::test]
    async fn network_status() {
        match ArbitrumClient::new("dev", "ws://127.0.0.1:8548").await {
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
    async fn test_account() {
        let result = ArbitrumClient::new("dev", "ws://127.0.0.1:8548").await;
        assert!(result.is_ok(), "Error creating ArbitrumClient");
        let client = result.unwrap();

        let value = 100 * u128::pow(10, client.config().currency_decimals);
        let wallet =
            Wallet::from_config(client.config().clone(), "ws://127.0.0.1:8548", None).await;
        match wallet {
            Ok(w) => {
                let _ = w
                    .faucet(
                        value,
                        Some("0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659"),
                    )
                    .await;

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

    #[tokio::test]
    #[allow(clippy::needless_raw_string_hashes)]
    async fn test_smart_contract() -> Result<()> {
        let result = ArbitrumClient::new("dev", "ws://127.0.0.1:8548").await;
        assert!(result.is_ok(), "Error creating ArbitrumClient");

        let client = result.unwrap();

        let faucet = 100 * u128::pow(10, client.config().currency_decimals);
        let wallet = Wallet::from_config(client.config().clone(), "ws://127.0.0.1:8548", None)
            .await
            .unwrap();
        wallet
            .faucet(
                faucet,
                Some("0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659"),
            )
            .await?;

        let bytes = compile_snippet(
            r"
    event AnEvent();
    function emitEvent() public {
        emit AnEvent();
    }
    ",
        )?;
        let tx_hash = wallet.eth_deploy_contract(bytes).await?;
        let receipt = wallet.eth_transaction_receipt(tx_hash).await?.unwrap();
        let contract_address = receipt.contract_address.unwrap();
        let tx_hash = {
            let call = TestContract::emitEventCall {};
            wallet.eth_send_call(contract_address.0, call.abi_encode(), 0).await?
        };
        let receipt = wallet.eth_transaction_receipt(tx_hash).await?.unwrap();
        assert_eq!(receipt.logs.len(), 1);
        let topic = receipt.logs[0].topics[0];
        let expected = H256(sha3::Keccak256::digest("AnEvent()").into());
        assert_eq!(topic, expected);

        Ok(())
    }

    #[tokio::test]
    #[allow(clippy::needless_raw_string_hashes)]
    async fn test_smart_contract_view() -> Result<()> {
        let result = ArbitrumClient::new("dev", "ws://127.0.0.1:8548").await;
        assert!(result.is_ok(), "Error creating ArbitrumClient");

        let client = result.unwrap();

        let faucet = 100 * u128::pow(10, client.config().currency_decimals);
        let wallet = Wallet::from_config(client.config().clone(), "ws://127.0.0.1:8548", None)
            .await
            .unwrap();
        wallet
            .faucet(
                faucet,
                Some("0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659"),
            )
            .await?;
        let bytes = compile_snippet(
            r"
            function identity(bool a) public view returns (bool) {
                return a;
            }
            ",
        )?;
        let tx_hash = wallet.eth_deploy_contract(bytes).await?;
        let receipt = wallet.eth_transaction_receipt(tx_hash).await?.unwrap();
        let contract_address = receipt.contract_address.unwrap();

        let response = {
            let call = TestContract::identityCall { a: true };
            wallet
                .eth_view_call(contract_address.0, call.abi_encode(), AtBlock::Latest)
                .await?
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
        Ok(())
    }
    #[tokio::test]
    async fn cleanup_success() {
        // Assuming cleanup is successful
        let result = ArbitrumEnv::cleanup().await;
        assert!(result.is_ok(), "Cleanup failed: {result:?}");
    }

    #[tokio::test]
    #[ignore]
    async fn cleanup_failure() {
        // Assuming cleanup fails
        let result = ArbitrumEnv::cleanup().await;
        assert!(result.is_err(), "Cleanup should have failed: {result:?}");
    }
}
