use crate::crypto::address::Address;
use crate::crypto::bip32::DerivedSecretKey;
use crate::crypto::bip44::ChildNumber;
use crate::crypto::SecretKey;
use crate::signer::{RosettaAccount, RosettaPublicKey, Signer};
use crate::types::{
    AccountBalanceRequest, AccountCoinsRequest, AccountFaucetRequest, AccountIdentifier, Amount,
    BlockIdentifier, BlockTransaction, Coin, ConstructionMetadataRequest,
    ConstructionSubmitRequest, PublicKey, SearchTransactionsRequest, SearchTransactionsResponse,
    TransactionIdentifier,
};
use crate::{BlockchainConfig, Client, TransactionBuilder};
use anyhow::{Context as _, Result};
use futures::{Future, Stream};
use rosetta_core::types::{
    Block, BlockRequest, BlockTransactionRequest, BlockTransactionResponse, CallRequest,
    CallResponse, PartialBlockIdentifier,
};
use serde_json::{json, Value};
use std::pin::Pin;
use std::task::{Context, Poll};
use surf::utils::async_trait;

pub enum GenericTransactionBuilder {
    Ethereum(rosetta_tx_ethereum::EthereumTransactionBuilder),
    Polkadot(rosetta_tx_polkadot::PolkadotTransactionBuilder),
}

impl GenericTransactionBuilder {
    pub fn new(config: &BlockchainConfig) -> Result<Self> {
        Ok(match config.blockchain {
            "astar" => Self::Ethereum(Default::default()),
            "ethereum" => Self::Ethereum(Default::default()),
            "polkadot" => Self::Polkadot(Default::default()),
            _ => anyhow::bail!("unsupported blockchain"),
        })
    }

    pub fn transfer(&self, address: &Address, amount: u128) -> Result<serde_json::Value> {
        Ok(match self {
            Self::Ethereum(tx) => serde_json::to_value(tx.transfer(address, amount)?)?,
            Self::Polkadot(tx) => serde_json::to_value(tx.transfer(address, amount)?)?,
        })
    }

    pub fn method_call(
        &self,
        contract: &str,
        method: &str,
        params: &[String],
        amount: u128,
    ) -> Result<serde_json::Value> {
        Ok(match self {
            Self::Ethereum(tx) => {
                serde_json::to_value(tx.method_call(contract, method, params, amount)?)?
            }
            Self::Polkadot(tx) => {
                serde_json::to_value(tx.method_call(contract, method, params, amount)?)?
            }
        })
    }

    pub fn deploy_contract(&self, contract_binary: Vec<u8>) -> Result<serde_json::Value> {
        Ok(match self {
            Self::Ethereum(tx) => serde_json::to_value(tx.deploy_contract(contract_binary)?)?,
            Self::Polkadot(tx) => serde_json::to_value(tx.deploy_contract(contract_binary)?)?,
        })
    }

    pub fn create_and_sign(
        &self,
        config: &BlockchainConfig,
        metadata_params: serde_json::Value,
        metadata: serde_json::Value,
        secret_key: &SecretKey,
    ) -> Vec<u8> {
        match self {
            Self::Ethereum(tx) => {
                let metadata_params = serde_json::from_value(metadata_params).unwrap();
                let metadata = serde_json::from_value(metadata).unwrap();
                tx.create_and_sign(config, &metadata_params, &metadata, secret_key)
            }
            Self::Polkadot(tx) => {
                let metadata_params = serde_json::from_value(metadata_params).unwrap();
                let metadata = serde_json::from_value(metadata).unwrap();
                tx.create_and_sign(config, &metadata_params, &metadata, secret_key)
            }
        }
    }
}

/// The wallet provides the main entry point to this crate.
pub struct Wallet {
    config: BlockchainConfig,
    client: Client,
    account: AccountIdentifier,
    secret_key: DerivedSecretKey,
    public_key: PublicKey,
    tx: GenericTransactionBuilder,
}

