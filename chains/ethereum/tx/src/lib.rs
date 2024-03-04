use anyhow::Result;
use rosetta_config_ethereum::{
    ext::types::{
        crypto::{Keypair, Signer},
        transactions::Eip1559Transaction,
        AccessList, TransactionT, H160, U256,
    },
    EthereumMetadata, EthereumMetadataParams,
};
use rosetta_core::{
    crypto::{address::Address, SecretKey},
    BlockchainConfig, TransactionBuilder,
};

#[derive(Default)]
pub struct EthereumTransactionBuilder;

impl TransactionBuilder for EthereumTransactionBuilder {
    type MetadataParams = EthereumMetadataParams;
    type Metadata = EthereumMetadata;

    fn transfer(&self, address: &Address, amount: u128) -> Result<Self::MetadataParams> {
        let destination: H160 = address.address().parse()?;
        let amount: U256 = amount.into();
        Ok(EthereumMetadataParams {
            destination: Some(destination.0),
            amount: amount.0,
            data: Vec::new(),
            nonce: None,
            gas_limit: None,
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
            destination: Some(destination.0),
            amount: amount.0,
            data: data.to_vec(),
            nonce: None,
            gas_limit: None,
        })
    }

    fn deploy_contract(&self, contract_binary: Vec<u8>) -> Result<Self::MetadataParams> {
        Ok(EthereumMetadataParams {
            destination: None,
            amount: [0, 0, 0, 0],
            data: contract_binary,
            nonce: None,
            gas_limit: None,
        })
    }

    fn create_and_sign(
        &self,
        _config: &BlockchainConfig,
        metadata_params: &Self::MetadataParams,
        metadata: &Self::Metadata,
        secret_key: &SecretKey,
    ) -> Vec<u8> {
        let to = metadata_params.destination.map(H160);
        let tx = Eip1559Transaction {
            to,
            gas_limit: metadata.gas_limit,
            value: U256(metadata_params.amount),
            data: metadata_params.data.iter().collect(),
            nonce: metadata.nonce,
            access_list: AccessList::default(),
            max_priority_fee_per_gas: U256(metadata.max_priority_fee_per_gas),
            max_fee_per_gas: U256(metadata.max_fee_per_gas),
            chain_id: metadata.chain_id,
        };
        let sighash = tx.sighash();
        #[allow(clippy::expect_used)]
        let signature = {
            let keypair =
                Keypair::from_bytes(secret_key.to_bytes()).expect("the keypair is valid; qed");
            keypair
                .sign_prehash(sighash, Some(metadata.chain_id))
                .expect("the signature is valid; qed")
        };
        tx.encode(Some(&signature)).0.to_vec()
    }
}
