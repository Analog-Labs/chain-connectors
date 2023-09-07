pub use rosetta_core::*;

#[cfg(feature = "ws")]
pub mod ws;

#[cfg(feature = "tests")]
pub mod tests {
    use super::*;
    use crate::types::PartialBlockIdentifier;
    use anyhow::Result;
    use nanoid::nanoid;
    use rosetta_docker::Env;
    use std::future::Future;

    fn env_id() -> String {
        nanoid!(
            10,
            &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',]
        )
    }

    pub async fn network_status<T, Fut, F>(
        start_connector: F,
        config: BlockchainConfig,
    ) -> Result<()>
    where
        T: BlockchainClient,
        Fut: Future<Output = Result<T>> + Send,
        F: FnMut(BlockchainConfig) -> Fut,
    {
        let env_id = env_id();
        let env = Env::new(
            &format!("{env_id}-network-status"),
            config.clone(),
            start_connector,
        )
        .await?;

        let client = env.node();

        // Check if the genesis is consistent
        let expected_genesis = client.genesis_block().clone();
        let actual_genesis = client
            .block(&PartialBlockIdentifier {
                index: Some(0),
                hash: None,
            })
            .await?
            .block_identifier;
        assert_eq!(expected_genesis, actual_genesis);

        // Check if the current block is consistent
        let expected_current = client.current_block().await?;
        let actual_current = client
            .block(&PartialBlockIdentifier {
                index: None,
                hash: Some(expected_current.hash.clone()),
            })
            .await?
            .block_identifier;
        assert_eq!(expected_current, actual_current);

        // Check if the finalized block is consistent
        let expected_finalized = client.finalized_block().await?;
        let actual_finalized = client
            .block(&PartialBlockIdentifier {
                index: None,
                hash: Some(expected_finalized.hash.clone()),
            })
            .await?
            .block_identifier;
        assert_eq!(expected_finalized, actual_finalized);

        env.shutdown().await?;
        Ok(())
    }

    pub async fn account<T, Fut, F>(start_connector: F, config: BlockchainConfig) -> Result<()>
    where
        T: BlockchainClient,
        Fut: Future<Output = Result<T>> + Send,
        F: FnMut(BlockchainConfig) -> Fut,
    {
        let env_id = env_id();
        let env = Env::new(
            &format!("{env_id}-account"),
            config.clone(),
            start_connector,
        )
        .await?;

        let value = 100 * u128::pow(10, config.currency_decimals);
        let wallet = env.ephemeral_wallet()?;
        wallet.faucet(value).await?;
        let amount = wallet.balance().await?;
        assert_eq!(amount.value, value.to_string());
        assert_eq!(amount.currency, config.currency());
        assert!(amount.metadata.is_none());

        env.shutdown().await?;
        Ok(())
    }

    pub async fn construction<T, Fut, F>(start_connector: F, config: BlockchainConfig) -> Result<()>
    where
        T: BlockchainClient,
        Fut: Future<Output = Result<T>> + Send,
        F: FnMut(BlockchainConfig) -> Fut,
    {
        let env_id = env_id();
        let env = Env::new(
            &format!("{env_id}-construction"),
            config.clone(),
            start_connector,
        )
        .await?;

        let faucet = 100 * u128::pow(10, config.currency_decimals);
        let value = u128::pow(10, config.currency_decimals);
        let alice = env.ephemeral_wallet()?;
        alice.faucet(faucet).await?;

        let bob = env.ephemeral_wallet()?;
        alice.transfer(bob.account(), value).await?;
        let amount = bob.balance().await?;
        assert_eq!(amount.value, value.to_string());

        env.shutdown().await?;
        Ok(())
    }
}
