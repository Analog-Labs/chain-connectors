pub use ethereum_types;
use ethereum_types::{Address, Bloom, H256, U256};

#[cfg(feature = "serde")]
use crate::serde_utils::{bytes_to_hex, uint_to_hex};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EthereumMetadataParams {
    #[cfg_attr(feature = "serde", serde(with = "bytes_to_hex"))]
    pub destination: Vec<u8>,
    pub amount: [u64; 4],
    #[cfg_attr(feature = "serde", serde(with = "bytes_to_hex"))]
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct EthereumMetadata {
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub chain_id: u64,
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub nonce: u64,
    pub max_priority_fee_per_gas: [u64; 4],
    pub max_fee_per_gas: [u64; 4],
    pub gas_limit: [u64; 4],
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
pub enum AtBlock {
    #[default]
    Latest,
    Hash(H256),
    Number(u64),
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AtBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::str::FromStr;

        let s: String = serde::Deserialize::deserialize(deserializer)?;
        if s == "latest" {
            return Ok(Self::Latest);
        }

        if let Some(hexdecimal) = s.strip_prefix("0x") {
            if s.len() == 66 {
                let hash = H256::from_str(hexdecimal).map_err(serde::de::Error::custom)?;
                Ok(Self::Hash(hash))
            } else if s.len() > 2 {
                let number =
                    u64::from_str_radix(hexdecimal, 16).map_err(serde::de::Error::custom)?;
                Ok(Self::Number(number))
            } else {
                Ok(Self::Number(0))
            }
        } else {
            let number = s.parse::<u64>().map_err(serde::de::Error::custom)?;
            Ok(Self::Number(number))
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for AtBlock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::Latest => serializer.serialize_str("latest"),
            Self::Hash(hash) => <H256 as serde::Serialize>::serialize(hash, serializer),
            Self::Number(number) => uint_to_hex::serialize(number, serializer),
        }
    }
}

///·Returns·the·balance·of·the·account·of·given·address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetBalance {
    /// Account address
    pub address: Address,
    /// Balance at the block
    pub block: AtBlock,
}

/// Executes a new message call immediately without creating a transaction on the blockchain.
#[derive(Clone, Default, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallContract {
    /// The address the transaction is sent from.
    pub from: Option<Address>,
    /// The address the transaction is directed to.
    pub to: Address,
    /// Integer of the value sent with this transaction.
    pub value: U256,
    /// Hash of the method signature and encoded parameters.
    #[cfg_attr(feature = "serde", serde(with = "bytes_to_hex"))]
    pub data: Vec<u8>,
    /// Call at block
    pub block: AtBlock,
}

/// Returns the account and storage values of the specified account including the Merkle-proof.
/// This call can be used to verify that the data you are pulling from is not tampered with.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct GetTransactionReceipt {
    pub tx_hash: H256,
}

/// Returns the value from a storage position at a given address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetStorageAt {
    /// Account address
    pub address: Address,
    /// integer of the position in the storage.
    pub at: H256,
    /// Storage at the block
    pub block: AtBlock,
}

/// Returns the account and storage values, including the Merkle proof, of the specified
/// account.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct GetProof {
    pub account: Address,
    pub storage_keys: Vec<H256>,
    pub block: AtBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "method", content = "params")
)]
pub enum Query {
    /// Returns the balance of the account of given address.
    #[cfg_attr(feature = "serde", serde(rename = "eth_getBalance"))]
    GetBalance(GetBalance),
    /// Returns the value from a storage position at a given address.
    #[cfg_attr(feature = "serde", serde(rename = "eth_getStorageAt"))]
    GetStorageAt(GetStorageAt),
    /// Returns the receipt of a transaction by transaction hash.
    #[cfg_attr(feature = "serde", serde(rename = "eth_getTransactionReceipt"))]
    GetTransactionReceipt(GetTransactionReceipt),
    /// Executes a new message call immediately without creating a transaction on the block
    /// chain.
    #[cfg_attr(feature = "serde", serde(rename = "eth_call"))]
    CallContract(CallContract),
    /// Returns the account and storage values of the specified account including the
    /// Merkle-proof. This call can be used to verify that the data you are pulling
    /// from is not tampered with.
    #[cfg_attr(feature = "serde", serde(rename = "eth_getProof"))]
    GetProof(GetProof),
    /// Returns the currently configured chain ID, a value used in replay-protected transaction
    /// signing as introduced by EIP-155
    #[cfg_attr(feature = "serde", serde(rename = "eth_chainId"))]
    ChainId,
}

/// The result of contract call execution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "status", content = "data")
)]
pub enum CallResult {
    /// Call executed succesfully
    #[cfg_attr(feature = "serde", serde(with = "bytes_to_hex", rename = "success"))]
    Success(Vec<u8>),
    /// Call reverted with message
    #[cfg_attr(feature = "serde", serde(with = "bytes_to_hex", rename = "revert"))]
    Revert(Vec<u8>),
    /// normal EVM error.
    #[cfg_attr(feature = "serde", serde(rename = "error"))]
    Error,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "method", content = "result")
)]
pub enum QueryResult {
    /// Returns the balance of the account of given address.
    #[cfg_attr(feature = "serde", serde(rename = "eth_getBalance"))]
    GetBalance(U256),
    /// Returns the value from a storage position at a given address.
    #[cfg_attr(feature = "serde", serde(rename = "eth_getStorageAt"))]
    GetStorageAt(H256),
    /// Returns the receipt of a transaction by transaction hash.
    #[cfg_attr(feature = "serde", serde(rename = "eth_getTransactionReceipt"))]
    GetTransactionReceipt(Option<TransactionReceipt>),
    /// Executes a new message call immediately without creating a transaction on the block
    /// chain.
    #[cfg_attr(feature = "serde", serde(rename = "eth_call"))]
    CallContract(CallResult),
    /// Returns the account and storage values of the specified account including the
    /// Merkle-proof. This call can be used to verify that the data you are pulling
    /// from is not tampered with.
    #[cfg_attr(feature = "serde", serde(rename = "eth_getProof"))]
    GetProof(EIP1186ProofResponse),
    /// Returns the account and storage values of the specified account including the
    /// Merkle-proof. This call can be used to verify that the data you are pulling
    /// from is not tampered with.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex", rename = "eth_chainId"))]
    ChainId(u64),
}

/// A log produced by a transaction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Log {
    /// H160. the contract that emitted the log
    pub address: Address,

    /// topics: Array of 0 to 4 32 Bytes of indexed log arguments.
    /// (In solidity: The first topic is the hash of the signature of the event
    /// (e.g. `Deposit(address,bytes32,uint256)`), except you declared the event
    /// with the anonymous specifier.)
    pub topics: Vec<H256>,

    /// Data
    #[cfg_attr(feature = "serde", serde(with = "bytes_to_hex"))]
    pub data: Vec<u8>,

    /// Block Hash
    pub block_hash: Option<H256>,

    /// Block Number
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_to_hex")
    )]
    pub block_number: Option<u64>,

    /// Transaction Hash
    pub transaction_hash: Option<H256>,

    /// Transaction Index
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_to_hex")
    )]
    pub transaction_index: Option<u64>,

    /// Integer of the log index position in the block. None if it's a pending log.
    pub log_index: Option<U256>,

    /// Integer of the transactions index position log was created from.
    /// None when it's a pending log.
    pub transaction_log_index: Option<U256>,

    /// Log Type
    pub log_type: Option<String>,

    /// True when the log was removed, due to a chain reorganization.
    /// false if it's a valid log.
    pub removed: Option<bool>,
}

