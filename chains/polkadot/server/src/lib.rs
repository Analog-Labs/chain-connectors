use anyhow::{Context, Result};
use chains::WestendDevConfig;
use parity_scale_codec::{Decode, Encode};
pub use rosetta_config_polkadot::{PolkadotMetadata, PolkadotMetadataParams};
use rosetta_core::{
    crypto::{address::Address, PublicKey},
    types::{BlockIdentifier, CallRequest, PartialBlockIdentifier},
    BlockchainClient, BlockchainConfig, EmptyEventStream,
};
use rosetta_server::ws::default_client;
use serde_json::Value;
use sp_keyring::AccountKeyring;
use subxt::{
    config::Header,
    tx::{PairSigner, SubmittableExtrinsic},
    utils::{AccountId32, MultiAddress},
};

mod call;
mod chains;
mod client;
mod types;

pub struct PolkadotClient {
    config: BlockchainConfig,
    client: client::SubstrateClient<chains::WestendDevConfig>,
    genesis_block: BlockIdentifier,
}

impl PolkadotClient {
    /// Creates a new polkadot client, loading the config from `network` and connects to `addr`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_polkadot::config(network)?;
        Self::from_config(config, addr).await
    }

    /// Creates a new substrate client using the provided `config` and connets to `addr`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn from_config(config: BlockchainConfig, addr: &str) -> Result<Self> {
        let ws_client = default_client(addr, None).await?;
        let client = client::SubstrateClient::<WestendDevConfig>::from_client(ws_client).await?;
        let genesis = client.genesis_hash();
        let genesis_block = BlockIdentifier { index: 0, hash: genesis.0 };
        Ok(Self { config, client, genesis_block })
    }
}

#[async_trait::async_trait]
impl BlockchainClient for PolkadotClient {
    type MetadataParams = PolkadotMetadataParams;
    type Metadata = PolkadotMetadata;
    type EventStream<'a> = EmptyEventStream<Self::BlockIdentifier, Self::Event>;
    type Call = CallRequest;
    type CallResult = Value;

    type AtBlock = PartialBlockIdentifier;
    type BlockIdentifier = BlockIdentifier;

    type Query = types::Query<chains::WestendDevConfig>;
    type Transaction = Vec<u8>;
    type Subscription = ();
    type Event = ();

    async fn query(
        &self,
        _query: Self::Query,
    ) -> Result<<Self::Query as rosetta_core::traits::Query>::Result> {
        anyhow::bail!("unsupported query");
    }

    fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    fn genesis_block(&self) -> BlockIdentifier {
        self.genesis_block.clone()
    }

    async fn current_block(&self) -> Result<BlockIdentifier> {
        let block = self
            .client
            .block_details(types::BlockIdentifier::<_>::Latest)
            .await?
            .context("no current block")?;
        // let block = self.rpc_methods.chain_get_block(None).await?.context("no current block")?;
        let index = u64::from(block.block.header.number);
        let hash = block.block.header.hash();
        Ok(BlockIdentifier { index, hash: hash.0 })
    }

    async fn finalized_block(&self) -> Result<BlockIdentifier> {
        let Some(block) = self.client.block_details(types::BlockIdentifier::<_>::Finalized).await?
        else {
            return Ok(self.genesis_block.clone());
        };
        let index = u64::from(block.block.header.number);
        let hash = block.block.header.hash();
        Ok(BlockIdentifier { index, hash: hash.0 })
    }

    async fn balance(
        &self,
        address: &Address,
        block_identifier: &PartialBlockIdentifier,
    ) -> Result<u128> {
        let account: AccountId32 = address
            .address()
            .parse()
            .map_err(|err| anyhow::anyhow!("{}", err))
            .context("invalid address")?;
        let account_info = self.client.account_info(account, block_identifier).await?;
        Ok(account_info.data.free)
    }

    async fn faucet(&self, address: &Address, value: u128) -> Result<Vec<u8>> {
        let address: AccountId32 = address
            .address()
            .parse()
            .map_err(|err| anyhow::anyhow!("{err}"))
            .context("invalid address")?;
        let signer = PairSigner::<_, _>::new(AccountKeyring::Alice.pair());
        let hash = self.client.faucet(signer, address.into(), value).await?;
        Ok(hash.0.to_vec())
    }

