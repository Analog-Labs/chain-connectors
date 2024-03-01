use anyhow::Result;
use ethers_core::types::{
    transaction::eip2930::AccessList, Eip1559TransactionRequest, NameOrAddress, Signature, H160,
};
use rosetta_config_ethereum::{EthereumMetadata, EthereumMetadataParams};
use rosetta_core::{
    crypto::{address::Address, SecretKey},
    BlockchainConfig, TransactionBuilder,
};
use sha3::{Digest, Keccak256};

pub use ethers_core::types::U256;

#[derive(Default)]
pub struct EthereumTransactionBuilder;

impl TransactionBuilder for EthereumTransactionBuilder {
    type MetadataParams = EthereumMetadataParams;
    type Metadata = EthereumMetadata;

    fn transfer(&self, address: &Address, amount: u128) -> Result<Self::MetadataParams> {
        let destination: H160 = address.address().parse()?;
        let amount: U256 = amount.into();
        Ok(EthereumMetadataParams {
            destination: destination.0.to_vec(),
            amount: amount.0,
            data: Vec::new(),
            nonce: None,
            gas_limit: None,
            gas_price: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
        })
    }

    fn method_call(
        &self,
        contract: &[u8; 20],
        data: &[u8],
        amount: u128,
    ) -> Result<Self::MetadataParams> {
        let destination = H160::from_slice(contract);
        let amount: U256 = amount.into();
        Ok(EthereumMetadataParams {
            destination: destination.0.to_vec(),
            amount: amount.0,
            data: data.to_vec(),
            nonce: None,
            gas_limit: None,
            gas_price: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
        })
    }

    fn deploy_contract(&self, contract_binary: Vec<u8>) -> Result<Self::MetadataParams> {
        Ok(EthereumMetadataParams {
            destination: vec![],
            amount: [0, 0, 0, 0],
            data: contract_binary,
            nonce: None,
            gas_limit: None,
            gas_price: None,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
        })
    }

    fn create_and_sign(
        &self,
        config: &BlockchainConfig,
        metadata_params: &Self::MetadataParams,
        metadata: &Self::Metadata,
        secret_key: &SecretKey,
    ) -> Vec<u8> {
        #[allow(clippy::unwrap_used)]
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
            access_list: AccessList::default(),
            max_priority_fee_per_gas: Some(U256(metadata.max_priority_fee_per_gas)),
            max_fee_per_gas: Some(U256(metadata.max_fee_per_gas)),
            chain_id: Some(metadata.chain_id.into()),
        };
        let mut hasher = Keccak256::new();
        hasher.update([0x02]);
        hasher.update(tx.rlp());
        let hash = hasher.finalize();
        #[allow(clippy::unwrap_used)]
        let signature = secret_key.sign_prehashed(&hash).unwrap().to_bytes();
        let rlp = tx.rlp_signed(&Signature {
            r: U256::from_big_endian(&signature[..32]),
            s: U256::from_big_endian(&signature[32..64]),
            v: u64::from(signature[64]),
        });
        let mut tx = Vec::with_capacity(rlp.len() + 1);
        tx.push(0x02);
        tx.extend(rlp);
        tx
    }
}