/// "Receipt" of an executed transaction: details of its execution.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct TransactionReceipt {
    /// Transaction hash.
    pub transaction_hash: H256,

    /// Index within the block.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub transaction_index: u64,

    /// Hash of the block this transaction was included within.
    pub block_hash: Option<H256>,

    /// Number of the block this transaction was included within.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", with = "uint_to_hex",)
    )]
    pub block_number: Option<u64>,

    /// address of the sender.
    pub from: Address,

    // address of the receiver. null when its a contract creation transaction.
    pub to: Option<Address>,

    /// Cumulative gas used within the block after this was executed.
    pub cumulative_gas_used: U256,

    /// Gas used by this transaction alone.
    ///
    /// Gas used is `None` if the the client is running in light client mode.
    pub gas_used: Option<U256>,

    /// Contract address created, or `None` if not a deployment.
    pub contract_address: Option<Address>,

    /// Logs generated within this transaction.
    pub logs: Vec<Log>,

    /// Status: either 1 (success) or 0 (failure). Only present after activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    #[cfg_attr(
        feature = "serde",
        serde(rename = "status", skip_serializing_if = "Option::is_none", with = "uint_to_hex",)
    )]
    pub status_code: Option<u64>,

    /// State root. Only present before activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub state_root: Option<H256>,

    /// Logs bloom
    pub logs_bloom: Bloom,

    /// The price paid post-execution by the transaction (i.e. base fee + priority fee).
    /// Both fields in 1559-style transactions are *maximums* (max fee + max priority fee), the
    /// amount that's actually paid by users can only be determined post-execution
    pub effective_gas_price: Option<U256>,

    /// EIP-2718 transaction type
    #[cfg_attr(
        feature = "serde",
        serde(rename = "type", skip_serializing_if = "Option::is_none", with = "uint_to_hex",)
    )]
    pub transaction_type: Option<u64>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageProof {
    pub key: H256,
    #[cfg_attr(feature = "serde", serde(with = "bytes_to_hex"))]
    pub proof: Vec<Vec<u8>>,
    pub value: U256,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct EIP1186ProofResponse {
    pub address: Address,
    pub balance: U256,
    pub code_hash: H256,
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub nonce: u64,
    pub storage_hash: H256,
    #[cfg_attr(feature = "serde", serde(with = "bytes_to_hex"))]
    pub account_proof: Vec<Vec<u8>>,
    pub storage_proof: Vec<StorageProof>,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use hex_literal::hex;
    use serde_json::json;
    use std::str::FromStr;

    use super::{AtBlock, CallResult, EIP1186ProofResponse, StorageProof};
    use ethereum_types::{Address, H256, U256};

    #[test]
    fn at_block_json_encode_works() {
        let tests = [
            (
                "\"0x0123456789012345678901234567890123456789012345678901234567891234\"",
                AtBlock::Hash(
                    H256::from_str(
                        "0123456789012345678901234567890123456789012345678901234567891234",
                    )
                    .unwrap(),
                ),
            ),
            ("\"latest\"", AtBlock::Latest),
            ("\"0xdeadbeef\"", AtBlock::Number(0xdead_beef)),
            ("\"0xffffffffffffffff\"", AtBlock::Number(0xffff_ffff_ffff_ffff)),
        ];
        for (expected_json, at_block) in tests {
            let actual_json = serde_json::to_string(&at_block).unwrap();
            assert_eq!(actual_json, expected_json);
            let decoded = serde_json::from_str::<AtBlock>(expected_json).unwrap();
            assert_eq!(decoded, at_block);
        }
    }

    #[test]
    fn can_encode_decode_call_result() {
        let json = json!({
            "status": "success",
            "data": "0x0123456789012345678901234567890123456789012345678901234567891234",
        });
        let expected = CallResult::Success(
            hex!("0123456789012345678901234567890123456789012345678901234567891234").to_vec(),
        );
        let actual = serde_json::from_value::<CallResult>(json.clone()).unwrap();
        assert_eq!(actual, expected);

        let encoded = serde_json::to_value(&actual).unwrap();
        assert_eq!(encoded, json);
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn can_encode_decode_eip1186_proof() {
        let json = json!({
            "address": "0xdac17f958d2ee523a2206206994597c13d831ec7",
            "accountProof": [
                "0xf90211a0f4e2cff38416963d94c95aca99bbec3e01a2625cff007beeaf4cdb7d8a515038a0ad190678a196b68cfe369210824a3fcdcde38302a27c8a089133723b89f9093ea03b085892d6da74a171484a34de93e3e65a9ac4c0b1ca6f88dfcfba8b0d8d2510a09672de1a9c0a380d80baa2bf6bcd05c5cb56eb865f712d597d12940576025a0da0a4a3f0024f8617ee242bbca2a16909f1f909d5a4ad97de91f60da2dc859e1f1ea0ef734ed7c209a02a7204e5895dabddaaf90fd1ba7fe987bf81ad0fb3dc04bd8ca0b69ab5fce31a80eec450d3f6dd6b45b2cc9d5220c82a6906f76cffeaabdbed22a0d227b181bf676d7eb5248156d3c75e8677fc4dfbe0ba73fcdc98259000f7cfc7a0096fdecfdf7f656ba5c9bb425e20aa6d6b21a6bab5b95f69c9a7f6e2f5135bbfa074671ee87f7f86716445dbbf1dbf8e113ba1c2c905dffebafaab43e674e3ba64a01c4b5eed71721e1e9f52d6e9987a86abd7e10e8b85509950992f12645b539b56a01997ff0652394c5f72e52db8c82330e5cf65df72f3a208344bf79b845a2601b6a08c9596b489fa9afd42900c865891e6e3a4274e376dffd121aca657fc99b64126a02cd49a7b8c05585c0c9dd368b5c2d6397bbcad6e3de653c02a68def7d4ba3e2fa05ad2400ea5f646fb7c9dc5c32fe60a9265109d1d9c142777c5fcbf1f778911aaa05657c5a23bb4307ef473e9d1af3e4faac9c81a82a730c228406453521580204580",
                "0xf90211a05220309e422aa851b9756aeb2a3bb7c821a666982310d1fb4f97bf117e821249a051965b23e120060dfa570686728db89c332f3a99e41703b96b8cf42e393f1cd8a0050c34a3b3115c8d2c430785c2f27976dd07a2daaa099a9dd0acc407b61af86da0ca09ba90042485a211eac80f6efcacba26777b3f235cf0226b23033d9e25b123a0f1a8baef6a03814c83575294425e6d9e04db1377b5235eda9710737975715597a0613ca0a9a5fd7a08b1c3c9ab30eb0b6725b53a65e1d39c6b83fe27513f807205a0fcc7c0bacd270cff6c47b6e8c202419e2255a45d1ba0d35ca7d68143ed7c7178a0be159bf599ea67ad2120a23428e808e45cd83909f87a783eda148c09b04f970fa028aff3c52064443f63ad0fefb9b203e7783a333460e3d62b857860524e23110ba015178d2d6231ae38cd90b62123444043e2dedfaa25dbca161e94eef8458eb4dba0df221a385a3d7a0966017fc1ce568ec4f8c360c79e99bf2bf2500c85f03e6b25a0a00cbaa7b1006ab375dcc118e0975dcf4c56de4514ec4770b76f2f95d2a71202a017dff70ca9e1410b4d619e19b49fb697dd5e5c58ca7a00dd7adc441965b5d422a0f2a67c1d3db23cc7791fb3c1cd9bcbbee2ed0f77d9be4e2fdc3ea5eb05539771a05d6f95e2bafb1b11102089264f2011262bffcc59543d3bb2b932a731c4647044a072b985603ef98a8e49c7f0c52e2cf2a213597196187735f424aba8c73ab3594b80",
                "0xf90211a0269272e950b4953aadcb8f24af852ef8b3be49924a3b69d7f52f6e98bd59a849a07a4146636b6d2d8bc3adf4ee8aa1d21cc66bf22221eba0837b95c62fea417a4da03e08c7275d9ab95d0b480c2735f5395d22f737318077e699de74e92d3072789fa093c020e64b492c18160066e6288b806e628a0066a812daf18a653e87fb8e9258a06e4d5b58166fa1c79d3b093dc74e6e6093dda276eccb20b4fde52cefb426eebea0356e28b87883e2268a9e4fca3d3e3f654776cf4b2c5f79fa7ae84f724d3a197ca06f716978b17b96e43dc2e085fd18cca8bdc8b576c0a2b22da821379d49ea0e7fa0177c98e2d3516dc12ba987d300cd248af032fe5136fb29e6155223982217bb79a008b522ff473da62e00ed8afddfc336c58dbff52799c4ec4afacf3603d7e61f16a01caadb1c046aaa477fbc1fad08e975fa93b1ba44f2166c6094c75cdcdc5dd52fa077a9d714db981b501fb7ca3c4c151ac712696334d45db57f1054ea84ad27daaaa017de69774c53d8f507d312c9594f13f03ce52d7fd8d706b138752f15eec14fc3a0fe9c84acdc43360e1f82022285cf9bad1198eeefa1aa82860af333d9191be851a02676aa89c3fe2af88468586e6aebc1113db1f0ab2d46f7b5be9169fcb03e538aa005de087a009e870b948f21ec5b816a1fffd77bd6045b6e984a54687e0404859da0b41409a679b7530d09b48a8be995368f9190c6654a039958292872d6e30e502a80",
                "0xf90211a065120a944f3ac11fd7a4f00a33ee69597b27960d4faf08488c095a54bf57171fa04364126d60c023dc29c4ece6ca7ad2b995b5661b20859faf904ea177ccdf5264a0863c7609c1a37e5f473a0e89a78a9f669336ce4a878b3229df5b776b15857003a0a1366941e9cdf61171f97d2909e999070dbb4f19d958be1303098f621c2d0cd3a0953c367aca82d2ffc36a0a38b089b4b4cae4421fe75b892bc8814c66c1f57f8fa02a5c39a5214f1af21675d8c8533e8309f271df89fe4ac6889b9ffdb7fcbafabba01cd4cc0ecf0609d17aba14998460c3ea7d5d087f6afa27e7f0f6938db6f4da48a0f0951d9162741ffdad9f88a12fe5adbd420cb4d51b5c86bb858ad9f6c663ca77a00a6ff6752d7fa2e2cf35f122f384fa81c3e31843208de33312a51daf8ec6f602a082dd49ddbf5fa63916480e19a6160b52eed3b528bf565d756338b71fcb40e226a04a04700f71e8beca15df068d2673acd9320057ddf9e944306c1d0f30fe081da3a0df3445cd82e2c301325854af499753ca70c993355ef316f501330c0685ee06b8a0cf7b31eb4ce89e2086ed15849e0dd1cebedb335fed1a3253a76abf4c4403600da0581f31842e2ba26a16ca6ba7ff199f2ad7b1372bd189e59bc75dbfb3cf328c8ba02112f5d832f7f8decaa0bb91972f6c0c4bfbd3c8eb4715e8c575672f52fa4bfba0ac29d3bfe5bab2855854beda7617d3c36c32bdd34cc1b4eac16064d824fcb93c80",
                "0xf90211a08cfbe475652906bec65ab34a0aafe3e108d8a3814302481d0702ff70e8b1ec7da0bb52733912f93ef21f2ad6d053baee454028f6f24c7b262e97e15b69fd838f29a01790de8c1b55778ff8792468918bdca8c080401727f1e7dfb1996fb817d4a5b2a0fa5bec47b083ad118ec0bedef4060ed5ff1eec1532f3b91da3fbf23488f3ffe9a0578b91094da870fc1730f276c74f612d65581df80433be59118132c97e5a90b1a0ca767563a6658d7b6a4840c3e1e3d65a98a6adb3952244c2def748be30c81d20a062260ad247f062a42f22076c3d8ebfdbfaf376048d67ace5e958b21b6ffa7ffda0a46528927602d743309235d3a50b9d42975048db410ca5deff039c5045a1bf6ea0fbdd64c68af03cde60a403ae7d1b08a0bc89b36bef3a2bc1b44b36316ecc7fdea0593b5de878ae20b1a14b4f7c63f1e64060f1e6bad4baa5704d28528fa9b3733ea0537aa00fbb93fbc8bf0d3043e44fbd6d227c648284e12157e6af533113d83e5ea085de7af69c1c38259bc36a5a93a8959b8d04856e6657fe528a775f2dd5e1be7ca00e2b391cd8c555e85e7406b21d853d18096be5a1f6f30399e0f569251f74d8f1a0b304bc6cbbcd5864c1338b29e7871f739976ebfa3734c04fad1288a10428352ca0fc83234d8f44b46c6a77c42f49cbbb62b5850c8274c3f33ac238d2966629aa47a0a30201dac0877046f23222c6e05cecda65a7f423e88946bcd099cc50446d865780",
                "0xf90211a0e31b4c19fcc4fcb854b8e624b318ad21b022c57c58e30f80f27bcd9fe4e61649a0950522557eb40bafbd082f0f5cc4be3bcdcb7f80c14eee43afd2bcd01f8d5137a006344fc5ae8c6063578d0b997b0caebc50abb303fc195a9875c55a5d3e7566b4a011a2f9312c3308640a0d6ceeae218747290f23806067456da1d444c65abae437a0b3097a108bfce79af6699da4ae3003cd4929f0b4576aad655c31cb725bde84c7a00975b2742460058745a4ee9f17d4c2cd50047b9831702204293a4648ea4e21d3a0b324be589fdd06f1d77158f23e1ce85661ed325abcd3c34aae53b2138db1f61aa02e7927d9c620923d4bace4882588753ce2bd16373ecfefe90b71a8973d8d6b15a09aa229b179290efb4a6b52687b928c0ab96962acf989d59de2fde5a1a657142ca0750babf0e184cfa59851d215e74e4a351fe5273adbce334c23a54a2a564c3a65a00b3757b624f3e65e3cadbd9b61e092af2f6086fe846ac6ed51a2f0261d21b475a0b7d528fc41c8fdc8ea18c6e7d0099270c777ec1403cf879d1f5134bdc12a6c6ca062a0b052298c12a244f472f1eec5f5d387da0760f654d348de7439855781b655a075fd0c30e17585c20d95627e7e9eb83bfc7b1be1a6ff69b80b1401b7ad393ba3a0c2aaa60bccbeb370c420a774007f7b35d86ae3e837309e12218664378c1573bea08bd2b242e992653fa60521d04209d0f948548de03ed9d063f6c847212da606f480",
                "0xf90191a00a7a0118e00981ab321049c9d340cd52c3a4781037540f7c48d0fdc27e899b3280a08537f2e248702a6ae2a57e9110a5740f5772c876389739ac90debd6a0692713ea00b3a26a05b5494fb3ff6f0b3897688a5581066b20b07ebab9252d169d928717fa0a9a54d84976d134d6dba06a65064c7f3a964a75947d452db6f6bb4b6c47b43aaa01e2a1ed3d1572b872bbf09ee44d2ed737da31f01de3c0f4b4e1f046740066461a064231d115790a3129ba68c7e94cb10bfb2b1fc3872f7738439b92510b06551bea0774a01a624cb14a50d17f2fe4b7ae6af8a67bbb029177ccc3dd729a734484d3ea04fb39e8b24158d822454a31e7c3898b1ad321f6c9d408e098a71ce0a5b5d82e2a0c8d71dd13d2806e2865a5c2cfa447f626471bf0b66182a8fd07230434e1cad2680a0e9864fdfaf3693b2602f56cd938ccd494b8634b1f91800ef02203a3609ca4c21a0c69d174ad6b6e58b0bd05914352839ec60915cd066dd2bee2a48016139687f21a0513dd5514fd6bad56871711441d38de2821cc6913cb192416b0385f025650731808080",
                "0xf8669d3802a763f7db875346d03fbf86f137de55814b191c069e721f47474733b846f8440101a0b1d979de742f9e49f16bc95d78216cba45ceb23eb94185d14552879ec1393568a0b44fb4e949d0f78f87f79ee46428f23a2a5713ce6fc6e0beb3dda78c2ac1ea55"
            ],
            "balance": "0x1",
            "codeHash": "0xb44fb4e949d0f78f87f79ee46428f23a2a5713ce6fc6e0beb3dda78c2ac1ea55",
            "nonce": "0x1",
            "storageHash": "0xb1d979de742f9e49f16bc95d78216cba45ceb23eb94185d14552879ec1393568",
            "storageProof": [
                {
                    "key": "0x61d6b5a02307fb160a7bed094f54b8182c2887c4349a7bcd4ac6936a4d620faa",
                    "value": "0x16bcc431cc060",
                    "proof": [
                        "0xf90211a07664e825351ee2559903e491b035d48baf90716617d9c771a51cb38a467fe9efa0aba0f80c633a27c16fb13a8ac92bd09cb27ba6c2d82a1a907c12942f79399a76a0204fd8890a2f789880340f79d12cd28a03060970a16442824c6132bc7351be4fa0c27158a8fa82e0aa7f15f847882ce06b5bd9aeceb7f7accb69c68c128f9a2d8ba002133406256ef781ad23ad7833e4e272e149bbec9704bcbbd6cc71126aa9bb95a05e5e108691679aa1b9fd69778871bf93339fe70dd80c090e6da9f361ad112ba8a05ff5593b34953ea40858ae61bcab89677656746f84d288ee490c57709a2f18a4a009ede6719eeb6aa535f05402a9fed93336e4095b177f70979d025b29978ea82ea06918f4c1f900d4ff9ade9968443af0a1fac265e7580bf49d246c63d5fef4b876a0d65743ac746b470a64b47e853ed82e3816f34bfaf6f86eda7e073f5fd5429d6ea01d1e8d2577abd72e0ccadaee3bc643f2529ce036bbab66e8e6fdb5c6db84149ca02153275cc8b8b89dbf25a854e2d4f604bc2701960f8ebcb1c19207b9fea1f168a03406e7c018f995f29bd344aad8db4d6d621c16583db4a7e8414a6cb3e2086a2ca070accfe4cbd95432dd5731643ee6672bd7abc01e0189265da32b2ea21bedaac8a00466ef35fc0960732dd08d0147bd5b4f606ec43115def41fa77cd4f31c2b6be1a002c62ef81f50e53fd3ec15e19fcece50775ca076657d42befcddab93fef0905880",
                        "0xf90211a0f9feb6d18df70a52b7faabaa486966e25e815348f89abb88a26613aee21f1728a0d38d07725cf4c0744f53fddf07d7d9d677caeffbc6d27c5cbc48de08a4cfabb0a0fbfac941f5550c295d2a4d0fb00fa12434861d74ed5aeddb69e6e20779a7bc7da062b62b461e9317bfcbceaf607daa4ed7afa52e98fbcef94436705c9d5314ba47a0c76acb6b4839d5330da1eff86496f157ef46d09e1aafbfbda0e3bc49abe7d1eea083de1c187d1e82bae56a8ee7670082a0c4955d62ea3ea4181ebbf4b4a9c54b06a0602a729afcb9b901520a956d859cb8b0b9425e4fc4950599e7c0a485eb4c9e12a0ac6e132e49c21d2d0f46ed19c95b87a78643f4b47c1cfcace74f7d32b43ff188a09450bd1f5fb22bd6bb1b3801a521a100cf5dd2b4999b8d8e318d65b5332102eba0cb6b312c970c010dccc6643b6178d1d594494acf837dafcddf2c170e98277afca0b9eb554af6dfec821b716df886741da322465540e042d1b624ced5830967a04ba052564e86b32298e801c7f183e730f6d4df783d14121cf2513c600403c24f46afa046a14e816a4a4d2f9dbc01d380d61103c74b542931eb03e42f33cd78640c9ae4a0d9fddeaad53578d7d9cc3e34624899ecea5522caf2e4d8ac94631a0370ec9904a0eb0ebc196c08dba1ad659293e61dee6240595752364cb0ba3c1cada9e9c3ecf1a01fb26ca7a5563ffcc47cd85d5dfcdb0055870f00111ff89c33273b263082d0a480",
                        "0xf90211a050334ca647f515eae4ffd1f372129684564627b0762da351cb64abfbebe5a2b4a077e12b72ffe91604d91440508cd23e1259e9bb430b35d6a140b5db236361e6e4a030204c15b02247347421af85be201e53f29998243a7cbf1d5b1c6841bd274000a04c196f18d0f7cd44b0f78459651388dd7580b5944ac81a6f0b5f8134fd5b9713a081ff955af7b8fd112203bb57eb57335eae4d83e0419dee5e00382b4b155c6bbca05b79b8ca9a6fde56e54b5f1cba081e6f113ec0e1aa5625c426aa014ec013604ba05b2183e90cbe7efdb2cc0568c5b1733d6f1e5ae7bd13482a7e4826d0150dececa0bb803ef214d38a3f89bfc388d9a58e1470fc6d2a0efa1a4a71363360b323d38ba0d1b50eb36cba91bb1585e9132fe5ea2e1d4cb2beb260357d668c2e37c4fe6f88a07faa02c13272dcf5993dea50c20ae65a205614fc9de37ae6f7646c59dd3d15fda00e43d223d9bec16152aba6e6d5cbf99d1c94cedc261e9cc451f19d8752f279cca0c2e27b9487115c7e7a454f2deb775ee914a5d57785ea3e594a6de1305c3c8419a08dbdc8c02cd069cdf1cae1b818bb23d9b1d1ef4244f78940befda9296b8492a1a07cbef9cb730ad97fee202fa060bb1e19ec132576ae6a120f52a9290bf5dbf56ea0bd4a3b04cf41493126e643cb73ab2bc42382af85b70eca8a2554336a92c19158a0cd2c5730c90df1c053c2d4d7ee1d67c803ec12774f4bf0a42c9f52c8fc56187680",
                        "0xf90211a09dc0c0b5ffdb00d49fd73c13d816943a4d0328f8a7e4200ac5fca94074fe37d9a0008201d0ddb8b50b7e8126098d66c6cc03104fd40877cc7be8964cb71a3f68c6a0a6a236a5055c42e21a8fc0ababdfe13b4cedd3308338b1695d02919eb97eae1ea03f7c7e823fc35d6d82566698a624ebf374535236da1d774e274b6a30f6b9f740a04575e395da64fd92771c0c16b968a6d66a1dd4b5ecb9f7ba7c14906039042618a0c4d521806c922a6c083789b34a023c11101282f3d9987caf7f442040610d2368a0093294883edcc6bf1cec4fef3102942b0747004caefc7006b2fb86249f8287a7a09ef9f0d36f206f959dfceb907fa3bd76c42a7b8f870eb4b938322d3f25e61ca4a076b8308d195f017747da24ebb46e5be13a7090f6b094a0399d5ff65eec302e98a0339ee7bce47d1211235244f0f05f3da6cbb856a25c9f006036c8d13dd4012958a0c28ca6cdb5ed3d19182a0e874844ab09eafc261a30c2db35aaffde7f1e8872eca0164dd52a75274e63da527ce01d22409bfa1c5cc4de7ffc3523fb08c3480cbce2a08113a1cbc5367389a60fba83040b2ca176a3d3b6fc93e04a4e724f9adeb66907a0e07e0b466f71aa1513c07a23e19f0475f255647655df79ff7cb3dae579208c35a0608f1b9b054948a48bf7cd317ff3708114dcaba99bc136569ea48bbea12cf6b9a06446e6a7fec8afc75cee385d181c721b466a32cd8555f4afe2a65ef8a837011a80",
                        "0xf90211a0b2c39dc101beebed6ed68f57c63d6f7333c38a70a2809ac29d9caad53b410957a0dc1571eaacadc4eea1a30e760a49267c7cf7945b09c3435e321bfbe8f71cbe19a0fdcde768c5738af5a7c71e65ef4d8b427e18e816657525bf7c38ea8bd5a29104a03a039a2bba043ce489d8c6145789c42121867739b9349f60769c96a41a465f6ba069cf2da149e40b92212b28ad8e031afb34a73942e4d9f861e40914b53abe4b67a05284d08672cdfa118396d99303c0a54f0b6d4a88672886ab9e8244cd7e23515ca0b56ab7578196f7a6d517519bfcf1899958ca97a590dc3de5d80ea8f34577d423a0f0eccb83567f2b4a21abc3ff39cefadef31994935a8ca707fefa1f4cc71f46e1a0a4b786589392014ca8d32425fa56e5f7b6da1d0dbf9a4c2fb98115cc6ccf15f0a0c455b01229b37964007474ae1398c877247275f1e7cff01203c60de253d44a3ba0b14d6ec313f1d918cbfc6ed5bfd65ec9436385084ef2aff37d3826403a68f789a02bad9e2afd24cb768b93e4790235b0ccfe35d15e84c8706bb22e0140eed0842ca033d9fb4d25f863f55f56dd500786060a3714a94e097e34ecde79412d80d25a80a02a57734da428a685a9f7339c60f508e5b6abe788b32ec3801afd1ffefbc25b01a04815c84b9eb7b1bd09d6ea7ca65c202d4bb96241cece38976a49011490f199ada0dc517014368265f40f1b4797f4060fc50f6a7d11c9ff270b3fb0fb63dba76ebd80",
                        "0xf8f1a0b84afd46c5ef7d58b7ec8ff4e54099bf9021e51c562bfc81a8291caa05e33db880a0ab5289bc9d06c3747d271ca8c0a2f6a482b7eb2b103d1ff6682ac5cf66685fff8080a05c67492e2a6961f8b702b3abb00694c775ae15c1c8ccfcc62f766c413c98bb1f80a05f4486c44fbe102c65b8a3e85bbb4833768739e55ba4e68286f8852f053afe688080a0dda7960d591160912efc067283517c2496c881fc12e2c8777233defd0bcef1fb8080a06a5a73737c8e13e0c062f175017894bcd744157a4a2f7353f3366b02a9715d3580a0ed913b44f62ac0485fdfb6daf7b45b95156daedbd54467f3c9b2e43bfcf63d4d80",
                        "0xf85180808080a09548469fab745c696f461425294816340edcd44fa9cf1d84c201342aa5b590c6a0e64144f645b0cd8966f9869bec53d27e74845b93d1226b617c4df287509074178080808080808080808080",
                        "0xe79d33878a1bf90b4a94e2dfcf7e46345d6c3f209bf31504ec1f349be19d2f8887016bcc431cc060"
                    ]
                },
                {
                    "key": "0x000000000000000000000000000000000000000000000000000000000000000a",
                    "value": "0x0",
                    "proof": [
                        "0xf90211a07664e825351ee2559903e491b035d48baf90716617d9c771a51cb38a467fe9efa0aba0f80c633a27c16fb13a8ac92bd09cb27ba6c2d82a1a907c12942f79399a76a0204fd8890a2f789880340f79d12cd28a03060970a16442824c6132bc7351be4fa0c27158a8fa82e0aa7f15f847882ce06b5bd9aeceb7f7accb69c68c128f9a2d8ba002133406256ef781ad23ad7833e4e272e149bbec9704bcbbd6cc71126aa9bb95a05e5e108691679aa1b9fd69778871bf93339fe70dd80c090e6da9f361ad112ba8a05ff5593b34953ea40858ae61bcab89677656746f84d288ee490c57709a2f18a4a009ede6719eeb6aa535f05402a9fed93336e4095b177f70979d025b29978ea82ea06918f4c1f900d4ff9ade9968443af0a1fac265e7580bf49d246c63d5fef4b876a0d65743ac746b470a64b47e853ed82e3816f34bfaf6f86eda7e073f5fd5429d6ea01d1e8d2577abd72e0ccadaee3bc643f2529ce036bbab66e8e6fdb5c6db84149ca02153275cc8b8b89dbf25a854e2d4f604bc2701960f8ebcb1c19207b9fea1f168a03406e7c018f995f29bd344aad8db4d6d621c16583db4a7e8414a6cb3e2086a2ca070accfe4cbd95432dd5731643ee6672bd7abc01e0189265da32b2ea21bedaac8a00466ef35fc0960732dd08d0147bd5b4f606ec43115def41fa77cd4f31c2b6be1a002c62ef81f50e53fd3ec15e19fcece50775ca076657d42befcddab93fef0905880",
                        "0xf90211a05f55533e0b43b528ea30aa81a8f70ba5c4f902cd4cd6c0dc44999e16462c9592a0c2922ce429e37b6d769f2305cacde39f777e7e90e8ce387959a8954591ad54c1a053e8e848d47002528949229d58c6f9c0672f88bc4d378cc1fe39c17b9bef86f8a0e7dc7b8aae803a5447a35f63ec2b2cebf49fee5489d5b9738e3d64c3156c6c2ba02dcd41fd90ebf9c932769c1fd7a284302bf05a25746530d45aedec98159faa4fa08529feaf44ddfa79256a045513d56cdcc17ad172704d2d651098c6c6cf643e53a00e8295e2ef6849a73a32c606ec0cf0d6e5f4cc826dbb26f41ca198792c4325a0a0341359e052289aa4bfc05791e28fe87440fe9a6cd2502aac70300f595da27bb2a0a8c574979a99f008a7b0fdb3f5d1aa5c81f75faf419f355c34f00bbae9e9674ea09bdfd474f8c35b5614b23311f65c3e1d107b7cb19120527ccb449a138954e375a02f656aecd41e6b13d93e7f074d2e47f10693c6a77bbda9b75557ae608395db31a0d56eed1e5a7bb97f42335a91cc773a2e9ff4c61cd02aab47c9ea3c21740559b8a096a539cc0c81fc92ebfd88262a35a243e01b61d0cb08cf30b3327d5b35426df6a0dd9a0d31ad7b4595a44af610170dfa44286f3dc9cbb93dcc5d3b1bd37ffa30c0a020b375a158b4a2f81182c9fbca3d6640455b549a404bc468d4e07f4e86dc150ba0461e6ae086898a77502b18cf1378eb5c59b5ec0a1311e3c97fa1d24a031c0b7f80",
                        "0xf90211a0e8c1590de40a9530e23414782dc039b15f3635be011b27883bc716ed12450cd4a0397d720c7c5460b2a0898e4d2d0e720607e0938572d971afdf140d745e989adea0f8a28e1f408ad7dee0a106762f44fd714fff00f7c0e9a03de6620de569222ad2a0967800891aa68296913852853498ea41c085b558677731a5af0b751a08e8e66fa098e1c0f92bb44553815db48ddad8fa3d932793fe136c9063b1459f64d106c598a0a1891322c34e83c223ce162139a98ffd771015c42ba858c71ecafb7b4c47f29ba072dffe10e2576b4b63ad0db97a75ce549d369c14de14a27377620ab4cd684171a0fe37330c7f1857fdf50a4dc14752512d3d7b00227164c5fac773984c94b7b1eaa0bbee82adbca630862a977a48ea7f569bb8fbd8318589b6944aff1f78b8176f0ba08f5abe4ae9d3790c1c3d0d84f01afd95d0862f14bca08150e82d304081d259aea02f7ad89afe641c50ae0cf2ee3ea555354014e3c3aecaed940497feab0c4dce1ba018907e75b0a548ca9a0c7a1eb6a3de4c8ee5ce18186acd0192db8f9044634706a06cb4f4a4286d52e31c873b3be7df59cee09c59fe18a8babc5e415c6abc50f3e9a0e2984cc1734a400ca68d064baf088db6e0961d42cba46079268d02f935ca7544a0f5e4ee51594d7d2900827e93bab68d9e9982db7c3a9af7370e5a6bc314481e87a093ed344f8b3f1cb4f5f6a258e3faf6f7ecd191db8ea9e1f4907084093fd1f30f80",
                        "0xf90211a048547a1df205a0b6b0e9da3adaa016823ff379846b7897b96d215f08db20ccbfa029d456415e8634f9a3ae7fff813444e054d04ccfbc7bfc7dd0c10be73f819964a02e0b673424c3415e45b12f33aeca36c7b6d0d03c8f3721668697034c8114a280a0d0ea5f5b0e4f2d5f7a5b4184f788453fe15d1d2632245547ffa5c96f0d863ffba0a2a24425947d630c98a2e166f89778cde440e67981cbe5c57fa744cc0f457bb3a03c3eaa8ebffb7492b29ac342ed41ce16c91f6ffc2c79dd607d5f79828f5a9899a0af56d30f92fa4d2ee68f8bfabb90694b97bcbda97e240ba8b184be105bf4a451a00fdae3ff0f39702d2a300c18673875511e64138316c5fa20157739aae676c45ba0b54357b6ddc34ea51c6f5f56bf896907433d59f05b08d9815d01868deb0586c1a014702023e2698e9543abb9555a3a013a2e7c94d3de8c94a0db09178cfa771c15a09b5315e6530e5b7c73d6d433f05837628f571571973727eea19f3a147335e412a0b331169bebd930bdd391fb40f176f191f2b0c67c74941ec3d06e0442a90db691a04ec34a19088bce4527ee558c915be5a9dd29d12dc658b7f1c8632caa3bfc64a7a09e55a3a76e2e021d3b04abac31483ff803d4e59f0db15be0432cf1067715b8f0a0896a76786f4752806a49742431ea1a16b3fefc9311821933f8084472e40edb36a036f160f60a75f405510e6cd4779719fb56cc01a6475a927d15e116f7a614f6e380",
                        "0xf90211a09b6f413723da36aeb58a3adebcdab36239722eb05dd95b53635428b3525db581a06aea7e704786aa011539a67c16a9c0e9ed4757bb2363546d2bea2bff5ff76087a0a27efb3f4bb3a80399f3ed167ba89d10e5e60aac92b6cbadb0546112c87b69f5a0e0055bc5c667a26cb31557a4acae534c212cc9606a17444647e9b7dc9d2f422ea0eb49d9ab4f166eadde20524a080d6f2280503f79cbe7d2a2b5d7066bbd759d57a060fe483849e68f3363b55cff913d12db46ec2122b9dc235e3f17ed0d15c1a82ca0884fd506a8540793d07c5fc4eb143da6387ca58524dcd11103acfbf030a037aea001361f05d1286357b5098648325d31742f45709046cbca190040227662dd71d1a0d12c1d02016df13933af4a74b200512a58c9f4c853f8f8dc1d8137f665c96fc3a01eb70d2b2b8ccb7ccec9a9427346dea64994e61878d8656c3078b8924c430f72a0b805fdf7a64e54babd019ddab4311e4ceaf97866657ab1cad28916f5f7957407a0d89e3483058c3748b3bfa66aae527d707278b0a73229fd8d8cd73fed0790bd44a0d551ce75f48a8242bb80f014d1e9dea6b3aca663d877972796f8f632a44b8407a03dc2dd3ca01ff61c108a03123e501448043f2c23a6bf070496d6a0e7a1963acea0fe6e10a0b5874a29c9b0719eba60c09dc569d8dbfdf1e4497681aa0f1c22a583a010b2f3bd30fb2ff3f5a937fbffa474bcbaf73bf1e91b12bb3d44cda3c1a459d480",
                        "0xf90171a03283e59372cf8ba97d07e74694ca34792553fbe66edb2977b0178f82eed5d08fa031d5c4844fba2038746021c42a037fdc66aaca5127ae5b751176c2e0ab6e4774a0b0dd79626691f1a2665747b3c30fbee64f07f6924c94f2de760e2e96c76eb5fca0d3d89073d0998707abe6a5fb9c588d2b5f2c7daf4e06f5c4035947dc948ab242808080a08d66e7ecb126f5a94a76224c8bf8c95e923b9b04447ea6bac6231eaaf016247780a08a1be972896cd2069bccdd7c43e8beeb4e53bf0d46a4f5e9570188237f34b7f7a01dc8b12dca2bf5991fb5c32228944884d05898d46fc1a8bca4afda2da07a31eba0040bfcfb1efca195677f9c895ebc128c7fc2da9dc9d7ba0f39910766fe08a730a08eba7db6df439f693d48604c1e4e9e8dffceb2d3f9fb9b024d64ed16d4c856a8a0cc8af5746c0a29c3209229ea8946d2005b64b9bf0c5f44e3675c0c475a0f16a6a0530fd9d5f91241a184b782702042e0d676a1db3a7ef8bf8ee36cd3ae229c1f098080",
                        "0xe49e2063df22a0e142a321099692a25f57671635492324bb2fdb852cbb7224528483559704"
                    ]
                }
            ]
        });
        let expected = EIP1186ProofResponse {
            address: Address::from(hex!("dac17f958d2ee523a2206206994597c13d831ec7")),
            balance: U256::from(1u8),
            nonce: 1,
            code_hash: H256::from(hex!("b44fb4e949d0f78f87f79ee46428f23a2a5713ce6fc6e0beb3dda78c2ac1ea55")),
            storage_hash: H256::from(hex!("b1d979de742f9e49f16bc95d78216cba45ceb23eb94185d14552879ec1393568")),
            account_proof: [
                hex!("f90211a0f4e2cff38416963d94c95aca99bbec3e01a2625cff007beeaf4cdb7d8a515038a0ad190678a196b68cfe369210824a3fcdcde38302a27c8a089133723b89f9093ea03b085892d6da74a171484a34de93e3e65a9ac4c0b1ca6f88dfcfba8b0d8d2510a09672de1a9c0a380d80baa2bf6bcd05c5cb56eb865f712d597d12940576025a0da0a4a3f0024f8617ee242bbca2a16909f1f909d5a4ad97de91f60da2dc859e1f1ea0ef734ed7c209a02a7204e5895dabddaaf90fd1ba7fe987bf81ad0fb3dc04bd8ca0b69ab5fce31a80eec450d3f6dd6b45b2cc9d5220c82a6906f76cffeaabdbed22a0d227b181bf676d7eb5248156d3c75e8677fc4dfbe0ba73fcdc98259000f7cfc7a0096fdecfdf7f656ba5c9bb425e20aa6d6b21a6bab5b95f69c9a7f6e2f5135bbfa074671ee87f7f86716445dbbf1dbf8e113ba1c2c905dffebafaab43e674e3ba64a01c4b5eed71721e1e9f52d6e9987a86abd7e10e8b85509950992f12645b539b56a01997ff0652394c5f72e52db8c82330e5cf65df72f3a208344bf79b845a2601b6a08c9596b489fa9afd42900c865891e6e3a4274e376dffd121aca657fc99b64126a02cd49a7b8c05585c0c9dd368b5c2d6397bbcad6e3de653c02a68def7d4ba3e2fa05ad2400ea5f646fb7c9dc5c32fe60a9265109d1d9c142777c5fcbf1f778911aaa05657c5a23bb4307ef473e9d1af3e4faac9c81a82a730c228406453521580204580").to_vec(),
                hex!("f90211a05220309e422aa851b9756aeb2a3bb7c821a666982310d1fb4f97bf117e821249a051965b23e120060dfa570686728db89c332f3a99e41703b96b8cf42e393f1cd8a0050c34a3b3115c8d2c430785c2f27976dd07a2daaa099a9dd0acc407b61af86da0ca09ba90042485a211eac80f6efcacba26777b3f235cf0226b23033d9e25b123a0f1a8baef6a03814c83575294425e6d9e04db1377b5235eda9710737975715597a0613ca0a9a5fd7a08b1c3c9ab30eb0b6725b53a65e1d39c6b83fe27513f807205a0fcc7c0bacd270cff6c47b6e8c202419e2255a45d1ba0d35ca7d68143ed7c7178a0be159bf599ea67ad2120a23428e808e45cd83909f87a783eda148c09b04f970fa028aff3c52064443f63ad0fefb9b203e7783a333460e3d62b857860524e23110ba015178d2d6231ae38cd90b62123444043e2dedfaa25dbca161e94eef8458eb4dba0df221a385a3d7a0966017fc1ce568ec4f8c360c79e99bf2bf2500c85f03e6b25a0a00cbaa7b1006ab375dcc118e0975dcf4c56de4514ec4770b76f2f95d2a71202a017dff70ca9e1410b4d619e19b49fb697dd5e5c58ca7a00dd7adc441965b5d422a0f2a67c1d3db23cc7791fb3c1cd9bcbbee2ed0f77d9be4e2fdc3ea5eb05539771a05d6f95e2bafb1b11102089264f2011262bffcc59543d3bb2b932a731c4647044a072b985603ef98a8e49c7f0c52e2cf2a213597196187735f424aba8c73ab3594b80").to_vec(),
                hex!("f90211a0269272e950b4953aadcb8f24af852ef8b3be49924a3b69d7f52f6e98bd59a849a07a4146636b6d2d8bc3adf4ee8aa1d21cc66bf22221eba0837b95c62fea417a4da03e08c7275d9ab95d0b480c2735f5395d22f737318077e699de74e92d3072789fa093c020e64b492c18160066e6288b806e628a0066a812daf18a653e87fb8e9258a06e4d5b58166fa1c79d3b093dc74e6e6093dda276eccb20b4fde52cefb426eebea0356e28b87883e2268a9e4fca3d3e3f654776cf4b2c5f79fa7ae84f724d3a197ca06f716978b17b96e43dc2e085fd18cca8bdc8b576c0a2b22da821379d49ea0e7fa0177c98e2d3516dc12ba987d300cd248af032fe5136fb29e6155223982217bb79a008b522ff473da62e00ed8afddfc336c58dbff52799c4ec4afacf3603d7e61f16a01caadb1c046aaa477fbc1fad08e975fa93b1ba44f2166c6094c75cdcdc5dd52fa077a9d714db981b501fb7ca3c4c151ac712696334d45db57f1054ea84ad27daaaa017de69774c53d8f507d312c9594f13f03ce52d7fd8d706b138752f15eec14fc3a0fe9c84acdc43360e1f82022285cf9bad1198eeefa1aa82860af333d9191be851a02676aa89c3fe2af88468586e6aebc1113db1f0ab2d46f7b5be9169fcb03e538aa005de087a009e870b948f21ec5b816a1fffd77bd6045b6e984a54687e0404859da0b41409a679b7530d09b48a8be995368f9190c6654a039958292872d6e30e502a80").to_vec(),
                hex!("f90211a065120a944f3ac11fd7a4f00a33ee69597b27960d4faf08488c095a54bf57171fa04364126d60c023dc29c4ece6ca7ad2b995b5661b20859faf904ea177ccdf5264a0863c7609c1a37e5f473a0e89a78a9f669336ce4a878b3229df5b776b15857003a0a1366941e9cdf61171f97d2909e999070dbb4f19d958be1303098f621c2d0cd3a0953c367aca82d2ffc36a0a38b089b4b4cae4421fe75b892bc8814c66c1f57f8fa02a5c39a5214f1af21675d8c8533e8309f271df89fe4ac6889b9ffdb7fcbafabba01cd4cc0ecf0609d17aba14998460c3ea7d5d087f6afa27e7f0f6938db6f4da48a0f0951d9162741ffdad9f88a12fe5adbd420cb4d51b5c86bb858ad9f6c663ca77a00a6ff6752d7fa2e2cf35f122f384fa81c3e31843208de33312a51daf8ec6f602a082dd49ddbf5fa63916480e19a6160b52eed3b528bf565d756338b71fcb40e226a04a04700f71e8beca15df068d2673acd9320057ddf9e944306c1d0f30fe081da3a0df3445cd82e2c301325854af499753ca70c993355ef316f501330c0685ee06b8a0cf7b31eb4ce89e2086ed15849e0dd1cebedb335fed1a3253a76abf4c4403600da0581f31842e2ba26a16ca6ba7ff199f2ad7b1372bd189e59bc75dbfb3cf328c8ba02112f5d832f7f8decaa0bb91972f6c0c4bfbd3c8eb4715e8c575672f52fa4bfba0ac29d3bfe5bab2855854beda7617d3c36c32bdd34cc1b4eac16064d824fcb93c80").to_vec(),
                hex!("f90211a08cfbe475652906bec65ab34a0aafe3e108d8a3814302481d0702ff70e8b1ec7da0bb52733912f93ef21f2ad6d053baee454028f6f24c7b262e97e15b69fd838f29a01790de8c1b55778ff8792468918bdca8c080401727f1e7dfb1996fb817d4a5b2a0fa5bec47b083ad118ec0bedef4060ed5ff1eec1532f3b91da3fbf23488f3ffe9a0578b91094da870fc1730f276c74f612d65581df80433be59118132c97e5a90b1a0ca767563a6658d7b6a4840c3e1e3d65a98a6adb3952244c2def748be30c81d20a062260ad247f062a42f22076c3d8ebfdbfaf376048d67ace5e958b21b6ffa7ffda0a46528927602d743309235d3a50b9d42975048db410ca5deff039c5045a1bf6ea0fbdd64c68af03cde60a403ae7d1b08a0bc89b36bef3a2bc1b44b36316ecc7fdea0593b5de878ae20b1a14b4f7c63f1e64060f1e6bad4baa5704d28528fa9b3733ea0537aa00fbb93fbc8bf0d3043e44fbd6d227c648284e12157e6af533113d83e5ea085de7af69c1c38259bc36a5a93a8959b8d04856e6657fe528a775f2dd5e1be7ca00e2b391cd8c555e85e7406b21d853d18096be5a1f6f30399e0f569251f74d8f1a0b304bc6cbbcd5864c1338b29e7871f739976ebfa3734c04fad1288a10428352ca0fc83234d8f44b46c6a77c42f49cbbb62b5850c8274c3f33ac238d2966629aa47a0a30201dac0877046f23222c6e05cecda65a7f423e88946bcd099cc50446d865780").to_vec(),
                hex!("f90211a0e31b4c19fcc4fcb854b8e624b318ad21b022c57c58e30f80f27bcd9fe4e61649a0950522557eb40bafbd082f0f5cc4be3bcdcb7f80c14eee43afd2bcd01f8d5137a006344fc5ae8c6063578d0b997b0caebc50abb303fc195a9875c55a5d3e7566b4a011a2f9312c3308640a0d6ceeae218747290f23806067456da1d444c65abae437a0b3097a108bfce79af6699da4ae3003cd4929f0b4576aad655c31cb725bde84c7a00975b2742460058745a4ee9f17d4c2cd50047b9831702204293a4648ea4e21d3a0b324be589fdd06f1d77158f23e1ce85661ed325abcd3c34aae53b2138db1f61aa02e7927d9c620923d4bace4882588753ce2bd16373ecfefe90b71a8973d8d6b15a09aa229b179290efb4a6b52687b928c0ab96962acf989d59de2fde5a1a657142ca0750babf0e184cfa59851d215e74e4a351fe5273adbce334c23a54a2a564c3a65a00b3757b624f3e65e3cadbd9b61e092af2f6086fe846ac6ed51a2f0261d21b475a0b7d528fc41c8fdc8ea18c6e7d0099270c777ec1403cf879d1f5134bdc12a6c6ca062a0b052298c12a244f472f1eec5f5d387da0760f654d348de7439855781b655a075fd0c30e17585c20d95627e7e9eb83bfc7b1be1a6ff69b80b1401b7ad393ba3a0c2aaa60bccbeb370c420a774007f7b35d86ae3e837309e12218664378c1573bea08bd2b242e992653fa60521d04209d0f948548de03ed9d063f6c847212da606f480").to_vec(),
                hex!("f90191a00a7a0118e00981ab321049c9d340cd52c3a4781037540f7c48d0fdc27e899b3280a08537f2e248702a6ae2a57e9110a5740f5772c876389739ac90debd6a0692713ea00b3a26a05b5494fb3ff6f0b3897688a5581066b20b07ebab9252d169d928717fa0a9a54d84976d134d6dba06a65064c7f3a964a75947d452db6f6bb4b6c47b43aaa01e2a1ed3d1572b872bbf09ee44d2ed737da31f01de3c0f4b4e1f046740066461a064231d115790a3129ba68c7e94cb10bfb2b1fc3872f7738439b92510b06551bea0774a01a624cb14a50d17f2fe4b7ae6af8a67bbb029177ccc3dd729a734484d3ea04fb39e8b24158d822454a31e7c3898b1ad321f6c9d408e098a71ce0a5b5d82e2a0c8d71dd13d2806e2865a5c2cfa447f626471bf0b66182a8fd07230434e1cad2680a0e9864fdfaf3693b2602f56cd938ccd494b8634b1f91800ef02203a3609ca4c21a0c69d174ad6b6e58b0bd05914352839ec60915cd066dd2bee2a48016139687f21a0513dd5514fd6bad56871711441d38de2821cc6913cb192416b0385f025650731808080").to_vec(),
                hex!("f8669d3802a763f7db875346d03fbf86f137de55814b191c069e721f47474733b846f8440101a0b1d979de742f9e49f16bc95d78216cba45ceb23eb94185d14552879ec1393568a0b44fb4e949d0f78f87f79ee46428f23a2a5713ce6fc6e0beb3dda78c2ac1ea55").to_vec(),
            ].to_vec(),
            storage_proof: vec![
                StorageProof {
                    key: H256::from(hex!("61d6b5a02307fb160a7bed094f54b8182c2887c4349a7bcd4ac6936a4d620faa")),
                    value: U256::from(0x0001_6bcc_431c_c060_u128),
                    proof: [
                        hex!("f90211a07664e825351ee2559903e491b035d48baf90716617d9c771a51cb38a467fe9efa0aba0f80c633a27c16fb13a8ac92bd09cb27ba6c2d82a1a907c12942f79399a76a0204fd8890a2f789880340f79d12cd28a03060970a16442824c6132bc7351be4fa0c27158a8fa82e0aa7f15f847882ce06b5bd9aeceb7f7accb69c68c128f9a2d8ba002133406256ef781ad23ad7833e4e272e149bbec9704bcbbd6cc71126aa9bb95a05e5e108691679aa1b9fd69778871bf93339fe70dd80c090e6da9f361ad112ba8a05ff5593b34953ea40858ae61bcab89677656746f84d288ee490c57709a2f18a4a009ede6719eeb6aa535f05402a9fed93336e4095b177f70979d025b29978ea82ea06918f4c1f900d4ff9ade9968443af0a1fac265e7580bf49d246c63d5fef4b876a0d65743ac746b470a64b47e853ed82e3816f34bfaf6f86eda7e073f5fd5429d6ea01d1e8d2577abd72e0ccadaee3bc643f2529ce036bbab66e8e6fdb5c6db84149ca02153275cc8b8b89dbf25a854e2d4f604bc2701960f8ebcb1c19207b9fea1f168a03406e7c018f995f29bd344aad8db4d6d621c16583db4a7e8414a6cb3e2086a2ca070accfe4cbd95432dd5731643ee6672bd7abc01e0189265da32b2ea21bedaac8a00466ef35fc0960732dd08d0147bd5b4f606ec43115def41fa77cd4f31c2b6be1a002c62ef81f50e53fd3ec15e19fcece50775ca076657d42befcddab93fef0905880").to_vec(),
                        hex!("f90211a0f9feb6d18df70a52b7faabaa486966e25e815348f89abb88a26613aee21f1728a0d38d07725cf4c0744f53fddf07d7d9d677caeffbc6d27c5cbc48de08a4cfabb0a0fbfac941f5550c295d2a4d0fb00fa12434861d74ed5aeddb69e6e20779a7bc7da062b62b461e9317bfcbceaf607daa4ed7afa52e98fbcef94436705c9d5314ba47a0c76acb6b4839d5330da1eff86496f157ef46d09e1aafbfbda0e3bc49abe7d1eea083de1c187d1e82bae56a8ee7670082a0c4955d62ea3ea4181ebbf4b4a9c54b06a0602a729afcb9b901520a956d859cb8b0b9425e4fc4950599e7c0a485eb4c9e12a0ac6e132e49c21d2d0f46ed19c95b87a78643f4b47c1cfcace74f7d32b43ff188a09450bd1f5fb22bd6bb1b3801a521a100cf5dd2b4999b8d8e318d65b5332102eba0cb6b312c970c010dccc6643b6178d1d594494acf837dafcddf2c170e98277afca0b9eb554af6dfec821b716df886741da322465540e042d1b624ced5830967a04ba052564e86b32298e801c7f183e730f6d4df783d14121cf2513c600403c24f46afa046a14e816a4a4d2f9dbc01d380d61103c74b542931eb03e42f33cd78640c9ae4a0d9fddeaad53578d7d9cc3e34624899ecea5522caf2e4d8ac94631a0370ec9904a0eb0ebc196c08dba1ad659293e61dee6240595752364cb0ba3c1cada9e9c3ecf1a01fb26ca7a5563ffcc47cd85d5dfcdb0055870f00111ff89c33273b263082d0a480").to_vec(),
                        hex!("f90211a050334ca647f515eae4ffd1f372129684564627b0762da351cb64abfbebe5a2b4a077e12b72ffe91604d91440508cd23e1259e9bb430b35d6a140b5db236361e6e4a030204c15b02247347421af85be201e53f29998243a7cbf1d5b1c6841bd274000a04c196f18d0f7cd44b0f78459651388dd7580b5944ac81a6f0b5f8134fd5b9713a081ff955af7b8fd112203bb57eb57335eae4d83e0419dee5e00382b4b155c6bbca05b79b8ca9a6fde56e54b5f1cba081e6f113ec0e1aa5625c426aa014ec013604ba05b2183e90cbe7efdb2cc0568c5b1733d6f1e5ae7bd13482a7e4826d0150dececa0bb803ef214d38a3f89bfc388d9a58e1470fc6d2a0efa1a4a71363360b323d38ba0d1b50eb36cba91bb1585e9132fe5ea2e1d4cb2beb260357d668c2e37c4fe6f88a07faa02c13272dcf5993dea50c20ae65a205614fc9de37ae6f7646c59dd3d15fda00e43d223d9bec16152aba6e6d5cbf99d1c94cedc261e9cc451f19d8752f279cca0c2e27b9487115c7e7a454f2deb775ee914a5d57785ea3e594a6de1305c3c8419a08dbdc8c02cd069cdf1cae1b818bb23d9b1d1ef4244f78940befda9296b8492a1a07cbef9cb730ad97fee202fa060bb1e19ec132576ae6a120f52a9290bf5dbf56ea0bd4a3b04cf41493126e643cb73ab2bc42382af85b70eca8a2554336a92c19158a0cd2c5730c90df1c053c2d4d7ee1d67c803ec12774f4bf0a42c9f52c8fc56187680").to_vec(),
                        hex!("f90211a09dc0c0b5ffdb00d49fd73c13d816943a4d0328f8a7e4200ac5fca94074fe37d9a0008201d0ddb8b50b7e8126098d66c6cc03104fd40877cc7be8964cb71a3f68c6a0a6a236a5055c42e21a8fc0ababdfe13b4cedd3308338b1695d02919eb97eae1ea03f7c7e823fc35d6d82566698a624ebf374535236da1d774e274b6a30f6b9f740a04575e395da64fd92771c0c16b968a6d66a1dd4b5ecb9f7ba7c14906039042618a0c4d521806c922a6c083789b34a023c11101282f3d9987caf7f442040610d2368a0093294883edcc6bf1cec4fef3102942b0747004caefc7006b2fb86249f8287a7a09ef9f0d36f206f959dfceb907fa3bd76c42a7b8f870eb4b938322d3f25e61ca4a076b8308d195f017747da24ebb46e5be13a7090f6b094a0399d5ff65eec302e98a0339ee7bce47d1211235244f0f05f3da6cbb856a25c9f006036c8d13dd4012958a0c28ca6cdb5ed3d19182a0e874844ab09eafc261a30c2db35aaffde7f1e8872eca0164dd52a75274e63da527ce01d22409bfa1c5cc4de7ffc3523fb08c3480cbce2a08113a1cbc5367389a60fba83040b2ca176a3d3b6fc93e04a4e724f9adeb66907a0e07e0b466f71aa1513c07a23e19f0475f255647655df79ff7cb3dae579208c35a0608f1b9b054948a48bf7cd317ff3708114dcaba99bc136569ea48bbea12cf6b9a06446e6a7fec8afc75cee385d181c721b466a32cd8555f4afe2a65ef8a837011a80").to_vec(),
                        hex!("f90211a0b2c39dc101beebed6ed68f57c63d6f7333c38a70a2809ac29d9caad53b410957a0dc1571eaacadc4eea1a30e760a49267c7cf7945b09c3435e321bfbe8f71cbe19a0fdcde768c5738af5a7c71e65ef4d8b427e18e816657525bf7c38ea8bd5a29104a03a039a2bba043ce489d8c6145789c42121867739b9349f60769c96a41a465f6ba069cf2da149e40b92212b28ad8e031afb34a73942e4d9f861e40914b53abe4b67a05284d08672cdfa118396d99303c0a54f0b6d4a88672886ab9e8244cd7e23515ca0b56ab7578196f7a6d517519bfcf1899958ca97a590dc3de5d80ea8f34577d423a0f0eccb83567f2b4a21abc3ff39cefadef31994935a8ca707fefa1f4cc71f46e1a0a4b786589392014ca8d32425fa56e5f7b6da1d0dbf9a4c2fb98115cc6ccf15f0a0c455b01229b37964007474ae1398c877247275f1e7cff01203c60de253d44a3ba0b14d6ec313f1d918cbfc6ed5bfd65ec9436385084ef2aff37d3826403a68f789a02bad9e2afd24cb768b93e4790235b0ccfe35d15e84c8706bb22e0140eed0842ca033d9fb4d25f863f55f56dd500786060a3714a94e097e34ecde79412d80d25a80a02a57734da428a685a9f7339c60f508e5b6abe788b32ec3801afd1ffefbc25b01a04815c84b9eb7b1bd09d6ea7ca65c202d4bb96241cece38976a49011490f199ada0dc517014368265f40f1b4797f4060fc50f6a7d11c9ff270b3fb0fb63dba76ebd80").to_vec(),
                        hex!("f8f1a0b84afd46c5ef7d58b7ec8ff4e54099bf9021e51c562bfc81a8291caa05e33db880a0ab5289bc9d06c3747d271ca8c0a2f6a482b7eb2b103d1ff6682ac5cf66685fff8080a05c67492e2a6961f8b702b3abb00694c775ae15c1c8ccfcc62f766c413c98bb1f80a05f4486c44fbe102c65b8a3e85bbb4833768739e55ba4e68286f8852f053afe688080a0dda7960d591160912efc067283517c2496c881fc12e2c8777233defd0bcef1fb8080a06a5a73737c8e13e0c062f175017894bcd744157a4a2f7353f3366b02a9715d3580a0ed913b44f62ac0485fdfb6daf7b45b95156daedbd54467f3c9b2e43bfcf63d4d80").to_vec(),
                        hex!("f85180808080a09548469fab745c696f461425294816340edcd44fa9cf1d84c201342aa5b590c6a0e64144f645b0cd8966f9869bec53d27e74845b93d1226b617c4df287509074178080808080808080808080").to_vec(),
                        hex!("e79d33878a1bf90b4a94e2dfcf7e46345d6c3f209bf31504ec1f349be19d2f8887016bcc431cc060").to_vec(),
                    ].to_vec(),
                },
                StorageProof {
                    key: H256::from(hex!("000000000000000000000000000000000000000000000000000000000000000a")),
                    value: U256::zero(),
                    proof: [
                        hex!("f90211a07664e825351ee2559903e491b035d48baf90716617d9c771a51cb38a467fe9efa0aba0f80c633a27c16fb13a8ac92bd09cb27ba6c2d82a1a907c12942f79399a76a0204fd8890a2f789880340f79d12cd28a03060970a16442824c6132bc7351be4fa0c27158a8fa82e0aa7f15f847882ce06b5bd9aeceb7f7accb69c68c128f9a2d8ba002133406256ef781ad23ad7833e4e272e149bbec9704bcbbd6cc71126aa9bb95a05e5e108691679aa1b9fd69778871bf93339fe70dd80c090e6da9f361ad112ba8a05ff5593b34953ea40858ae61bcab89677656746f84d288ee490c57709a2f18a4a009ede6719eeb6aa535f05402a9fed93336e4095b177f70979d025b29978ea82ea06918f4c1f900d4ff9ade9968443af0a1fac265e7580bf49d246c63d5fef4b876a0d65743ac746b470a64b47e853ed82e3816f34bfaf6f86eda7e073f5fd5429d6ea01d1e8d2577abd72e0ccadaee3bc643f2529ce036bbab66e8e6fdb5c6db84149ca02153275cc8b8b89dbf25a854e2d4f604bc2701960f8ebcb1c19207b9fea1f168a03406e7c018f995f29bd344aad8db4d6d621c16583db4a7e8414a6cb3e2086a2ca070accfe4cbd95432dd5731643ee6672bd7abc01e0189265da32b2ea21bedaac8a00466ef35fc0960732dd08d0147bd5b4f606ec43115def41fa77cd4f31c2b6be1a002c62ef81f50e53fd3ec15e19fcece50775ca076657d42befcddab93fef0905880").to_vec(),
                        hex!("f90211a05f55533e0b43b528ea30aa81a8f70ba5c4f902cd4cd6c0dc44999e16462c9592a0c2922ce429e37b6d769f2305cacde39f777e7e90e8ce387959a8954591ad54c1a053e8e848d47002528949229d58c6f9c0672f88bc4d378cc1fe39c17b9bef86f8a0e7dc7b8aae803a5447a35f63ec2b2cebf49fee5489d5b9738e3d64c3156c6c2ba02dcd41fd90ebf9c932769c1fd7a284302bf05a25746530d45aedec98159faa4fa08529feaf44ddfa79256a045513d56cdcc17ad172704d2d651098c6c6cf643e53a00e8295e2ef6849a73a32c606ec0cf0d6e5f4cc826dbb26f41ca198792c4325a0a0341359e052289aa4bfc05791e28fe87440fe9a6cd2502aac70300f595da27bb2a0a8c574979a99f008a7b0fdb3f5d1aa5c81f75faf419f355c34f00bbae9e9674ea09bdfd474f8c35b5614b23311f65c3e1d107b7cb19120527ccb449a138954e375a02f656aecd41e6b13d93e7f074d2e47f10693c6a77bbda9b75557ae608395db31a0d56eed1e5a7bb97f42335a91cc773a2e9ff4c61cd02aab47c9ea3c21740559b8a096a539cc0c81fc92ebfd88262a35a243e01b61d0cb08cf30b3327d5b35426df6a0dd9a0d31ad7b4595a44af610170dfa44286f3dc9cbb93dcc5d3b1bd37ffa30c0a020b375a158b4a2f81182c9fbca3d6640455b549a404bc468d4e07f4e86dc150ba0461e6ae086898a77502b18cf1378eb5c59b5ec0a1311e3c97fa1d24a031c0b7f80").to_vec(),
                        hex!("f90211a0e8c1590de40a9530e23414782dc039b15f3635be011b27883bc716ed12450cd4a0397d720c7c5460b2a0898e4d2d0e720607e0938572d971afdf140d745e989adea0f8a28e1f408ad7dee0a106762f44fd714fff00f7c0e9a03de6620de569222ad2a0967800891aa68296913852853498ea41c085b558677731a5af0b751a08e8e66fa098e1c0f92bb44553815db48ddad8fa3d932793fe136c9063b1459f64d106c598a0a1891322c34e83c223ce162139a98ffd771015c42ba858c71ecafb7b4c47f29ba072dffe10e2576b4b63ad0db97a75ce549d369c14de14a27377620ab4cd684171a0fe37330c7f1857fdf50a4dc14752512d3d7b00227164c5fac773984c94b7b1eaa0bbee82adbca630862a977a48ea7f569bb8fbd8318589b6944aff1f78b8176f0ba08f5abe4ae9d3790c1c3d0d84f01afd95d0862f14bca08150e82d304081d259aea02f7ad89afe641c50ae0cf2ee3ea555354014e3c3aecaed940497feab0c4dce1ba018907e75b0a548ca9a0c7a1eb6a3de4c8ee5ce18186acd0192db8f9044634706a06cb4f4a4286d52e31c873b3be7df59cee09c59fe18a8babc5e415c6abc50f3e9a0e2984cc1734a400ca68d064baf088db6e0961d42cba46079268d02f935ca7544a0f5e4ee51594d7d2900827e93bab68d9e9982db7c3a9af7370e5a6bc314481e87a093ed344f8b3f1cb4f5f6a258e3faf6f7ecd191db8ea9e1f4907084093fd1f30f80").to_vec(),
                        hex!("f90211a048547a1df205a0b6b0e9da3adaa016823ff379846b7897b96d215f08db20ccbfa029d456415e8634f9a3ae7fff813444e054d04ccfbc7bfc7dd0c10be73f819964a02e0b673424c3415e45b12f33aeca36c7b6d0d03c8f3721668697034c8114a280a0d0ea5f5b0e4f2d5f7a5b4184f788453fe15d1d2632245547ffa5c96f0d863ffba0a2a24425947d630c98a2e166f89778cde440e67981cbe5c57fa744cc0f457bb3a03c3eaa8ebffb7492b29ac342ed41ce16c91f6ffc2c79dd607d5f79828f5a9899a0af56d30f92fa4d2ee68f8bfabb90694b97bcbda97e240ba8b184be105bf4a451a00fdae3ff0f39702d2a300c18673875511e64138316c5fa20157739aae676c45ba0b54357b6ddc34ea51c6f5f56bf896907433d59f05b08d9815d01868deb0586c1a014702023e2698e9543abb9555a3a013a2e7c94d3de8c94a0db09178cfa771c15a09b5315e6530e5b7c73d6d433f05837628f571571973727eea19f3a147335e412a0b331169bebd930bdd391fb40f176f191f2b0c67c74941ec3d06e0442a90db691a04ec34a19088bce4527ee558c915be5a9dd29d12dc658b7f1c8632caa3bfc64a7a09e55a3a76e2e021d3b04abac31483ff803d4e59f0db15be0432cf1067715b8f0a0896a76786f4752806a49742431ea1a16b3fefc9311821933f8084472e40edb36a036f160f60a75f405510e6cd4779719fb56cc01a6475a927d15e116f7a614f6e380").to_vec(),
                        hex!("f90211a09b6f413723da36aeb58a3adebcdab36239722eb05dd95b53635428b3525db581a06aea7e704786aa011539a67c16a9c0e9ed4757bb2363546d2bea2bff5ff76087a0a27efb3f4bb3a80399f3ed167ba89d10e5e60aac92b6cbadb0546112c87b69f5a0e0055bc5c667a26cb31557a4acae534c212cc9606a17444647e9b7dc9d2f422ea0eb49d9ab4f166eadde20524a080d6f2280503f79cbe7d2a2b5d7066bbd759d57a060fe483849e68f3363b55cff913d12db46ec2122b9dc235e3f17ed0d15c1a82ca0884fd506a8540793d07c5fc4eb143da6387ca58524dcd11103acfbf030a037aea001361f05d1286357b5098648325d31742f45709046cbca190040227662dd71d1a0d12c1d02016df13933af4a74b200512a58c9f4c853f8f8dc1d8137f665c96fc3a01eb70d2b2b8ccb7ccec9a9427346dea64994e61878d8656c3078b8924c430f72a0b805fdf7a64e54babd019ddab4311e4ceaf97866657ab1cad28916f5f7957407a0d89e3483058c3748b3bfa66aae527d707278b0a73229fd8d8cd73fed0790bd44a0d551ce75f48a8242bb80f014d1e9dea6b3aca663d877972796f8f632a44b8407a03dc2dd3ca01ff61c108a03123e501448043f2c23a6bf070496d6a0e7a1963acea0fe6e10a0b5874a29c9b0719eba60c09dc569d8dbfdf1e4497681aa0f1c22a583a010b2f3bd30fb2ff3f5a937fbffa474bcbaf73bf1e91b12bb3d44cda3c1a459d480").to_vec(),
                        hex!("f90171a03283e59372cf8ba97d07e74694ca34792553fbe66edb2977b0178f82eed5d08fa031d5c4844fba2038746021c42a037fdc66aaca5127ae5b751176c2e0ab6e4774a0b0dd79626691f1a2665747b3c30fbee64f07f6924c94f2de760e2e96c76eb5fca0d3d89073d0998707abe6a5fb9c588d2b5f2c7daf4e06f5c4035947dc948ab242808080a08d66e7ecb126f5a94a76224c8bf8c95e923b9b04447ea6bac6231eaaf016247780a08a1be972896cd2069bccdd7c43e8beeb4e53bf0d46a4f5e9570188237f34b7f7a01dc8b12dca2bf5991fb5c32228944884d05898d46fc1a8bca4afda2da07a31eba0040bfcfb1efca195677f9c895ebc128c7fc2da9dc9d7ba0f39910766fe08a730a08eba7db6df439f693d48604c1e4e9e8dffceb2d3f9fb9b024d64ed16d4c856a8a0cc8af5746c0a29c3209229ea8946d2005b64b9bf0c5f44e3675c0c475a0f16a6a0530fd9d5f91241a184b782702042e0d676a1db3a7ef8bf8ee36cd3ae229c1f098080").to_vec(),
                        hex!("e49e2063df22a0e142a321099692a25f57671635492324bb2fdb852cbb7224528483559704").to_vec(),
                    ].to_vec(),
                }
            ],
        };

        // Can decode json
        let actual = serde_json::from_value::<EIP1186ProofResponse>(json.clone()).unwrap();
        assert_eq!(actual, expected);

        // Can encode as json
        let encoded = serde_json::to_value(&actual).unwrap();
        assert_eq!(encoded, json);
    }
}
