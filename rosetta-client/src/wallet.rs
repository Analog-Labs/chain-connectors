use crate::{
    client::{GenericClient, GenericMetadata, GenericMetadataParams},
    crypto::{address::Address, bip32::DerivedSecretKey, bip44::ChildNumber, Signature},
    signer::{RosettaAccount, RosettaPublicKey, Signer},
    tx_builder::GenericTransactionBuilder,
    types::{AccountIdentifier, BlockIdentifier, PublicKey},
    Blockchain, BlockchainConfig,
};
use anyhow::Result;
use futures::channel::mpsc;
use futures::{SinkExt, Stream, StreamExt};
use rosetta_core::{
    types::PartialBlockIdentifier, BlockOrIdentifier, BlockchainClient, ClientEvent,
    RosettaAlgorithm,
};
use rosetta_server_ethereum::{
    config::{
        ext::types::{self as ethereum_types, Address as EthAddress, H256, U256},
        AtBlock, CallContract, CallResult, EIP1186ProofResponse, GetProof, GetStorageAt,
        GetTransactionReceipt, Query as EthQuery, QueryResult as EthQueryResult,
        TransactionReceipt,
    },
    SubmitResult,
};
use std::pin::Pin;
use std::sync::Arc;

/// The wallet provides the main entry point to this crate.
pub struct Wallet {
    /// `GenericClient` instance
    pub client: Arc<GenericClient>,
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
        mnemonic: &str,
        private_key: Option<[u8; 32]>,
    ) -> Result<Self> {
        let client = GenericClient::new(blockchain, network, url, private_key).await?;
        Self::from_client(client, mnemonic)
    }

    /// Creates a new wallet from a config, url and keyfile.
    #[allow(clippy::missing_errors_doc)]
    pub async fn from_config(
        config: BlockchainConfig,
        url: &str,
        mnemonic: &str,
        private_key: Option<[u8; 32]>,
    ) -> Result<Self> {
        let client = GenericClient::from_config(config, url, private_key).await?;
        Self::from_client(client, mnemonic)
    }

    /// Creates a new wallet from a client, url and keyfile.
    #[allow(clippy::missing_errors_doc)]
    pub fn from_client(client: GenericClient, mnemonic: &str) -> Result<Self> {
        /*let store = MnemonicStore::new(keyfile)?;
        let mnemonic = match keyfile {
            Some(_) => store.get_or_generate_mnemonic()?,
            None => store.generate()?,
        };*/
        let signer = Signer::new(&mnemonic.parse()?, "")?;
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

        Ok(Self { client: Arc::new(client), account, secret_key, public_key, tx })
    }

    /// Returns the blockchain config.
    pub fn config(&self) -> &BlockchainConfig {
        self.client.config()
    }

    /// Signs a prehashed message.
    pub fn sign_prehashed(&self, hash: &[u8]) -> Result<Signature> {
        self.secret_key.secret_key().sign_prehashed(hash)
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
        // self.client.finalized_block().await
        match &*self.client {
            GenericClient::Astar(client) => client.finalized_block().await,
            GenericClient::Ethereum(client) => client.finalized_block().await,
            GenericClient::Polkadot(client) => client.finalized_block().await,
        }
    }

    /// Returns the balance of the wallet.
    #[allow(clippy::missing_errors_doc)]
    pub async fn balance(&self, address: String) -> Result<u128> {
        let block = self.client.current_block().await?;
        let address = Address::new(self.client.config().address_format, address);
        let balance = match &*self.client {
            GenericClient::Astar(client) => {
                client.balance(&address, &PartialBlockIdentifier::from(block)).await?
            },
            GenericClient::Ethereum(client) => {
                client.balance(&address, &PartialBlockIdentifier::from(block)).await?
            },
            GenericClient::Polkadot(client) => {
                client.balance(&address, &PartialBlockIdentifier::from(block)).await?
            },
        };
        Ok(balance)
    }

    /// Return a stream of events, return None if the blockchain doesn't support events.
    #[allow(clippy::missing_errors_doc)]
    pub async fn listen(
        &self,
    ) -> Result<Option<<GenericClient as BlockchainClient>::EventStream<'_>>> {
        self.client.listen().await
    }

    /// Returns a stream of finalized blocks.
    pub fn block_stream(&self) -> Pin<Box<dyn Stream<Item = u64> + Send>> {
        let (mut tx, rx) = mpsc::channel(1);
        let client = self.client.clone();
        // spawn a task to avoid lifetime issue
        tokio::task::spawn(async move {
            loop {
                let mut stream = match client.listen().await {
                    Ok(Some(stream)) => stream,
                    Ok(None) => {
                        log::debug!("error opening listener");
                        continue;
                    },
                    Err(err) => {
                        log::debug!("error opening listener {}", err);
                        continue;
                    },
                };
                while let Some(event) = stream.next().await {
                    match event {
                        ClientEvent::NewFinalized(BlockOrIdentifier::Identifier(identifier)) => {
                            if tx.send(identifier.index).await.is_err() {
                                return;
                            }
                        },
                        ClientEvent::NewFinalized(BlockOrIdentifier::Block(block)) => {
                            if tx.send(block.block_identifier.index).await.is_err() {
                                return;
                            }
                        },
                        ClientEvent::NewHead(_) => {},
                        ClientEvent::Event(_) => {},
                        ClientEvent::Close(reason) => {
                            log::warn!("block stream closed {}", reason);
                        },
                    }
                }
            }
        });
        Box::pin(rx)
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
    pub async fn submit(&self, transaction: &[u8]) -> Result<SubmitResult> {
        self.client.submit(transaction).await
    }

    /// Creates, signs and submits a transaction.
    #[allow(clippy::missing_errors_doc)]
    pub async fn construct(&self, params: &GenericMetadataParams) -> Result<SubmitResult> {
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
    pub async fn transfer(
        &self,
        account: &AccountIdentifier,
        amount: u128,
        nonce: Option<u64>,
        gas_limit: Option<u64>,
    ) -> Result<SubmitResult> {
        let address = Address::new(self.client.config().address_format, account.address.clone());
        let mut metadata_params = self.tx.transfer(&address, amount)?;
        update_metadata_params(&mut metadata_params, nonce, gas_limit)?;
        self.construct(&metadata_params).await
    }

    /// Uses the faucet on dev chains to seed the account with funds.
    /// Parameters:
    /// - `faucet_parameter`: the amount to seed the account with
    #[allow(clippy::missing_errors_doc)]
    pub async fn faucet(
        &self,
        faucet_parameter: u128,
        high_gas_price: Option<u128>,
    ) -> Result<Vec<u8>> {
        let address =
            Address::new(self.client.config().address_format, self.account.address.clone());
        self.client.faucet(&address, faucet_parameter, high_gas_price).await
    }

    /// deploys contract to chain
    #[allow(clippy::missing_errors_doc)]
    pub async fn eth_deploy_contract(&self, bytecode: Vec<u8>) -> Result<SubmitResult> {
        let metadata_params = self.tx.deploy_contract(bytecode)?;
        self.construct(&metadata_params).await
    }

    /// calls contract send call function
    #[allow(clippy::missing_errors_doc)]
    pub async fn eth_send_call(
        &self,
        contract_address: [u8; 20],
        data: Vec<u8>,
        amount: u128,
        nonce: Option<u64>,
        gas_limit: Option<u64>,
    ) -> Result<SubmitResult> {
        let mut metadata_params = self.tx.method_call(&contract_address, data.as_ref(), amount)?;
        update_metadata_params(&mut metadata_params, nonce, gas_limit)?;
        self.construct(&metadata_params).await
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
                GenericMetadata::Polkadot(_) => anyhow::bail!("unsupported op"),
            };
        Ok(u128::from(metadata.gas_limit))
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
        let result = match &*self.client {
            GenericClient::Ethereum(client) => client.call(&EthQuery::CallContract(call)).await?,
            GenericClient::Astar(client) => client.call(&EthQuery::CallContract(call)).await?,
            GenericClient::Polkadot(_) => anyhow::bail!("polkadot doesn't support eth_view_call"),
        };
        let EthQueryResult::CallContract(exit_reason) = result else {
            anyhow::bail!("[this is a bug] invalid result type");
        };
        Ok(exit_reason)
    }

    /// Peforms an arbitrary query to EVM compatible blockchain.
    ///
    /// # Errors
    /// Returns `Err` if the blockchain doesn't support EVM calls, or the due another client issue
    pub async fn query<Q: rosetta_server_ethereum::QueryItem>(
        &self,
        query: Q,
    ) -> Result<<Q as rosetta_core::traits::Query>::Result> {
        let query = <Q as rosetta_server_ethereum::QueryItem>::into_query(query);
        let result = match &*self.client {
            GenericClient::Ethereum(client) => client.call(&query).await?,
            GenericClient::Astar(client) => client.call(&query).await?,
            GenericClient::Polkadot(_) => anyhow::bail!("polkadot doesn't support eth_view_call"),
        };
        let result = <Q as rosetta_server_ethereum::QueryItem>::parse_result(result)?;
        Ok(result)
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
        let result = match &*self.client {
            GenericClient::Ethereum(client) => {
                client.call(&EthQuery::GetStorageAt(get_storage)).await?
            },
            GenericClient::Astar(client) => {
                client.call(&EthQuery::GetStorageAt(get_storage)).await?
            },
            GenericClient::Polkadot(_) => anyhow::bail!("polkadot doesn't support eth_storage"),
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
        let result = match &*self.client {
            GenericClient::Ethereum(client) => client.call(&EthQuery::GetProof(get_proof)).await?,
            GenericClient::Astar(client) => client.call(&EthQuery::GetProof(get_proof)).await?,
            GenericClient::Polkadot(_) => anyhow::bail!("polkadot doesn't support eth_storage"),
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
        let result = match &*self.client {
            GenericClient::Ethereum(client) => {
                client.call(&EthQuery::GetTransactionReceipt(get_tx_receipt)).await?
            },
            GenericClient::Astar(client) => {
                client.call(&EthQuery::GetTransactionReceipt(get_tx_receipt)).await?
            },
            GenericClient::Polkadot(_) => anyhow::bail!("polkadot doesn't support eth_storage"),
        };
        let EthQueryResult::GetTransactionReceipt(maybe_receipt) = result else {
            anyhow::bail!("[this is a bug] invalid result type");
        };
        Ok(maybe_receipt)
    }

    /// gets the currently configured chain ID, a value used in replay-protected transaction signing
    /// as introduced by EIP-155.
    /// # Errors
    /// Returns `Err` if the blockchain doesn't support `eth_chainId` or the client connection
    /// failed.
    pub async fn eth_chain_id(&self) -> Result<u64> {
        let result = match &*self.client {
            GenericClient::Ethereum(client) => client.call(&EthQuery::ChainId).await?,
            GenericClient::Astar(client) => client.call(&EthQuery::ChainId).await?,
            GenericClient::Polkadot(_) => anyhow::bail!("polkadot doesn't support eth_chainId"),
        };
        let EthQueryResult::ChainId(value) = result else {
            anyhow::bail!("[this is a bug] invalid result type");
        };
        Ok(value)
    }
}

/// Updates the metadata parameters with the given nonce and gas limit.
fn update_metadata_params(
    params: &mut GenericMetadataParams,
    nonce: Option<u64>,
    gas_limit: Option<u64>,
) -> Result<()> {
    match params {
        GenericMetadataParams::Ethereum(params) => {
            params.nonce = nonce;
            params.gas_limit = gas_limit;
        },
        GenericMetadataParams::Astar(params) => {
            params.0.nonce = nonce;
            params.0.gas_limit = gas_limit;
        },
        GenericMetadataParams::Polkadot(params) => {
            if let Some(nonce) = nonce {
                if let Ok(nonce) = u32::try_from(nonce) {
                    params.nonce = Some(nonce);
                } else {
                    anyhow::bail!("invalid nonce");
                }
            }
        },
    }
    Ok(())
}