impl Wallet {
    /// Creates a new wallet from a config, signer and client.
    pub fn new(config: BlockchainConfig, signer: &Signer, client: Client) -> Result<Self> {
        let tx = GenericTransactionBuilder::new(&config)?;
        let secret_key = if config.bip44 {
            signer
                .bip44_account(config.algorithm, config.coin, 0)?
                .derive(ChildNumber::non_hardened_from_u32(0))?
        } else {
            signer.master_key(config.algorithm)?.clone()
        };
        let public_key = secret_key.public_key();
        let account = public_key.to_address(config.address_format).to_rosetta();
        let public_key = public_key.to_rosetta();
        Ok(Self {
            config,
            client,
            account,
            secret_key,
            public_key,
            tx,
        })
    }

    /// Returns the blockchain config.
    pub fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    /// Returns the rosetta client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Returns the public key.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the account identifier.
    pub fn account(&self) -> &AccountIdentifier {
        &self.account
    }

    /// Returns the current block identifier.
    pub async fn status(&self) -> Result<BlockIdentifier> {
        let status = self.client.network_status(self.config.network()).await?;
        Ok(status.current_block_identifier)
    }

    /// Returns the balance of the wallet.
    pub async fn balance(&self) -> Result<Amount> {
        let balance = self
            .client
            .account_balance(&AccountBalanceRequest {
                network_identifier: self.config.network(),
                account_identifier: self.account.clone(),
                block_identifier: None,
                currencies: Some(vec![self.config.currency()]),
            })
            .await?;
        Ok(balance.balances[0].clone())
    }

    /// Returns block data
    /// Takes PartialBlockIdentifier
    pub async fn block(&self, data: PartialBlockIdentifier) -> Result<Block> {
        let req = BlockRequest {
            network_identifier: self.config.network(),
            block_identifier: data,
        };
        let block = self.client.block(&req).await?;
        block.block.context("block not found")
    }

    /// Returns transactions included in a block
    /// Parameters:
    /// 1. block_identifier: BlockIdentifier containing block number and hash
    /// 2. tx_identifier: TransactionIdentifier containing hash of transaction
    pub async fn block_transaction(
        &self,
        block_identifer: BlockIdentifier,
        tx_identifier: TransactionIdentifier,
    ) -> Result<BlockTransactionResponse> {
        let req = BlockTransactionRequest {
            network_identifier: self.config.network(),
            block_identifier: block_identifer,
            transaction_identifier: tx_identifier,
        };
        let block = self.client.block_transaction(&req).await?;
        Ok(block)
    }

    /// Extension of rosetta-api does multiple things
    /// 1. fetching storage
    /// 2. calling extrinsic/contract
    pub async fn call(
        &self,
        method: String,
        params: &serde_json::Value,
        block_identifier: Option<BlockIdentifier>,
    ) -> Result<CallResponse> {
        let req = CallRequest {
            network_identifier: self.config.network(),
            method,
            parameters: params.clone(),
            block_identifier,
        };
        let response = self.client.call(&req).await?;
        Ok(response)
    }

    /// Returns the coins of the wallet.
    pub async fn coins(&self) -> Result<Vec<Coin>> {
        let coins = self
            .client
            .account_coins(&AccountCoinsRequest {
                network_identifier: self.config.network(),
                account_identifier: self.account.clone(),
                include_mempool: false,
                currencies: Some(vec![self.config.currency()]),
            })
            .await?;
        Ok(coins.coins)
    }

    /// Returns the on chain metadata.
    /// Parameters:
    /// - metadata_params: the metadata parameters which we got from transaction builder.
    pub async fn metadata(&self, metadata_params: serde_json::Value) -> Result<serde_json::Value> {
        let req = ConstructionMetadataRequest {
            network_identifier: self.config.network(),
            options: Some(metadata_params),
            public_keys: vec![self.public_key.clone()],
        };
        let response = self.client.construction_metadata(&req).await?;
        Ok(response.metadata)
    }

    /// Submits a transaction and returns the transaction identifier.
    /// Parameters:
    /// - transaction: the transaction bytes to submit
    pub async fn submit(&self, transaction: &[u8]) -> Result<TransactionIdentifier> {
        let req = ConstructionSubmitRequest {
            network_identifier: self.config.network(),
            signed_transaction: hex::encode(transaction),
        };
        let submit = self.client.construction_submit(&req).await?;
        Ok(submit.transaction_identifier)
    }

