use anyhow::{Context, Result};
use parity_scale_codec::{Decode, Encode};
use rosetta_config_polkadot::metadata::dev as polkadot_metadata;
pub use rosetta_config_polkadot::{PolkadotMetadata, PolkadotMetadataParams};
use rosetta_core::{
    crypto::{address::Address, PublicKey},
    types::{
        Block, BlockIdentifier, CallRequest, Coin, PartialBlockIdentifier, Transaction,
        TransactionIdentifier,
    },
    BlockchainClient, BlockchainConfig, EmptyEventStream,
};
use rosetta_server::ws::default_client;
use serde_json::Value;
use sp_keyring::AccountKeyring;
use std::time::Duration;
use subxt::config::{Hasher, Header};
use subxt::{
    dynamic::Value as SubtxValue,
    rpc::types::BlockNumber,
    tx::{PairSigner, SubmittableExtrinsic},
    utils::{AccountId32, MultiAddress},
    Config, OnlineClient, PolkadotConfig,
};

mod block;
mod call;

pub struct PolkadotClient {
    config: BlockchainConfig,
    client: OnlineClient<PolkadotConfig>,
    genesis_block: BlockIdentifier,
}

impl PolkadotClient {
    pub async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_polkadot::config(network)?;
        Self::from_config(config, addr).await
    }

    pub async fn from_config(config: BlockchainConfig, addr: &str) -> Result<Self> {
        let client = {
            let ws_client = default_client(addr, None).await?;
            OnlineClient::<PolkadotConfig>::from_rpc_client(std::sync::Arc::new(ws_client)).await?
        };
        let genesis = client.genesis_hash();
        let genesis_block = BlockIdentifier {
            index: 0,
            hash: hex::encode(genesis.as_ref()),
        };
        Ok(Self {
            config,
            client,
            genesis_block,
        })
    }

    async fn account_info(
        &self,
        address: &Address,
        maybe_block: Option<&BlockIdentifier>,
    ) -> Result<AccountInfo<u32, AccountData>> {
        let account: AccountId32 = address
            .address()
            .parse()
            .map_err(|err| anyhow::anyhow!("{}", err))
            .context("invalid address")?;

        // Build a dynamic storage query to iterate account information.
        let storage_query =
            subxt::dynamic::storage("System", "Account", vec![SubtxValue::from_bytes(account)]);

        let block_hash = {
            let block_number = maybe_block.map(|block| BlockNumber::from(block.index));
            self.client
                .rpc()
                .block_hash(block_number)
                .await?
                .ok_or_else(|| anyhow::anyhow!("no block hash found"))?
        };

        let account_info = self
            .client
            .storage()
            .at(block_hash)
            .fetch(&storage_query)
            .await?
            .ok_or_else(|| anyhow::anyhow!("account not found"))?;

        AccountInfo::decode(&mut account_info.encoded())
            .map_err(|_| anyhow::anyhow!("invalid format"))
    }
}

#[async_trait::async_trait]
impl BlockchainClient for PolkadotClient {
    type MetadataParams = PolkadotMetadataParams;
    type Metadata = PolkadotMetadata;
    type EventStream<'a> = EmptyEventStream;

    fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    fn genesis_block(&self) -> &BlockIdentifier {
        &self.genesis_block
    }

    async fn node_version(&self) -> Result<String> {
        Ok(self.client.rpc().system_version().await?)
    }

    async fn current_block(&self) -> Result<BlockIdentifier> {
        let block = self
            .client
            .rpc()
            .block(None)
            .await?
            .context("no current block")?;
        let index = block.block.header.number as _;
        let hash = block.block.header.hash();
        Ok(BlockIdentifier {
            index,
            hash: hex::encode(hash.as_ref()),
        })
    }

    async fn finalized_block(&self) -> Result<BlockIdentifier> {
        let finalized_head = self.client.rpc().finalized_head().await?;
        let block = self
            .client
            .rpc()
            .block(Some(finalized_head))
            .await?
            .context("no finalized block")?;
        let index = block.block.header.number as _;
        let hash = block.block.header.hash();
        Ok(BlockIdentifier {
            index,
            hash: hex::encode(hash.as_ref()),
        })
    }

    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128> {
        let account_info = self.account_info(address, Some(block)).await?;
        Ok(account_info.data.free)
    }

