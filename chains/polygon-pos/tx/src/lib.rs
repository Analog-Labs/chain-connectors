use anyhow::Result;
use ethabi::token::{LenientTokenizer, Tokenizer};
use ethers_core::abi::HumanReadableParser;
use ethers_core::types::{Eip1559TransactionRequest, NameOrAddress, Signature, H160};
use rosetta_config_polygon_pos::{PolygonMetadata, PolygonMetadataParams};
use rosetta_core::crypto::address::Address;
use rosetta_core::crypto::SecretKey;
use rosetta_core::{BlockchainConfig, TransactionBuilder};
use sha3::{Digest, Keccak256};

pub use ethers_core::types::U256;

#[derive(Default)]
pub struct PolygonPosTransactionBuilder;

impl TransactionBuilder for PolygonPosTransactionBuilder {
    type MetadataParams = PolygonMetadataParams;
    type Metadata = PolygonMetadata;

    fn transfer(&self, address: &Address, amount: u128) -> Result<Self::MetadataParams> {
        let destination: H160 = address.address().parse()?;
        let amount: U256 = amount.into();
        Ok(PolygonMetadataParams {
            destination: destination.0.to_vec(),
            amount: amount.0,
            data: vec![],
        })
    }

    fn method_call(
        &self,
        contract: &str,
        method: &str,
        params: &[String],
        amount: u128,
    ) -> Result<Self::MetadataParams> {
        let destination: H160 = contract.parse()?;
        let amount: U256 = amount.into();
        let function = HumanReadableParser::parse_function(method)?;
        let mut tokens = Vec::with_capacity(params.len());
        for (ty, arg) in function.inputs.iter().zip(params) {
            tokens.push(LenientTokenizer::tokenize(&ty.kind, arg)?);
        }
        let bytes = function.encode_input(&tokens)?;
        Ok(PolygonMetadataParams {
            destination: destination.0.to_vec(),
            amount: amount.0,
            data: bytes,
        })
    }

    fn deploy_contract(&self, contract_binary: Vec<u8>) -> Result<Self::MetadataParams> {
        Ok(PolygonMetadataParams {
            destination: vec![],
            amount: [0, 0, 0, 0],
            data: contract_binary,
        })
    }

    fn create_and_sign(
        &self,
        config: &BlockchainConfig,
        metadata_params: &Self::MetadataParams,
        metadata: &Self::Metadata,
        secret_key: &SecretKey,
    ) -> Vec<u8> {
        let from = secret_key
            .public_key()
            .to_address(config.address_format)
            .address()
            .parse()
            .unwrap();
        let to: Option<NameOrAddress> = if metadata_params.destination.len() >= 20 {
            Some(H160::from_slice(&metadata_params.destination).into())
        } else {
            None
        };
        let tx = Eip1559TransactionRequest {
            from: Some(from),
            to,
            gas: Some(U256(metadata.gas_limit)),
            value: Some(U256(metadata_params.amount)),
            data: Some(metadata_params.data.clone().into()),
            nonce: Some(metadata.nonce.into()),
            access_list: Default::default(),
            max_priority_fee_per_gas: Some(U256(metadata.max_priority_fee_per_gas)),
            max_fee_per_gas: Some(U256(metadata.max_fee_per_gas)),
            chain_id: Some(metadata.chain_id.into()),
        };
        let mut hasher = Keccak256::new();
        hasher.update([0x02]);
        hasher.update(tx.rlp());
        let hash = hasher.finalize();
        let signature = secret_key.sign_prehashed(&hash).unwrap().to_bytes();
        let rlp = tx.rlp_signed(&Signature {
            r: U256::from_big_endian(&signature[..32]),
            s: U256::from_big_endian(&signature[32..64]),
            v: signature[64] as _,
        });
        let mut tx = Vec::with_capacity(rlp.len() + 1);
        tx.push(0x02);
        tx.extend(rlp);
        tx
    }
}
