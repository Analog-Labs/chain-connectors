use anyhow::{Context, Result};
use parity_scale_codec::Decode;
use rosetta_core::{
    crypto::{
        address::{Address, AddressFormat, Ss58AddressFormat},
        PublicKey,
    },
    types::{
        Block, BlockIdentifier, Coin, PartialBlockIdentifier, Transaction, TransactionIdentifier,
    },
    BlockchainClient, BlockchainConfig,
};
use rosetta_config_humanode::metadata::{
    dev as humanode_metadata,
    dev::runtime_types::{frame_system::AccountInfo, pallet_balances::AccountData},
};
use rosetta_config_ethereum::{
    ethereum_types::H160, EthereumMetadata, EthereumMetadataParams, Query as EthQuery, QueryResult as EthQueryResult
};
use rosetta_server_ethereum::MaybeWsEthereumClient;
use rosetta_server::ws::default_client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use subxt::{
    backend::{
        legacy::{rpc_methods::BlockNumber, LegacyBackend, LegacyRpcMethods},
        rpc::RpcClient,
    }, dynamic::Value as SubtxValue, ext::sp_core, tx::PairSigner, utils::AccountId32, OnlineClient, PolkadotConfig
};


#[derive(Deserialize, Serialize)]
pub struct HumanodeMetadataParams(pub EthereumMetadataParams);

#[derive(Deserialize, Serialize)]
pub struct HumanodeMetadata(pub EthereumMetadata);

pub struct HumanodeClient {
    client: MaybeWsEthereumClient,
    ws_client: OnlineClient<PolkadotConfig>,
    rpc_methods: LegacyRpcMethods<PolkadotConfig>,
}

impl HumanodeClient {
    /// Creates a new polkadot client, loading the config from `network` and connects to `addr`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn new(network: &str, url: &str) -> Result<Self> {
        let config = rosetta_config_humanode::config(network)?;
        Self::from_config(config, url).await
    }

    /// Creates a new polkadot client using the provided `config` and connects to `addr`
    ///
    /// # Errors
    /// Will return `Err` when the network is invalid, or when the provided `addr` is unreacheable.
    pub async fn from_config(config: BlockchainConfig, url: &str) -> Result<Self> {
        let ws_client = default_client(url, None).await?;
        let rpc_client = RpcClient::new(ws_client.clone());
        let rpc_methods = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client.clone());
        let backend = LegacyBackend::new(rpc_client);
        let substrate_client =
            OnlineClient::<PolkadotConfig>::from_backend(Arc::new(backend)).await?;
        let ethereum_client = MaybeWsEthereumClient::from_jsonrpsee(config, ws_client).await?;
        Ok(Self { client: ethereum_client, ws_client: substrate_client, rpc_methods })
    }

    async fn account_info(
        &self,
        address: &Address,
        maybe_block: Option<&BlockIdentifier>,
    ) -> Result<AccountInfo<u32, AccountData<u128>>> {
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
            self.rpc_methods
                .chain_get_block_hash(block_number)
                .await?
                .ok_or_else(|| anyhow::anyhow!("no block hash found"))?
        };

        let account_info = self.ws_client.storage().at(block_hash).fetch(&storage_query).await?;

        account_info.map_or_else(
            || {
                Ok(AccountInfo::<u32, AccountData<u128>> {
                    nonce: 0,
                    consumers: 0,
                    providers: 0,
                    sufficients: 0,
                    data: AccountData {
                        free: 0,
                        reserved: 0,
                        misc_frozen: 0,
                        fee_frozen: 0,
                    },
                })
            },
            |account_info| {
                <AccountInfo<u32, AccountData<u128>>>::decode(&mut account_info.encoded())
                    .map_err(|_| anyhow::anyhow!("invalid format"))
            },
        )
    }
}


#[async_trait::async_trait]
impl BlockchainClient for HumanodeClient {
    type MetadataParams = HumanodeMetadataParams;
    type Metadata = HumanodeMetadata;
    type EventStream<'a> = <MaybeWsEthereumClient as BlockchainClient>::EventStream<'a>;
    type Call = EthQuery;
    type CallResult = EthQueryResult;

    fn config(&self) -> &BlockchainConfig {
        self.client.config()
    }

    fn genesis_block(&self) -> &BlockIdentifier {
        self.client.genesis_block()
    }

    async fn node_version(&self) -> Result<String> {
        self.client.node_version().await
    }

    async fn current_block(&self) -> Result<BlockIdentifier> {
        self.client.current_block().await
    }

    async fn finalized_block(&self) -> Result<BlockIdentifier> {
        self.client.finalized_block().await
    }

    async fn balance(&self, address: &Address, block: &BlockIdentifier) -> Result<u128> {
        let balance = match address.format() {
            AddressFormat::Ss58(_) => {
                let account_info = self.account_info(address, Some(block)).await?;
                account_info.data.free
            },
            AddressFormat::Eip55 => {
                // Frontier `eth_getBalance` returns the reducible_balance instead the free balance:
                // https://github.com/paritytech/frontier/blob/polkadot-v0.9.43/frame/evm/src/lib.rs#L853-L855
                // using substrate to get the free balance
                let address = address
                    .evm_to_ss58(Ss58AddressFormat::custom(42))
                    .map_err(|err| anyhow::anyhow!("{}", err))?;
                let account_info = self.account_info(&address, Some(block)).await?;
                account_info.data.free
            },
            AddressFormat::Bech32(_) => return Err(anyhow::anyhow!("invalid address format")),
        };
        Ok(balance)
    }

    async fn coins(&self, address: &Address, block: &BlockIdentifier) -> Result<Vec<Coin>> {
        self.client.coins(address, block).await
    }

    async fn faucet(&self, address: &Address, value: u128) -> Result<Vec<u8>> {
        // convert address
        let dest = {
            let address: H160 = address.address().parse()?;
            let mut data = [0u8; 24];
            data[0..4].copy_from_slice(b"evm:");
            data[4..24].copy_from_slice(&address[..]);
            let hash = sp_core::hashing::blake2_256(&data);
            AccountId32::from(Into::<[u8; 32]>::into(hash))
        };

        // Build the transfer transaction
        let balance_transfer_tx = humanode_metadata::tx().balances().transfer(dest.into(), value);
        let alice = sp_keyring::AccountKeyring::Alice.pair();
        let signer = PairSigner::<PolkadotConfig, _>::new(alice);

        let hash = self
            .ws_client
            .tx()
            .sign_and_submit_then_watch_default(&balance_transfer_tx, &signer)
            .await?
            .wait_for_finalized_success()
            .await?
            .extrinsic_hash();
        Ok(hash.0.to_vec())
    }

    async fn metadata(
        &self,
        public_key: &PublicKey,
        options: &Self::MetadataParams,
    ) -> Result<Self::Metadata> {
        Ok(HumanodeMetadata(self.client.metadata(public_key, &options.0).await?))
    }

    async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        self.client.submit(transaction).await
    }

    async fn block(&self, block_identifier: &PartialBlockIdentifier) -> Result<Block> {
        self.client.block(block_identifier).await
    }

    async fn block_transaction(
        &self,
        block_identifier: &BlockIdentifier,
        tx: &TransactionIdentifier,
    ) -> Result<Transaction> {
        self.client.block_transaction(block_identifier, tx).await
    }

    async fn call(&self, req: &EthQuery) -> Result<EthQueryResult> {
        self.client.call(req).await
    }

    async fn listen<'a>(&'a self) -> Result<Option<Self::EventStream<'a>>> {
        self.client.listen().await
    }
}