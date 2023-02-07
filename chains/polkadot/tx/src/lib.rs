use anyhow::{Context, Result};
use parity_scale_codec::{Decode, Encode};
use rosetta_core::crypto::address::Address;
use rosetta_core::crypto::{SecretKey, Signature};
use rosetta_core::TransactionBuilder;
use subxt::config::polkadot::PolkadotExtrinsicParams;
use subxt::config::{ExtrinsicParams, PolkadotConfig, SubstrateConfig};
use subxt::tx::StaticTxPayload;
use subxt::utils::{AccountId32, MultiAddress};
use subxt::{OfflineClient, OnlineClient};

pub struct PolkadotTransactionBuilder {
    client: OfflineClient<PolkadotConfig>,
}

impl PolkadotTransactionBuilder {
    pub async fn new(addr: &str) -> Result<Self> {
        let client = OnlineClient::<PolkadotConfig>::from_url(format!("ws://{}", addr))
            .await?
            .offline();
        Ok(Self { client })
    }
}

impl TransactionBuilder for PolkadotTransactionBuilder {
    fn transfer(
        &self,
        address: &Address,
        amount: u128,
        metadata: &serde_json::Value,
    ) -> Result<Vec<u8>> {
        let address: AccountId32 = address
            .address()
            .parse()
            .map_err(|err| anyhow::anyhow!("{}", err))
            .context("invalid address")?;
        let dest = MultiAddress::Id(address);

        #[derive(Debug, Decode, Encode)]
        struct Transfer {
            pub dest: MultiAddress<AccountId32, u32>,
            #[codec(compact)]
            pub amount: u128,
        }

        let call_hash = self.client.metadata().call_hash("Balances", "transfer")?;
        let payload =
            StaticTxPayload::new("Balances", "transfer", Transfer { dest, amount }, call_hash);
        let call_data = self.client.tx().call_data(&payload)?;

        let additional_params = {
            let runtime = self.client.runtime_version();
            let nonce = metadata["account_nonce"]
                .as_u64()
                .unwrap()
                .try_into()
                .unwrap();
            PolkadotExtrinsicParams::<SubstrateConfig>::new(
                runtime.spec_version,
                runtime.transaction_version,
                nonce,
                self.client.genesis_hash(),
                Default::default(),
            )
        };

        let mut transaction = vec![];
        call_data.encode_to(&mut transaction);
        additional_params.encode_extra_to(&mut transaction);
        additional_params.encode_additional_to(&mut transaction);
        Ok(transaction)
    }

    fn sign(&self, secret_key: &SecretKey, payload: &[u8]) -> Signature {
        if payload.len() > 256 {
            let hash = blake2b_simd::blake2b(payload);
            secret_key.sign(hash.as_bytes(), "substrate")
        } else {
            secret_key.sign(payload, "substrate")
        }
    }
}
