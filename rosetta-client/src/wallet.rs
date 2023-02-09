use crate::crypto::address::Address;
use crate::crypto::bip32::DerivedSecretKey;
use crate::crypto::bip44::ChildNumber;
use crate::signer::{RosettaAccount, RosettaPublicKey, Signer};
use crate::types::{
    AccountBalanceRequest, AccountCoinsRequest, AccountFaucetRequest, AccountIdentifier, Amount,
    BlockIdentifier, Coin, ConstructionMetadataRequest, ConstructionSubmitRequest, PublicKey,
    SearchTransactionsRequest, SearchTransactionsResponse, TransactionIdentifier,
};
use crate::{BlockchainConfig, Client, TransactionBuilder};
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct Wallet {
    config: BlockchainConfig,
    client: Client,
    account: AccountIdentifier,
    secret_key: DerivedSecretKey,
    public_key: PublicKey,
}

impl Wallet {
    pub fn new(config: BlockchainConfig, signer: &Signer, client: Client) -> Result<Self> {
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
        })
    }

    pub fn config(&self) -> &BlockchainConfig {
        &self.config
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn account(&self) -> &AccountIdentifier {
        &self.account
    }

    pub async fn status(&self) -> Result<BlockIdentifier> {
        let status = self.client.network_status(self.config.network()).await?;
        Ok(status.current_block_identifier)
    }

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

    pub async fn metadata<I, O>(&self, metadata_params: I) -> Result<O>
    where
        I: Serialize,
        O: DeserializeOwned + Send + Sync + 'static,
    {
        let req = ConstructionMetadataRequest {
            network_identifier: self.config.network(),
            options: Some(serde_json::to_value(metadata_params)?),
            public_keys: vec![self.public_key.clone()],
        };
        let response = self.client.construction_metadata(&req).await?;
        Ok(serde_json::from_value(response.metadata)?)
    }

    pub async fn submit(&self, transaction: &[u8]) -> Result<TransactionIdentifier> {
        let req = ConstructionSubmitRequest {
            network_identifier: self.config.network(),
            signed_transaction: hex::encode(transaction),
        };
        let submit = self.client.construction_submit(&req).await?;
        Ok(submit.transaction_identifier)
    }

    pub async fn transfer(
        &self,
        account: &AccountIdentifier,
        amount: u128,
    ) -> Result<TransactionIdentifier> {
        anyhow::ensure!(self.config.blockchain == "polkadot");
        let tx = rosetta_tx_polkadot::PolkadotTransactionBuilder;
        let address = Address::new(self.config.address_format, account.address.clone());
        let metadata_params = tx.transfer(&address, amount)?;
        let metadata = self.metadata(metadata_params.clone()).await?;
        let transaction = tx.create_and_sign(
            &self.config,
            &metadata_params,
            &metadata,
            self.secret_key.secret_key(),
        );
        self.submit(&transaction).await
    }

    pub async fn faucet(&self, faucet_parameter: u128) -> Result<TransactionIdentifier> {
        let req = AccountFaucetRequest {
            network_identifier: self.config.network(),
            account_identifier: self.account.clone(),
            faucet_parameter,
        };
        let resp = self.client.account_faucet(&req).await?;
        Ok(resp.transaction_identifier)
    }

    pub async fn transactions(&self) -> Result<SearchTransactionsResponse> {
        let req = SearchTransactionsRequest {
            network_identifier: self.config().network(),
            operator: None,
            max_block: None,
            offset: None,
            limit: None,
            transaction_identifier: None,
            account_identifier: Some(self.account.clone()),
            coin_identifier: None,
            currency: None,
            status: None,
            r#type: None,
            address: None,
            success: None,
        };
        self.client.search_transactions(&req).await
    }
}
