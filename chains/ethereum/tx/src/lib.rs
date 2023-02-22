use anyhow::{anyhow, Result};
use ethers::abi::token::{LenientTokenizer, Tokenizer};
use ethers::abi::{Abi, Function, HumanReadableParser, Param, Token};
use ethers_core::types::{Eip1559TransactionRequest, Signature, H160, U256};
use rosetta_config_ethereum::{EthereumMetadata, EthereumMetadataParams};
use rosetta_core::crypto::address::Address;
use rosetta_core::crypto::SecretKey;
use rosetta_core::{BlockchainConfig, TransactionBuilder};
use serde_json::Value;
use sha3::{Digest, Keccak256};

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
            data: vec![],
        })
    }

    fn method_call(&self, address: &Address, params: &Value) -> Result<Self::MetadataParams> {
        let destination: H160 = address.address().parse()?;

        let method_str = params["method_signature"]
            .as_str()
            .ok_or(anyhow!("Method signature not found"))?;
        let function_params = params["params"]
            .as_array()
            .ok_or(anyhow!("Params not found"))?;

        let function = parse_method(method_str)?;

        let tokens = tokenize_params(function_params, &function.inputs);

        let bytes: Vec<u8> = function.encode_input(&tokens).map(Into::into)?;

        Ok(EthereumMetadataParams {
            destination: destination.0.to_vec(),
            amount: [0, 0, 0, 0],
            data: bytes,
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
        let to = H160::from_slice(&metadata_params.destination);
        let tx = Eip1559TransactionRequest {
            from: Some(from),
            to: Some(to.into()),
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

fn parse_method(method: &str) -> Result<Function> {
    let parse_result = HumanReadableParser::parse_function(method);
    if parse_result.is_ok() {
        parse_result.map_err(|e| anyhow!(e))
    } else {
        let json_parse: Result<Abi, serde_json::Error> =
            if !(method.starts_with('[') && method.ends_with(']')) {
                let abi_str = format!("[{method}]");
                serde_json::from_str(&abi_str)
            } else {
                serde_json::from_str(method)
            };
        let abi: Abi = json_parse.unwrap();
        let (_, functions): (&String, &Vec<Function>) = abi.functions.iter().next().unwrap();
        let function: Function = functions.get(0).unwrap().clone();
        Ok(function)
    }
}

fn tokenize_params(values: &[Value], inputs: &[Param]) -> Vec<Token> {
    let value_strings: Vec<String> = values.iter().map(|v| v.as_str().unwrap().into()).collect();

    inputs
        .iter()
        .zip(value_strings.iter())
        .map(|(param, arg)| LenientTokenizer::tokenize(&param.kind, arg).unwrap())
        .collect()
}