    async fn coins(&self, _address: &Address, _block: &BlockIdentifier) -> Result<Vec<Coin>> {
        anyhow::bail!("not a utxo chain")
    }

    async fn faucet(&self, address: &Address, value: u128) -> Result<Vec<u8>> {
        let address: AccountId32 = address
            .address()
            .parse()
            .map_err(|err| anyhow::anyhow!("{}", err))
            .context("invalid address")?;
        let signer = PairSigner::<PolkadotConfig, _>::new(AccountKeyring::Alice.pair());

        let tx = polkadot_metadata::tx()
            .balances()
            .transfer(address.into(), value);

        let hash = self
            .client
            .tx()
            .sign_and_submit_then_watch_default(&tx, &signer)
            .await?
            .wait_for_finalized_success()
            .await?
            .extrinsic_hash();
        Ok(hash.0.to_vec())
    }

    async fn metadata(
        &self,
        public_key: &PublicKey,
        params: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        let address = public_key.to_address(self.config().address_format);
        let account_info = self.account_info(&address, None).await?;
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
        let hash = SubmittableExtrinsic::from_bytes(self.client.clone(), transaction.to_vec())
            .submit_and_watch()
            .await?
            .wait_for_finalized_success()
            .await?
            .extrinsic_hash();
        Ok(hash.0.to_vec())
    }

    async fn block(&self, block_identifier: &PartialBlockIdentifier) -> Result<Block> {
        let block_hash = if let Some(hash) = block_identifier.hash.as_ref() {
            hash.parse()?
        } else {
            self.client
                .rpc()
                .block_hash(block_identifier.index.map(BlockNumber::from))
                .await?
                .context("block not found")?
        };

        let block = self.client.blocks().at(block_hash).await?;

        // Build timestamp query
        let timestamp_now_query = polkadot_metadata::storage().timestamp().now();

        let timestamp = block
            .storage()
            .fetch_or_default(&timestamp_now_query)
            .await?;
        let body = block.body().await?;
        let mut transactions = vec![];
        for extrinsic in body.extrinsics().iter().filter_map(Result::ok) {
            let transaction = crate::block::get_transaction(self.config(), &extrinsic).await?;
            transactions.push(transaction);
        }
        Ok(Block {
            block_identifier: BlockIdentifier {
                index: block.number() as _,
                hash: hex::encode(block.hash()),
            },
            parent_block_identifier: BlockIdentifier {
                index: block.number().saturating_sub(1) as _,
                hash: hex::encode(block.header().parent_hash),
            },
            timestamp: Duration::from_millis(timestamp).as_nanos() as i64,
            transactions,
            metadata: None,
        })
    }

    async fn block_transaction(
        &self,
        block_identifier: &BlockIdentifier,
        transaction_identifier: &TransactionIdentifier,
    ) -> Result<Transaction> {
        let block_hash = block_identifier.hash.parse()?;
        let transaction_hash = transaction_identifier.hash.parse()?;
        let body = self.client.blocks().at(block_hash).await?.body().await?;
        let extrinsic = body
            .extrinsics()
            .iter()
            .filter_map(Result::ok)
            .find(|extrinsic| {
                <PolkadotConfig as Config>::Hasher::hash_of(&extrinsic.bytes()) == transaction_hash
            })
            .context("transaction not found")?;
        crate::block::get_transaction(self.config(), &extrinsic).await
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
            "constant" => crate::call::dynamic_constant_req(&self.client, pallet_name, call_name),
            "storage" => {
                crate::call::dynamic_storage_req(
                    &self.client,
                    pallet_name,
                    call_name,
                    request.parameters.clone(),
                )
                .await
            }
            _ => {
                anyhow::bail!("invalid query type");
            }
        }
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
    pub misc_frozen: u128,
    pub fee_frozen: u128,
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
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_docker::tests::network_status::<PolkadotClient, _, _>(client_from_config, config)
            .await
    }

    #[tokio::test]
    async fn test_account() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_docker::tests::account::<PolkadotClient, _, _>(client_from_config, config).await
    }

    #[tokio::test]
    async fn test_construction() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_docker::tests::construction::<PolkadotClient, _, _>(client_from_config, config)
            .await
    }
}
