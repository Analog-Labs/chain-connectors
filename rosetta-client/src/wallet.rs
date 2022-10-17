use crate::crypto::bip32::DerivedSecretKey;
use crate::crypto::bip44::ChildNumber;
use crate::signer::{RosettaPublicKey, RosettaSecretKey, Signer};
use crate::types::{
    AccountBalanceRequest, AccountCoinsRequest, AccountIdentifier, Amount, Coin,
    ConstructionCombineRequest, ConstructionDeriveRequest, ConstructionHashRequest,
    ConstructionMetadataRequest, ConstructionMetadataResponse, ConstructionParseRequest,
    ConstructionParseResponse, ConstructionPayloadsRequest, ConstructionPayloadsResponse,
    ConstructionPreprocessRequest, ConstructionPreprocessResponse, ConstructionSubmitRequest,
    Operation, PublicKey, Signature, TransactionIdentifier,
};
use crate::{BlockchainConfig, Client, TransactionBuilder};
use anyhow::Result;
use serde_json::Value;

pub struct Wallet {
    config: BlockchainConfig,
    client: Client,
    account: AccountIdentifier,
    secret_key: DerivedSecretKey,
    public_key: PublicKey,
}

impl Wallet {
    pub async fn new(config: BlockchainConfig, signer: &Signer) -> Result<Self> {
        let secret_key = if config.bip44 {
            signer
                .bip44_account(config.algorithm, config.coin, 0)?
                .derive(ChildNumber::non_hardened_from_u32(0))?
        } else {
            signer.master_key(config.algorithm)?.clone()
        };
        let public_key = secret_key.public_key().to_rosetta();

        let client = Client::new(&config.url)?;
        let req = ConstructionDeriveRequest {
            network_identifier: config.network.clone(),
            public_key: public_key.clone(),
            metadata: None,
        };
        let derive = client.construction_derive(&req).await?;
        let account = derive
            .account_identifier
            .ok_or_else(|| anyhow::anyhow!("expected account"))?;
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

    pub async fn balance(&self) -> Result<Amount> {
        let balance = self
            .client
            .account_balance(&AccountBalanceRequest {
                network_identifier: self.config.network.clone(),
                account_identifier: self.account.clone(),
                block_identifier: None,
                currencies: Some(vec![self.config.currency.clone()]),
            })
            .await?;
        Ok(balance.balances[0].clone())
    }

    pub async fn coins(&self) -> Result<Vec<Coin>> {
        let coins = self
            .client
            .account_coins(&AccountCoinsRequest {
                network_identifier: self.config.network.clone(),
                account_identifier: self.account.clone(),
                include_mempool: false,
                currencies: Some(vec![self.config.currency.clone()]),
            })
            .await?;
        Ok(coins.coins)
    }

    pub async fn preprocess(
        &self,
        operations: &[Operation],
    ) -> Result<ConstructionPreprocessResponse> {
        let req = ConstructionPreprocessRequest {
            network_identifier: self.config.network.clone(),
            operations: operations.to_vec(),
            metadata: None,
        };
        self.client.construction_preprocess(&req).await
    }

    pub async fn metadata(&self, options: &Option<Value>) -> Result<ConstructionMetadataResponse> {
        let req = ConstructionMetadataRequest {
            network_identifier: self.config.network.clone(),
            options: options.clone(),
        };
        self.client.construction_metadata(&req).await
    }

    pub async fn payloads(
        &self,
        operations: &[Operation],
        metadata: &Value,
    ) -> Result<ConstructionPayloadsResponse> {
        let req = ConstructionPayloadsRequest {
            network_identifier: self.config.network.clone(),
            operations: operations.to_vec(),
            public_keys: None,
            metadata: Some(metadata.clone()),
        };
        self.client.construction_payloads(&req).await
    }

    pub async fn combine(
        &self,
        unsigned_transaction: &str,
        signatures: Vec<Signature>,
    ) -> Result<String> {
        let req = ConstructionCombineRequest {
            network_identifier: self.config.network.clone(),
            signatures,
            unsigned_transaction: unsigned_transaction.to_string(),
        };
        let combine = self.client.construction_combine(&req).await?;
        Ok(combine.signed_transaction)
    }

    pub async fn parse(&self, tx: &str) -> Result<ConstructionParseResponse> {
        let req = ConstructionParseRequest {
            network_identifier: self.config.network.clone(),
            signed: true,
            transaction: tx.to_string(),
        };
        self.client.construction_parse(&req).await
    }

    pub async fn hash(&self, tx: &str) -> Result<TransactionIdentifier> {
        let req = ConstructionHashRequest {
            network_identifier: self.config.network.clone(),
            signed_transaction: tx.to_string(),
        };
        let hash = self.client.construction_hash(&req).await?;
        Ok(hash.transaction_identifier)
    }

    pub async fn submit(&self, tx: &str) -> Result<TransactionIdentifier> {
        let req = ConstructionSubmitRequest {
            network_identifier: self.config.network.clone(),
            signed_transaction: tx.to_string(),
        };
        let submit = self.client.construction_submit(&req).await?;
        Ok(submit.transaction_identifier)
    }

    pub async fn transfer(
        &self,
        account: &AccountIdentifier,
        amount: u128,
    ) -> Result<TransactionIdentifier> {
        let mut tx = TransactionBuilder::new();
        if self.config.utxo {
            let coins = self.coins().await?;
            for coin in &coins {
                if tx.input_amount() > amount {
                    break;
                }
                tx.input(&self.account, coin)?;
            }
            tx.output(account, amount, &self.config.currency);
            tx.output(self.account(), 0, &self.config.currency);
        } else {
            tx.transfer(&self.account, account, amount, &self.config.currency);
        }

        let preprocess = self.preprocess(tx.operations()).await?;
        let metadata = self.metadata(&preprocess.options).await?;
        let fee: u128 = if let Some(suggested_fee) = &metadata.suggested_fee {
            anyhow::ensure!(suggested_fee[0].currency == self.config.currency);
            suggested_fee[0].value.parse()?
        } else {
            0
        };

        if self.config.utxo {
            let change_amount = tx
                .input_amount()
                .checked_sub(amount)
                .ok_or_else(|| anyhow::anyhow!("overflowed"))?
                .checked_sub(fee)
                .ok_or_else(|| anyhow::anyhow!("overflowed"))?;
            tx.pop();
            tx.output(&self.account, change_amount, &self.config.currency);
        }

        let payloads = self.payloads(tx.operations(), &metadata.metadata).await?;
        let signatures = payloads
            .payloads
            .into_iter()
            .map(|payload| self.secret_key.sign(payload))
            .collect::<Result<Vec<_>>>()?;
        let tx = self
            .combine(&payloads.unsigned_transaction, signatures)
            .await?;

        self.submit(&tx).await
    }
}
