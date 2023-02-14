use anyhow::{Context, Result};
use ethers::prelude::*;
use rosetta_config_ethereum::{EthereumMetadata, EthereumMetadataParams};
use rosetta_server::crypto::address::Address;
use rosetta_server::crypto::PublicKey;
use rosetta_server::types::{BlockIdentifier, Coin};
use rosetta_server::{BlockchainClient, BlockchainConfig};

pub struct EthereumClient {
    config: BlockchainConfig,
    client: Provider<Http>,
    genesis_block: BlockIdentifier,
}

#[async_trait::async_trait]
impl BlockchainClient for EthereumClient {
    type MetadataParams = EthereumMetadataParams;
    type Metadata = EthereumMetadata;

    async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_ethereum::config(network)?;
        let client = Provider::<Http>::try_from(format!("http://{}", addr))?;
        let genesis = client.get_block(0).await?.unwrap();
        let genesis_block = BlockIdentifier {
            index: 0,
            hash: hex::encode(genesis.hash.as_ref().unwrap()),
        };
        Ok(Self {
            config,
            client,
            genesis_block,
        })
    }

    fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    fn genesis_block(&self) -> &BlockIdentifier {
        &self.genesis_block
    }

    async fn node_version(&self) -> Result<String> {
        Ok(self.client.client_version().await?)
    }

    async fn current_block(&self) -> Result<BlockIdentifier> {
        let index = self.client.get_block_number().await?.as_u64();
        let block = self
            .client
            .get_block(index)
            .await?
            .context("missing block")?;
        Ok(BlockIdentifier {
            index,
            hash: hex::encode(block.hash.as_ref().unwrap()),
        })
    }

    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128> {
        let block = hex::decode(&block.hash)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid block hash"))?;
        let address: H160 = address.address().parse()?;
        Ok(self
            .client
            .get_balance(address, Some(BlockId::Hash(H256(block))))
            .await?
            .as_u128())
    }

    async fn coins(&self, _address: &Address, _block: &BlockIdentifier) -> Result<Vec<Coin>> {
        anyhow::bail!("not a utxo chain");
    }

    async fn faucet(&self, address: &Address, param: u128) -> Result<Vec<u8>> {
        // first account will be the coinbase account on a dev net
        let coinbase = self.client.get_accounts().await?[0];
        let address: H160 = address.address().parse()?;
        let tx = TransactionRequest::new()
            .to(address)
            .value(param)
            .from(coinbase);
        Ok(self
            .client
            .send_transaction(tx, None)
            .await?
            .await?
            .unwrap()
            .transaction_hash
            .0
            .to_vec())
    }

    async fn metadata(
        &self,
        public_key: &PublicKey,
        options: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        let from: H160 = public_key
            .to_address(self.config().address_format)
            .address()
            .parse()
            .unwrap();
        let to = H160::from_slice(&options.destination);
        let chain_id = self.client.get_chainid().await?;
        let nonce = self.client.get_transaction_count(from, None).await?;
        let (max_fee_per_gas, max_priority_fee_per_gas) =
            self.client.estimate_eip1559_fees(None).await?;
        let tx = Eip1559TransactionRequest::new()
            .from(from)
            .to(to)
            .value(U256(options.amount))
            .data(options.data.clone());
        let gas_limit = self.client.estimate_gas(&tx.into(), None).await?;
        Ok(EthereumMetadata {
            chain_id: chain_id.as_u64(),
            nonce: nonce.as_u64(),
            max_priority_fee_per_gas: max_priority_fee_per_gas.0,
            max_fee_per_gas: max_fee_per_gas.0,
            gas_limit: gas_limit.0,
        })
    }

    async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        let tx = transaction.to_vec().into();
        Ok(self
            .client
            .send_raw_transaction(Bytes(tx))
            .await?
            .await?
            .unwrap()
            .transaction_hash
            .0
            .to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_list() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::network_list(config).await
    }

    #[tokio::test]
    async fn test_network_options() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::network_options::<EthereumClient>(config).await
    }

    #[tokio::test]
    async fn test_network_status() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::network_status::<EthereumClient>(config).await
    }

    #[tokio::test]
    async fn test_account() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::account(config).await
    }

    #[tokio::test]
    async fn test_construction() -> Result<()> {
        let config = rosetta_config_ethereum::config("dev")?;
        rosetta_server::tests::construction(config).await
    }
}