    /// Creates, signs and submits a transaction.
    pub async fn construct(&self, metadata_params: Value) -> Result<TransactionIdentifier> {
        let metadata = self.metadata(metadata_params.clone()).await?;
        let transaction = self.tx.create_and_sign(
            &self.config,
            metadata_params,
            metadata,
            self.secret_key.secret_key(),
        );
        self.submit(&transaction).await
    }

    /// Makes a transfer.
    /// Parameters:
    /// - account: the account to transfer to
    /// - amount: the amount to transfer
    pub async fn transfer(
        &self,
        account: &AccountIdentifier,
        amount: u128,
    ) -> Result<TransactionIdentifier> {
        let address = Address::new(self.config.address_format, account.address.clone());
        let metadata_params = self.tx.transfer(&address, amount)?;
        self.construct(metadata_params).await
    }

    /// Uses the faucet on dev chains to seed the account with funds.
    /// Parameters:
    /// - faucet_parameter: the amount to seed the account with
    pub async fn faucet(&self, faucet_parameter: u128) -> Result<TransactionIdentifier> {
        let req = AccountFaucetRequest {
            network_identifier: self.config.network(),
            account_identifier: self.account.clone(),
            faucet_parameter,
        };
        let resp = self.client.account_faucet(&req).await?;
        Ok(resp.transaction_identifier)
    }

    /// Returns the transaction matching the transaction identifier.
    /// Parameters:
    /// - tx: the transaction identifier to search for.
    pub async fn transaction(&self, tx: TransactionIdentifier) -> Result<BlockTransaction> {
        let req = SearchTransactionsRequest {
            network_identifier: self.config().network(),
            operator: None,
            max_block: None,
            offset: None,
            limit: None,
            transaction_identifier: Some(tx),
            account_identifier: None,
            coin_identifier: None,
            currency: None,
            status: None,
            r#type: None,
            address: None,
            success: None,
        };
        let resp = self.client.search_transactions(&req).await?;
        anyhow::ensure!(resp.transactions.len() == 1);
        Ok(resp.transactions[0].clone())
    }

    /// Returns a stream of transactions associated with the account.
    pub fn transactions(&self, limit: u16) -> TransactionStream {
        let req = SearchTransactionsRequest {
            network_identifier: self.config().network(),
            operator: None,
            max_block: None,
            offset: None,
            limit: Some(limit as i64),
            transaction_identifier: None,
            account_identifier: Some(self.account.clone()),
            coin_identifier: None,
            currency: None,
            status: None,
            r#type: None,
            address: None,
            success: None,
        };
        TransactionStream::new(self.client.clone(), req)
    }
}

/// Extension trait for the wallet. for ethereum chain
#[async_trait]
pub trait EthereumExt {
    /// deploys contract to chain
    async fn eth_deploy_contract(&self, bytecode: Vec<u8>) -> Result<TransactionIdentifier>;
    /// calls a contract view call function
    async fn eth_view_call(
        &self,
        contract_address: &str,
        method_signature: &str,
        params: &[String],
        block_identifier: Option<BlockIdentifier>,
    ) -> Result<CallResponse>;
    /// calls contract send call function
    async fn eth_send_call(
        &self,
        contract_address: &str,
        method_signature: &str,
        params: &[String],
        amount: u128,
    ) -> Result<TransactionIdentifier>;
    /// estimates gas of send call
    async fn eth_send_call_estimate_gas(
        &self,
        contract_address: &str,
        method_signature: &str,
        params: &[String],
        amount: u128,
    ) -> Result<u128>;
    /// gets storage from ethereum contract
    async fn eth_storage(&self, contract_address: &str, storage_slot: &str)
        -> Result<CallResponse>;
    /// gets storage proof from ethereum contract
    async fn eth_storage_proof(
        &self,
        contract_address: &str,
        storage_slot: &str,
    ) -> Result<CallResponse>;
    /// gets transaction receipt of specific hash
    async fn eth_transaction_receipt(&self, tx_hash: &str) -> Result<CallResponse>;
}

