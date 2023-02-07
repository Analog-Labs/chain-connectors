use anyhow::{Context, Result};
use parity_scale_codec::{Compact, Decode, Encode};
use rosetta_config_polkadot::{PolkadotMetadata, PolkadotMetadataParams};
use rosetta_core::crypto::address::Address;
use rosetta_core::crypto::{SecretKey, Signature};
use rosetta_core::TransactionBuilder;
use sp_runtime::generic::Era;
use sp_runtime::{AccountId32, MultiAddress};

pub struct PolkadotTransactionBuilder;

impl TransactionBuilder for PolkadotTransactionBuilder {
    type MetadataParams = PolkadotMetadataParams;
    type Metadata = PolkadotMetadata;

    fn transfer_params(&self) -> Self::MetadataParams {
        PolkadotMetadataParams {
            pallet_name: "Balances".into(),
            call_name: "transfer".into(),
        }
    }

    fn transfer(
        &self,
        address: &Address,
        amount: u128,
        metadata: &Self::Metadata,
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
        let call_args = Transfer { dest, amount };

        let mut transaction = vec![];
        metadata.pallet_index.encode_to(&mut transaction);
        metadata.call_index.encode_to(&mut transaction);
        call_args.encode_to(&mut transaction);
        // extra parameters
        (
            Era::Immortal,
            Compact(metadata.nonce as u64),
            // plain tip
            Compact(0u128),
        )
            .encode_to(&mut transaction);
        // additional parameters
        (
            metadata.spec_version,
            metadata.transaction_version,
            metadata.genesis_hash,
            metadata.genesis_hash,
        )
            .encode_to(&mut transaction);
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
