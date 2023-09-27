use crate::client::{GenericClient, GenericMetadata, GenericMetadataParams};
use crate::crypto::address::Address;
use crate::crypto::bip32::DerivedSecretKey;
use crate::crypto::bip44::ChildNumber;
use crate::mnemonic::MnemonicStore;
use crate::signer::{RosettaAccount, RosettaPublicKey, Signer};
use crate::tx_builder::GenericTransactionBuilder;
use crate::types::{
    AccountIdentifier, Amount, BlockIdentifier, Coin, PublicKey, TransactionIdentifier,
};
use crate::{Blockchain, BlockchainConfig};
use anyhow::Result;
use rosetta_core::types::{Block, CallRequest, PartialBlockIdentifier, Transaction};
use rosetta_core::{BlockchainClient, RosettaAlgorithm};
use serde_json::json;
use std::path::Path;

/// The wallet provides the main entry point to this crate.
pub struct Wallet {
    client: GenericClient,
    account: AccountIdentifier,
    secret_key: DerivedSecretKey,
    public_key: PublicKey,
    tx: GenericTransactionBuilder,
}

impl Wallet {
    /// Creates a new wallet from blockchain, network, url and keyfile.
    pub async fn new(
        blockchain: Blockchain,
        network: &str,
        url: &str,
        keyfile: Option<&Path>,
    ) -> Result<Self> {
        let client = GenericClient::new(blockchain, network, url).await?;
        Self::from_client(client, keyfile).await
    }

    /// Creates a new wallet from a config, url and keyfile.
    pub async fn from_config(
        config: BlockchainConfig,
        url: &str,
        keyfile: Option<&Path>,
    ) -> Result<Self> {
        let client = GenericClient::from_config(config, url).await?;
        Self::from_client(client, keyfile).await
    }

    /// Creates a new wallet from a client, url and keyfile.
    pub async fn from_client(client: GenericClient, keyfile: Option<&Path>) -> Result<Self> {
        let store = MnemonicStore::new(keyfile)?;
        let mnemonic = match keyfile {
            Some(_) => store.get_or_generate_mnemonic()?,
            None => store.generate()?,
        };
        let signer = Signer::new(&mnemonic, "")?;
        let tx = GenericTransactionBuilder::new(client.config())?;
        let secret_key = if client.config().bip44 {
            signer
                .bip44_account(client.config().algorithm, client.config().coin, 0)?
                .derive(ChildNumber::non_hardened_from_u32(0))?
        } else {
            signer.master_key(client.config().algorithm)?.clone()
        };
        let public_key = secret_key.public_key();
        let account = public_key
            .to_address(client.config().address_format)
            .to_rosetta();
        let public_key = public_key.to_rosetta();

        if public_key.curve_type != client.config().algorithm.to_curve_type() {
            anyhow::bail!("The signer and client curve type aren't compatible.")
        }

        Ok(Self {
            client,
            account,
            secret_key,
            public_key,
            tx,
        })
    }

    /// Returns the blockchain config.
    pub fn config(&self) -> &BlockchainConfig {
        self.client.config()
    }

    /// Returns the public key.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the account identifier.
    pub fn account(&self) -> &AccountIdentifier {
        &self.account
    }

    /// Returns the latest finalized block identifier.
    pub async fn status(&self) -> Result<BlockIdentifier> {
        self.client.finalized_block().await
    }

    /// Returns the balance of the wallet.
    pub async fn balance(&self) -> Result<Amount> {
        let block = self.client.current_block().await?;
        let address = Address::new(
            self.client.config().address_format,
            self.account.address.clone(),
        );
        let balance = self.client.balance(&address, &block).await?;
        Ok(Amount {
            value: format!("{balance}"),
            currency: self.client.config().currency(),
            metadata: None,
        })
    }

    /// Returns block data
    /// Takes PartialBlockIdentifier
    pub async fn block(&self, data: PartialBlockIdentifier) -> Result<Block> {
        self.client.block(&data).await
    }

    /// Returns transactions included in a block
    /// Parameters:
    /// 1. block_identifier: BlockIdentifier containing block number and hash
    /// 2. tx_identifier: TransactionIdentifier containing hash of transaction
    pub async fn block_transaction(
        &self,
        block_identifer: BlockIdentifier,
        tx_identifier: TransactionIdentifier,
    ) -> Result<Transaction> {
        self.client
            .block_transaction(&block_identifer, &tx_identifier)
            .await
    }

    /// Extension of rosetta-api does multiple things
    /// 1. fetching storage
    /// 2. calling extrinsic/contract
    async fn call(
        &self,
        method: String,
        params: &serde_json::Value,
        block_identifier: Option<PartialBlockIdentifier>,
    ) -> Result<serde_json::Value> {
        let req = CallRequest {
            network_identifier: self.client.config().network(),
            method,
            parameters: params.clone(),
            block_identifier,
        };
        self.client.call(&req).await
    }