#[async_trait]
impl EthereumExt for Wallet {
    async fn eth_deploy_contract(&self, bytecode: Vec<u8>) -> Result<TransactionIdentifier> {
        let metadata_params = self.tx.deploy_contract(bytecode)?;
        self.construct(metadata_params).await
    }

    async fn eth_send_call(
        &self,
        contract_address: &str,
        method_signature: &str,
        params: &[String],
        amount: u128,
    ) -> Result<TransactionIdentifier> {
        let metadata_params =
            self.tx
                .method_call(contract_address, method_signature, params, amount)?;
        self.construct(metadata_params).await
    }

    async fn eth_send_call_estimate_gas(
        &self,
        contract_address: &str,
        method_signature: &str,
        params: &[String],
        amount: u128,
    ) -> Result<u128> {
        let metadata_params =
            self.tx
                .method_call(contract_address, method_signature, params, amount)?;
        let metadata = self.metadata(metadata_params).await?;
        let metadata: rosetta_config_ethereum::EthereumMetadata = serde_json::from_value(metadata)?;
        Ok(rosetta_tx_ethereum::U256(metadata.gas_limit).as_u128())
    }

    async fn eth_view_call(
        &self,
        contract_address: &str,
        method_signature: &str,
        params: &[String],
        block_identifier: Option<BlockIdentifier>,
    ) -> Result<CallResponse> {
        let method = format!("{}-{}-call", contract_address, method_signature);
        self.call(method, &json!(params), block_identifier).await
    }

    async fn eth_storage(
        &self,
        contract_address: &str,
        storage_slot: &str,
    ) -> Result<CallResponse> {
        let method = format!("{}-{}-storage", contract_address, storage_slot);
        self.call(method, &json!({}), None).await
    }

    async fn eth_storage_proof(
        &self,
        contract_address: &str,
        storage_slot: &str,
    ) -> Result<CallResponse> {
        let method = format!("{}-{}-storage_proof", contract_address, storage_slot);
        self.call(method, &json!({}), None).await
    }

    async fn eth_transaction_receipt(&self, tx_hash: &str) -> Result<CallResponse> {
        let call_method = format!("{}--transaction_receipt", tx_hash);
        self.call(call_method, &json!({}), None).await
    }
}

/// A paged transaction stream.
pub struct TransactionStream {
    client: Client,
    request: SearchTransactionsRequest,
    future: Option<Pin<Box<dyn Future<Output = Result<SearchTransactionsResponse>> + 'static>>>,
    finished: bool,
    total_count: Option<i64>,
}

impl TransactionStream {
    fn new(client: Client, mut request: SearchTransactionsRequest) -> Self {
        request.offset = Some(0);
        Self {
            client,
            request,
            future: None,
            finished: false,
            total_count: None,
        }
    }

    /// Returns the total number of transactions.
    pub fn total_count(&self) -> Option<i64> {
        self.total_count
    }
}

impl Stream for TransactionStream {
    type Item = Result<Vec<BlockTransaction>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        loop {
            if self.finished {
                return Poll::Ready(None);
            } else if let Some(future) = self.future.as_mut() {
                futures::pin_mut!(future);
                match future.poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Ok(response)) => {
                        self.future.take();
                        self.request.offset = response.next_offset;
                        self.total_count = Some(response.total_count);
                        if response.transactions.len() < self.request.limit.unwrap() as _ {
                            self.finished = true;
                        }
                        if response.transactions.is_empty() {
                            continue;
                        }
                        return Poll::Ready(Some(Ok(response.transactions)));
                    }
                    Poll::Ready(Err(error)) => {
                        self.future.take();
                        return Poll::Ready(Some(Err(error)));
                    }
                };
            } else {
                let client = self.client.clone();
                let request = self.request.clone();
                self.future = Some(Box::pin(async move {
                    client.search_transactions(&request).await
                }));
            }
        }
    }
}