    async fn metadata(
        &self,
        public_key: &PublicKey,
        params: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        let address = public_key.to_address(self.config().address_format);

        let account: AccountId32 = address
            .address()
            .parse()
            .map_err(|err| anyhow::anyhow!("{err}"))
            .context("invalid address")?;

        let account_info =
            self.client.account_info(&account, types::BlockIdentifier::<_>::Latest).await?;
        let runtime = self.client.runtime_version();
        let metadata = self.client.metadata();
        let pallet = metadata
            .pallet_by_name(&params.pallet_name)
            .ok_or_else(|| anyhow::anyhow!("pallet not found"))?;
        let pallet_index = pallet.index();
        let call_variant = pallet
            .call_variant_by_name(&params.call_name)
            .ok_or_else(|| anyhow::anyhow!("call name not found"))?;
        let call_index = call_variant.index;
        let call_hash = pallet
            .call_hash(&params.call_name)
            .ok_or_else(|| anyhow::anyhow!("call hash not found"))?;
        let genesis_hash = self.client.genesis_hash().0;
        Ok(PolkadotMetadata {
            nonce: account_info.nonce,
            spec_version: runtime.spec_version,
            transaction_version: runtime.transaction_version,
            genesis_hash,
            pallet_index,
            call_index,
            call_hash,
        })
    }

    async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        let hash =
            SubmittableExtrinsic::from_bytes(self.client.client().clone(), transaction.to_vec())
                .submit_and_watch()
                .await?
                .wait_for_finalized_success()
                .await?
                .extrinsic_hash();
        Ok(hash.0.to_vec())
    }

    async fn call(&self, request: &CallRequest) -> Result<Value> {
        let call_details = request.method.split('-').collect::<Vec<&str>>();
        if call_details.len() != 3 {
            anyhow::bail!("Invalid length of call request params");
        }
        let pallet_name = call_details[0];
        let call_name = call_details[1];
        let query_type = call_details[2];
        match query_type.to_lowercase().as_str() {
            "constant" => {
                crate::call::dynamic_constant_req(self.client.client(), pallet_name, call_name)
            },
            "storage" => {
                crate::call::dynamic_storage_req(
                    self.client.client(),
                    pallet_name,
                    call_name,
                    request.parameters.clone(),
                )
                .await
            },
            _ => {
                anyhow::bail!("invalid query type");
            },
        }
    }

    fn subscribe(&self, _sub: &Self::Subscription) -> Result<u32> {
        anyhow::bail!("not implemented");
    }
}

#[derive(Decode, Encode, Debug)]
struct AccountInfo<Index, AccountData> {
    pub nonce: Index,
    pub consumers: Index,
    pub providers: Index,
    pub sufficients: Index,
    pub data: AccountData,
}

#[derive(Decode, Encode, Debug)]
struct AccountData {
    pub free: u128,
    pub reserved: u128,
    pub frozen: u128,
}

#[derive(Decode, Encode, Debug)]
pub struct Transfer {
    pub dest: MultiAddress<AccountId32, u32>,
    #[codec(compact)]
    pub value: u128,
}

#[cfg(test)]
mod tests {
    use super::*;

    pub async fn client_from_config(config: BlockchainConfig) -> Result<PolkadotClient> {
        let url = config.node_uri.to_string();
        PolkadotClient::from_config(config, url.as_str()).await
    }

    #[tokio::test]
    async fn test_network_status() -> Result<()> {
        let config = rosetta_config_polkadot::config("westend-dev")?;
        rosetta_docker::tests::network_status::<PolkadotClient, _, _>(client_from_config, config)
            .await
    }

    #[tokio::test]
    async fn test_account() -> Result<()> {
        let config = rosetta_config_polkadot::config("westend-dev")?;
        rosetta_docker::tests::account::<PolkadotClient, _, _>(client_from_config, config).await
    }

    #[tokio::test]
    async fn test_construction() -> Result<()> {
        let config = rosetta_config_polkadot::config("westend-dev")?;
        rosetta_docker::tests::construction::<PolkadotClient, _, _>(client_from_config, config)
            .await
    }
}
