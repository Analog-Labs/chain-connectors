use crate::{
    client::{GenericClient, GenericMetadata, GenericMetadataParams},
    crypto::{address::Address, bip32::DerivedSecretKey, bip44::ChildNumber},
    mnemonic::MnemonicStore,
    signer::{RosettaAccount, RosettaPublicKey, Signer},
    tx_builder::GenericTransactionBuilder,
    types::{AccountIdentifier, Amount, BlockIdentifier, Coin, PublicKey, TransactionIdentifier},
    Blockchain, BlockchainConfig,
};
use anyhow::Result;
use rosetta_core::{
    types::{Block, PartialBlockIdentifier, Transaction},
    BlockchainClient, RosettaAlgorithm,
};
use rosetta_server_ethereum::config::{
    ethereum_types::{self, Address as EthAddress, H256, U256},
    AtBlock, CallContract, CallResult, EIP1186ProofResponse, GetProof, GetStorageAt,
    GetTransactionReceipt, Query as EthQuery, QueryResult as EthQueryResult, TransactionReceipt,
};
use std::path::Path;

/// The wallet provides the main entry point to this crate.
pub struct Wallet {
    /// GenericClient instance
    pub client: GenericClient,
    account: AccountIdentifier,
    secret_key: DerivedSecretKey,
    public_key: PublicKey,
    tx: GenericTransactionBuilder,
}

impl Wallet {
    /// Creates a new wallet from blockchain, network, url and keyfile.
    #[allow(clippy::missing_errors_doc)]
    pub async fn new(
        blockchain: Blockchain,
        network: &str,
        url: &str,
        keyfile: Option<&Path>,
    ) -> Result<Self> {
        let client = GenericClient::new(blockchain, network, url).await?;
        Self::from_client(client, keyfile)
    }

    /// Creates a new wallet from a config, url and keyfile.
    #[allow(clippy::missing_errors_doc)]
    pub async fn from_config(
        config: BlockchainConfig,
        url: &str,
        keyfile: Option<&Path>,
    ) -> Result<Self> {
        let client = GenericClient::from_config(config, url).await?;
        Self::from_client(client, keyfile)
    }

    /// Creates a new wallet from a client, url and keyfile.
    #[allow(clippy::missing_errors_doc)]
    pub fn from_client(client: GenericClient, keyfile: Option<&Path>) -> Result<Self> {
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
            signer.master_key(client.config().algorithm).clone()
        };
        let public_key = secret_key.public_key();
        let account = public_key.to_address(client.config().address_format).to_rosetta();
        let public_key = public_key.to_rosetta();

        if public_key.curve_type != client.config().algorithm.to_curve_type() {
            anyhow::bail!("The signer and client curve type aren't compatible.")
        }