    /// Returns the coins of the wallet.
    pub async fn coins(&self) -> Result<Vec<Coin>> {
        let block = self.client.current_block().await?;
        let address = Address::new(
            self.client.config().address_format,
            self.account.address.clone(),
        );
        self.client.coins(&address, &block).await
    }

    /// Returns the on chain metadata.
    /// Parameters:
    /// - metadata_params: the metadata parameters which we got from transaction builder.
    pub async fn metadata(
        &self,
        metadata_params: &GenericMetadataParams,
    ) -> Result<GenericMetadata> {
        let public_key_bytes = hex::decode(&self.public_key.hex_bytes)?;
        let public_key = crate::crypto::PublicKey::from_bytes(
            self.client.config().algorithm,
            &public_key_bytes,
        )?;
        self.client.metadata(&public_key, metadata_params).await
    }

    /// Submits a transaction and returns the transaction identifier.
    /// Parameters:
    /// - transaction: the transaction bytes to submit
    pub async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        self.client.submit(transaction).await
    }

    /// Creates, signs and submits a transaction.
    pub async fn construct(&self, params: &GenericMetadataParams) -> Result<Vec<u8>> {
        let metadata = self.metadata(params).await?;
        let transaction = self.tx.create_and_sign(
            self.client.config(),
            params,
            &metadata,
            self.secret_key.secret_key(),
        )?;
        self.submit(&transaction).await
    }

    /// Makes a transfer.
    /// Parameters:
    /// - account: the account to transfer to
    /// - amount: the amount to transfer
    pub async fn transfer(&self, account: &AccountIdentifier, amount: u128) -> Result<Vec<u8>> {
        let address = Address::new(self.client.config().address_format, account.address.clone());
        let metadata_params = self.tx.transfer(&address, amount)?;
        self.construct(&metadata_params).await
    }

    /// Uses the faucet on dev chains to seed the account with funds.
    /// Parameters:
    /// - faucet_parameter: the amount to seed the account with
    pub async fn faucet(&self, faucet_parameter: u128) -> Result<Vec<u8>> {
        let address = Address::new(
            self.client.config().address_format,
            self.account.address.clone(),
        );
        self.client.faucet(&address, faucet_parameter).await
    }

    /// deploys contract to chain
    pub async fn eth_deploy_contract(&self, bytecode: Vec<u8>) -> Result<Vec<u8>> {
        let metadata_params = self.tx.deploy_contract(bytecode)?;
        self.construct(&metadata_params).await
    }

    /// calls contract send call function
    pub async fn eth_send_call(
        &self,
        contract_address: &str,
        method_signature: &str,
        params: &[String],
        amount: u128,
    ) -> Result<Vec<u8>> {
        let metadata_params =
            self.tx
                .method_call(contract_address, method_signature, params, amount)?;
        self.construct(&metadata_params).await
    }

    /// estimates gas of send call
    pub async fn eth_send_call_estimate_gas(
        &self,
        contract_address: &str,
        method_signature: &str,
        params: &[String],
        amount: u128,
    ) -> Result<u128> {
        let metadata_params =
            self.tx
                .method_call(contract_address, method_signature, params, amount)?;
        let metadata = match self.metadata(&metadata_params).await? {
            GenericMetadata::Ethereum(metadata) => metadata,
            GenericMetadata::Astar(metadata) => metadata.0,
            _ => anyhow::bail!("unsupported op"),
        };
        Ok(rosetta_tx_ethereum::U256(metadata.gas_limit).as_u128())
    }

    /// calls a contract view call function
    pub async fn eth_view_call(
        &self,
        contract_address: &str,
        method_signature: &str,
        params: &[String],
        block_identifier: Option<PartialBlockIdentifier>,
    ) -> Result<serde_json::Value> {
        let method = format!("{}-{}-call", contract_address, method_signature);
        self.call(method, &json!(params), block_identifier).await
    }

    /// gets storage from ethereum contract
    pub async fn eth_storage(
        &self,
        contract_address: &str,
        storage_slot: &str,
        block_identifier: Option<PartialBlockIdentifier>,
    ) -> Result<serde_json::Value> {
        let method = format!("{}-{}-storage", contract_address, storage_slot);
        self.call(method, &json!({}), block_identifier).await
    }

    /// gets storage proof from ethereum contract
    pub async fn eth_storage_proof(
        &self,
        contract_address: &str,
        storage_slot: &str,
        block_identifier: Option<PartialBlockIdentifier>,
    ) -> Result<serde_json::Value> {
        let method = format!("{}-{}-storage_proof", contract_address, storage_slot);
        self.call(method, &json!({}), block_identifier).await
    }

    /// gets transaction receipt of specific hash
    pub async fn eth_transaction_receipt(&self, tx_hash: &[u8]) -> Result<serde_json::Value> {
        let call_method = format!("{}--transaction_receipt", hex::encode(tx_hash));
        self.call(call_method, &json!({}), None).await
    }
}
