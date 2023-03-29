use anyhow::{bail, Context, Result};
use parity_scale_codec::{Compact, Decode, Encode};
use rosetta_config_polkadot::{PolkadotMetadata, PolkadotMetadataParams};
use rosetta_core::crypto::address::Address;
use rosetta_core::crypto::SecretKey;
use rosetta_core::{BlockchainConfig, TransactionBuilder};

#[derive(Debug, Decode, Encode)]
struct AccountId32([u8; 32]);

#[derive(Debug, Decode, Encode)]
enum MultiAddress {
    Id(AccountId32),
}

#[derive(Encode)]
enum MultiSignature {
    #[allow(unused)]
    Ed25519([u8; 64]),
    Sr25519([u8; 64]),
}

#[derive(Encode)]
enum Era {
    Immortal,
}

fn parse_address(address: &Address) -> Result<AccountId32> {
    const CHECKSUM_LEN: usize = 2;
    let body_len = 32;

    let data = bs58::decode(address.address()).into_vec()?;
    if data.len() < 2 {
        anyhow::bail!("ss58: bad length");
    }
    let (prefix_len, _ident) = match data[0] {
        0..=63 => (1, data[0] as u16),
        64..=127 => {
            // weird bit manipulation owing to the combination of LE encoding and missing two
            // bits from the left.
            // d[0] d[1] are: 01aaaaaa bbcccccc
            // they make the LE-encoded 16-bit value: aaaaaabb 00cccccc
            // so the lower byte is formed of aaaaaabb and the higher byte is 00cccccc
            let lower = (data[0] << 2) | (data[1] >> 6);
            let upper = data[1] & 0b00111111;
            (2, (lower as u16) | ((upper as u16) << 8))
        }
        _ => anyhow::bail!("ss58: invalid prefix"),
    };
    if data.len() != prefix_len + body_len + CHECKSUM_LEN {
        anyhow::bail!("ss58: bad length");
    }
    //let format = ident.into();
    //if !Self::format_is_allowed(format) {
    //    anyhow::bail!("ss58: format not allowed");
    //}

    let hash = ss58hash(&data[0..body_len + prefix_len]);
    let checksum = &hash.as_bytes()[0..CHECKSUM_LEN];
    if data[body_len + prefix_len..body_len + prefix_len + CHECKSUM_LEN] != *checksum {
        // Invalid checksum.
        anyhow::bail!("invalid checksum");
    }

    let result = data[prefix_len..body_len + prefix_len]
        .try_into()
        .context("ss58: bad length")?;
    Ok(AccountId32(result))
}

fn ss58hash(data: &[u8]) -> blake2_rfc::blake2b::Blake2bResult {
    let mut context = blake2_rfc::blake2b::Blake2b::new(64);
    context.update(b"SS58PRE");
    context.update(data);
    context.finalize()
}

#[derive(Default)]
pub struct PolkadotTransactionBuilder;

impl TransactionBuilder for PolkadotTransactionBuilder {
    type MetadataParams = PolkadotMetadataParams;
    type Metadata = PolkadotMetadata;

    fn transfer(&self, address: &Address, amount: u128) -> Result<Self::MetadataParams> {
        let address: AccountId32 = parse_address(address)?;
        let dest = MultiAddress::Id(address);
        #[derive(Debug, Decode, Encode)]
        struct Transfer {
            pub dest: MultiAddress,
            #[codec(compact)]
            pub amount: u128,
        }
        Ok(PolkadotMetadataParams {
            pallet_name: "Balances".into(),
            call_name: "transfer".into(),
            call_args: Transfer { dest, amount }.encode(),
        })
    }

    fn method_call(
        &self,
        _module: &str,
        _method: &str,
        _params: &serde_json::Value,
    ) -> Result<Self::MetadataParams> {
        bail!("Not Implemented")
    }

    fn create_and_sign(
        &self,
        _config: &BlockchainConfig,
        metadata_params: &Self::MetadataParams,
        metadata: &Self::Metadata,
        secret_key: &SecretKey,
    ) -> Vec<u8> {
        let address = AccountId32(secret_key.public_key().to_bytes().try_into().unwrap());
        let address = MultiAddress::Id(address);
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
            let hash = blake2_rfc::blake2b::blake2b(64, &[], &payload);
            secret_key.sign(hash.as_bytes(), "substrate")
        } else {
            secret_key.sign(&payload, "substrate")
        };
        let signature =
            MultiSignature::Sr25519(signature.to_bytes().as_slice().try_into().unwrap());

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

    fn deploy_contract(&self, _contract_binary: Vec<u8>) -> Result<Self::MetadataParams> {
        bail!("Not Implemented")
    }
}
