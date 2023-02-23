use anyhow::{Context, Result};
use parity_scale_codec::{Decode, Encode};
use rosetta_config_polkadot::{PolkadotMetadata, PolkadotMetadataParams};
use rosetta_server::crypto::address::Address;
use rosetta_server::crypto::PublicKey;
use rosetta_server::types::{
    Block, BlockIdentifier, CallRequest, Coin, PartialBlockIdentifier, Transaction,
    TransactionIdentifier,
};
use rosetta_server::{BlockchainClient, BlockchainConfig};
use serde_json::Value;
use sp_keyring::AccountKeyring;
use std::time::Duration;
use subxt::config::{Hasher, Header};
use subxt::metadata::DecodeStaticType;
use subxt::rpc::types::BlockNumber;
use subxt::storage::address::{StorageHasher, StorageMapKey, Yes};
use subxt::storage::StaticStorageAddress;
use subxt::tx::{PairSigner, StaticTxPayload, SubmittableExtrinsic};
use subxt::utils::{AccountId32, MultiAddress, H256};
use subxt::{Config, OnlineClient, PolkadotConfig};

mod block;
mod call;

pub struct PolkadotClient {
    config: BlockchainConfig,
    client: OnlineClient<PolkadotConfig>,
    genesis_block: BlockIdentifier,
}

impl PolkadotClient {
    async fn account_info(
        &self,
        address: &Address,
        block: Option<&BlockIdentifier>,
    ) -> Result<AccountInfo<u32, AccountData>> {
        let address: AccountId32 = address
            .address()
            .parse()
            .map_err(|err| anyhow::anyhow!("{}", err))
            .context("invalid address")?;
        let hash = self.client.metadata().storage_hash("System", "Account")?;
        let key = StaticStorageAddress::<
            DecodeStaticType<AccountInfo<u32, AccountData>>,
            Yes,
            Yes,
            Yes,
        >::new(
            "System",
            "Account",
            vec![StorageMapKey::new(
                &address,
                StorageHasher::Blake2_128Concat,
            )],
            hash,
        );

        let block = if let Some(block) = block {
            let block = hex::decode(&block.hash)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid block"))?;
            Some(H256(block))
        } else {
            None
        };
        let account_info = self
            .client
            .storage()
            .at(block)
            .await?
            .fetch_or_default(&key)
            .await?;
        Ok(account_info)
    }
}

#[async_trait::async_trait]
impl BlockchainClient for PolkadotClient {
    type MetadataParams = PolkadotMetadataParams;
    type Metadata = PolkadotMetadata;

    async fn new(network: &str, addr: &str) -> Result<Self> {
        let config = rosetta_config_polkadot::config(network)?;
        let client = OnlineClient::<PolkadotConfig>::from_url(format!("ws://{addr}")).await?;
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
        let dest: MultiAddress<AccountId32, u32> = MultiAddress::Id(address);
        let tx = StaticTxPayload::new("Balances", "transfer", Transfer { dest, value }, [0; 32])
            .unvalidated();
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
        let pallet = metadata.pallet(&params.pallet_name)?;
        let pallet_index = pallet.index();
        let call_index = pallet.call_index(&params.call_name)?;
        let call_hash = metadata.call_hash(&params.pallet_name, &params.call_name)?;
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
        let block = self.client.blocks().at(Some(block_hash)).await?;
        let timestamp_now_address =
            StaticStorageAddress::<DecodeStaticType<u64>, Yes, Yes, ()>::new(
                "Timestamp",
                "Now",
                vec![],
                [0; 32],
            )
            .unvalidated();
        let timestamp = block
            .storage()
            .fetch_or_default(&timestamp_now_address)
            .await?;
        let body = block.body().await?;
        let mut transactions = vec![];
        for extrinsic in body.extrinsics() {
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
        let body = self
            .client
            .blocks()
            .at(Some(block_hash))
            .await?
            .body()
            .await?;
        let extrinsic = body
            .extrinsics()
            .find(|extrinsic| {
                <PolkadotConfig as Config>::Hasher::hash_of(&extrinsic.bytes()) == transaction_hash
            })
            .context("transaction not found")?;
        crate::block::get_transaction(self.config(), &extrinsic).await
    }

    async fn call(&self, request: &CallRequest) -> Result<Value> {
        let call_details = request.method.split(',').collect::<Vec<&str>>();
        if call_details.len() != 3 {
            anyhow::bail!("invalid call request");
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

    #[tokio::test]
    async fn test_network_list() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_server::tests::network_list(config).await
    }

    #[tokio::test]
    async fn test_network_options() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_server::tests::network_options::<PolkadotClient>(config).await
    }

    #[tokio::test]
    async fn test_network_status() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_server::tests::network_status::<PolkadotClient>(config).await
    }

    #[tokio::test]
    async fn test_account() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_server::tests::account(config).await
    }

    #[tokio::test]
    async fn test_construction() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_server::tests::construction(config).await
    }

    #[tokio::test]
    async fn test_find_transaction() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_server::tests::find_transaction(config).await
    }

    #[tokio::test]
    async fn test_list_transactions() -> Result<()> {
        let config = rosetta_config_polkadot::config("dev")?;
        rosetta_server::tests::list_transactions(config).await
    }
}
