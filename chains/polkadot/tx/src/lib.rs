use anyhow::{Context, Result};
use parity_scale_codec::{Compact, Decode, Encode};
use rosetta_config_polkadot::{PolkadotMetadata, PolkadotMetadataParams};
use rosetta_core::crypto::address::Address;
use rosetta_core::crypto::SecretKey;
use rosetta_core::{BlockchainConfig, TransactionBuilder};
use sp_runtime::generic::Era;
use sp_runtime::{AccountId32, MultiAddress, MultiSignature};

#[derive(Default)]
pub struct PolkadotTransactionBuilder;

impl TransactionBuilder for PolkadotTransactionBuilder {
    type MetadataParams = PolkadotMetadataParams;
    type Metadata = PolkadotMetadata;

    fn transfer(&self, address: &Address, amount: u128) -> Result<Self::MetadataParams> {
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
        Ok(PolkadotMetadataParams {
            pallet_name: "Balances".into(),
            call_name: "transfer".into(),
            call_args: Transfer { dest, amount }.encode(),
        })
    }

    fn create_and_sign(
        &self,
        config: &BlockchainConfig,
        metadata_params: &Self::MetadataParams,
        metadata: &Self::Metadata,
        secret_key: &SecretKey,
    ) -> Vec<u8> {
        let address: AccountId32 = secret_key
            .public_key()
            .to_address(config.address_format)
            .address()
            .parse()
            .expect("valid address");
        let address: MultiAddress<AccountId32, u32> = MultiAddress::Id(address);
        let extra_parameters = (
            Era::Immortal,
            Compact(metadata.nonce as u64),
            // plain tip
            Compact(0u128),
        );
        let additional_parameters = (
            metadata.spec_version,
            metadata.transaction_version,
            metadata.genesis_hash,
            metadata.genesis_hash,
        );

        // construct payload
        let mut payload = vec![];
        metadata.pallet_index.encode_to(&mut payload);
        metadata.call_index.encode_to(&mut payload);
        payload.extend(&metadata_params.call_args);
        extra_parameters.encode_to(&mut payload);
        additional_parameters.encode_to(&mut payload);

        // sign payload
        let signature = if payload.len() > 256 {
            let hash = blake2b_simd::blake2b(&payload);
            secret_key.sign(hash.as_bytes(), "substrate")
        } else {
            secret_key.sign(&payload, "substrate")
        };
        let signature = sp_core::sr25519::Signature::try_from(signature.to_bytes().as_slice())
            .expect("valid signature");
        let signature = MultiSignature::Sr25519(signature);

        // encode transaction
        let mut encoded = vec![];
        // "is signed" + transaction protocol version (4)
        (0b10000000 + 4u8).encode_to(&mut encoded);
        // from address for signature
        address.encode_to(&mut encoded);
        // signature encode pending to vector
        signature.encode_to(&mut encoded);
        // attach custom extra params
        extra_parameters.encode_to(&mut encoded);
        // and now, call data
        metadata.pallet_index.encode_to(&mut encoded);
        metadata.call_index.encode_to(&mut encoded);
        encoded.extend(&metadata_params.call_args);

        // now, prefix byte length:
        let len = Compact(encoded.len() as u32);
        let mut transaction = vec![];
        len.encode_to(&mut transaction);
        transaction.extend(encoded);
        transaction
    }
}