        Ok(Self { client, account, secret_key, public_key, tx })
    }

    /// Returns the blockchain config.
    pub fn config(&self) -> &BlockchainConfig {
        self.client.config()
    }

    /// Returns the public key.
    pub const fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the account identifier.
    pub const fn account(&self) -> &AccountIdentifier {
        &self.account
    }

    /// Returns the latest finalized block identifier.
    #[allow(clippy::missing_errors_doc)]
    pub async fn status(&self) -> Result<BlockIdentifier> {
        self.client.finalized_block().await
    }

    /// Returns the balance of the wallet.
    #[allow(clippy::missing_errors_doc)]
    pub async fn balance(&self) -> Result<Amount> {
        let block = self.client.current_block().await?;
        let address =
            Address::new(self.client.config().address_format, self.account.address.clone());
        let balance = self.client.balance(&address, &block).await?;
        Ok(Amount {
            value: format!("{balance}"),
            currency: self.client.config().currency(),
            metadata: None,
        })
    }

    /// Return a stream of events, return None if the blockchain doesn't support events.
    #[allow(clippy::missing_errors_doc)]
    pub async fn listen(
        &self,
    ) -> Result<Option<<GenericClient as BlockchainClient>::EventStream<'_>>> {
        self.client.listen().await
    }

    /// Returns block data
    /// Takes `PartialBlockIdentifier`
    #[allow(clippy::missing_errors_doc)]
    pub async fn block(&self, data: PartialBlockIdentifier) -> Result<Block> {
        self.client.block(&data).await
    }

    /// Returns transactions included in a block
    /// Parameters:
    /// 1. `block_identifier`: `BlockIdentifier` containing block number and hash
    /// 2. `tx_identifier`: `TransactionIdentifier` containing hash of transaction
    #[allow(clippy::missing_errors_doc)]
    pub async fn block_transaction(
        &self,
        block_identifer: BlockIdentifier,
        tx_identifier: TransactionIdentifier,
    ) -> Result<Transaction> {
        self.client.block_transaction(&block_identifer, &tx_identifier).await
    }

    /// Returns the coins of the wallet.
    #[allow(clippy::missing_errors_doc)]
    pub async fn coins(&self) -> Result<Vec<Coin>> {
        let block = self.client.current_block().await?;
        let address =
            Address::new(self.client.config().address_format, self.account.address.clone());
        self.client.coins(&address, &block).await
    }

    /// Returns the on chain metadata.
    /// Parameters:
    /// - `metadata_params`: the metadata parameters which we got from transaction builder.
    #[allow(clippy::missing_errors_doc)]
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
    #[allow(clippy::missing_errors_doc)]
    pub async fn submit(&self, transaction: &[u8]) -> Result<Vec<u8>> {
        self.client.submit(transaction).await
    }

    /// Creates, signs and submits a transaction.
    #[allow(clippy::missing_errors_doc)]
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
    #[allow(clippy::missing_errors_doc)]
    pub async fn transfer(&self, account: &AccountIdentifier, amount: u128) -> Result<Vec<u8>> {
        let address = Address::new(self.client.config().address_format, account.address.clone());
        let metadata_params = self.tx.transfer(&address, amount)?;
        self.construct(&metadata_params).await
    }

    /// Uses the faucet on dev chains to seed the account with funds.
    /// Parameters:
    /// - `faucet_parameter`: the amount to seed the account with
    #[allow(clippy::missing_errors_doc)]
    pub async fn faucet(&self, faucet_parameter: u128) -> Result<Vec<u8>> {
        let address =
            Address::new(self.client.config().address_format, self.account.address.clone());
        self.client.faucet(&address, faucet_parameter).await
    }

    /// deploys contract to chain
    #[allow(clippy::missing_errors_doc)]
    pub async fn eth_deploy_contract(&self, bytecode: Vec<u8>) -> Result<[u8; 32]> {
        let metadata_params = self.tx.deploy_contract(bytecode)?;
        let bytes = self.construct(&metadata_params).await?;
        let mut tx_hash = [0u8; 32];
        tx_hash.copy_from_slice(&bytes[0..32]);
        Ok(tx_hash)
    }

    /// calls contract send call function
    #[allow(clippy::missing_errors_doc)]
    pub async fn eth_send_call(
        &self,
        contract_address: [u8; 20],
        data: Vec<u8>,
        amount: u128,
    ) -> Result<[u8; 32]> {
        let metadata_params = self.tx.method_call(&contract_address, data.as_ref(), amount)?;
        let bytes = self.construct(&metadata_params).await?;
        let mut tx_hash = [0u8; 32];
        tx_hash.copy_from_slice(&bytes[0..32]);
        Ok(tx_hash)
    }

    /// estimates gas of send call
    #[allow(clippy::missing_errors_doc)]
    pub async fn eth_send_call_estimate_gas(
        &self,
        contract_address: [u8; 20],
        data: Vec<u8>,
        amount: u128,
    ) -> Result<u128> {
        let metadata_params = self.tx.method_call(&contract_address, data.as_ref(), amount)?;
        let metadata: rosetta_server_ethereum::EthereumMetadata =
            match self.metadata(&metadata_params).await? {
                GenericMetadata::Ethereum(metadata) => metadata,
                GenericMetadata::Astar(metadata) => metadata.0,
                _ => anyhow::bail!("unsupported op"),
            };
        Ok(rosetta_tx_ethereum::U256(metadata.gas_limit).as_u128())
    }

    /// calls a contract view call function
    #[allow(clippy::missing_errors_doc)]
    pub async fn eth_view_call(
        &self,
        contract_address: [u8; 20],
        data: Vec<u8>,
        block_identifier: AtBlock,
    ) -> Result<CallResult> {
        let contract_address = EthAddress::from(contract_address);
        let call = CallContract {
            from: None,
            to: contract_address,
            value: U256::zero(),
            data,
            block: block_identifier,
        };
        let result = match &self.client {
            GenericClient::Ethereum(client) => client.call(&EthQuery::CallContract(call)).await?,
            GenericClient::Astar(client) => client.call(&EthQuery::CallContract(call)).await?,
            GenericClient::Polkadot(_) => anyhow::bail!("polkadot doesn't support eth_view_call"),
            GenericClient::Bitcoin(_) => anyhow::bail!("bitcoin doesn't support eth_view_call"),
        };
        let EthQueryResult::CallContract(exit_reason) = result else {
            anyhow::bail!("[this is a bug] invalid result type");
        };
        Ok(exit_reason)
    }

    /// gets storage from ethereum contract
    #[allow(clippy::missing_errors_doc)]
    pub async fn eth_storage(
        &self,
        contract_address: [u8; 20],
        storage_slot: [u8; 32],
        block_identifier: AtBlock,
    ) -> Result<H256> {
        let contract_address = EthAddress::from(contract_address);
        let storage_slot = H256(storage_slot);
        let get_storage =
            GetStorageAt { address: contract_address, at: storage_slot, block: block_identifier };
        let result = match &self.client {
            GenericClient::Ethereum(client) => {
                client.call(&EthQuery::GetStorageAt(get_storage)).await?
            },
            GenericClient::Astar(client) => {
                client.call(&EthQuery::GetStorageAt(get_storage)).await?
            },
            GenericClient::Polkadot(_) => anyhow::bail!("polkadot doesn't support eth_storage"),
            GenericClient::Bitcoin(_) => anyhow::bail!("bitcoin doesn't support eth_storage"),
        };
        let EthQueryResult::GetStorageAt(value) = result else {
            anyhow::bail!("[this is a bug] invalid result type");
        };
        Ok(value)
    }

    /// gets storage proof from ethereum contract
    #[allow(clippy::missing_errors_doc)]
    pub async fn eth_storage_proof<I: Iterator<Item = ethereum_types::H256> + Send + Sync>(
        &self,
        contract_address: [u8; 20],
        storage_keys: I,
        block_identifier: AtBlock,
    ) -> Result<EIP1186ProofResponse> {
        use ethereum_types::Address;
        let contract_address = Address::from(contract_address);
        let get_proof = GetProof {
            account: contract_address,
            storage_keys: storage_keys.collect(),
            block: block_identifier,
        };
        let result = match &self.client {
            GenericClient::Ethereum(client) => client.call(&EthQuery::GetProof(get_proof)).await?,
            GenericClient::Astar(client) => client.call(&EthQuery::GetProof(get_proof)).await?,
            GenericClient::Polkadot(_) => anyhow::bail!("polkadot doesn't support eth_storage"),
            GenericClient::Bitcoin(_) => anyhow::bail!("bitcoin doesn't support eth_storage"),
        };
        let EthQueryResult::GetProof(proof) = result else {
            anyhow::bail!("[this is a bug] invalid result type");
        };
        Ok(proof)
    }

    /// gets transaction receipt of specific hash
    #[allow(clippy::missing_errors_doc)]
    pub async fn eth_transaction_receipt(
        &self,
        tx_hash: [u8; 32],
    ) -> Result<Option<TransactionReceipt>> {
        let tx_hash = H256(tx_hash);
        let get_tx_receipt = GetTransactionReceipt { tx_hash };
        let result = match &self.client {
            GenericClient::Ethereum(client) => {
                client.call(&EthQuery::GetTransactionReceipt(get_tx_receipt)).await?
            },
            GenericClient::Astar(client) => {
                client.call(&EthQuery::GetTransactionReceipt(get_tx_receipt)).await?
            },
            GenericClient::Polkadot(_) => anyhow::bail!("polkadot doesn't support eth_storage"),
            GenericClient::Bitcoin(_) => anyhow::bail!("bitcoin doesn't support eth_storage"),
        };
        let EthQueryResult::GetTransactionReceipt(maybe_receipt) = result else {
            anyhow::bail!("[this is a bug] invalid result type");
        };
        Ok(maybe_receipt)
    }
}
