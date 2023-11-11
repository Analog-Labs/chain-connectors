use crate::{
    bytes::Bytes,
    eth_hash::{Address, TxHash, H256, H512},
    eth_uint::{U256, U64},
    transactions::access_list::AccessList,
};

/// Transaction
#[derive(Clone, Default, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct RpcTransaction {
    /// Hash
    pub hash: TxHash,
    /// Nonce
    pub nonce: U64,
    /// Block hash
    pub block_hash: Option<H256>,
    /// Block number
    pub block_number: Option<U64>,
    /// Transaction Index
    pub transaction_index: Option<U64>,
    /// Sender
    pub from: Address,
    /// Recipient
    pub to: Option<Address>,
    /// Transfered value
    pub value: U256,
    /// Gas Price
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub gas_price: Option<U256>,
    /// Max BaseFeePerGas the user is willing to pay.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub max_fee_per_gas: Option<U256>,
    /// The miner's tip.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub max_priority_fee_per_gas: Option<U256>,
    /// Gas limit
    #[cfg_attr(feature = "with-serde", serde(rename = "gas"))]
    pub gas_limit: U256,
    /// Data
    pub input: Bytes,
    /// Creates contract
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub creates: Option<Address>,
    /// Raw transaction data
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub raw: Option<Bytes>,
    /// Public key of the signer.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub public_key: Option<H512>,
    /// The network id of the transaction, if any.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub chain_id: Option<U64>,
    /// The V field of the signature.
    pub v: U64,
    /// The R field of the signature.
    pub r: U256,
    /// The S field of the signature.
    pub s: U256,
    /// Pre-pay to warm storage access.
    #[cfg_attr(
        feature = "with-serde",
        serde(default, skip_serializing_if = "AccessList::is_empty")
    )]
    pub access_list: AccessList,
    /// EIP-2718 type
    #[cfg_attr(
        feature = "with-serde",
        serde(rename = "type", skip_serializing_if = "Option::is_none")
    )]
    pub transaction_type: Option<U256>,
}

#[cfg(all(test, feature = "with-serde", feature = "with-rlp", feature = "with-crypto"))]
mod tests {
    use super::RpcTransaction;
    use crate::{
        bytes::Bytes,
        eth_hash::Address,
        eth_uint::{U256, U64},
        transactions::access_list::AccessList,
    };
    use hex_literal::hex;

    #[test]
    fn decode_legacy_json_works() {
        let json = r#"
        {
            "hash": "0x831a62a594cb62b250a606a63d3a762300815c8d3765c6192d46d6bca440faa6",
            "nonce": "0x32a",
            "blockHash": "0xdbdb6ab6ef116b498ceab7141a8ab1646960e2550bafbe3e8e22f1daffacc7cf",
            "blockNumber": "0x15780",
            "transactionIndex": "0x0",
            "from": "0x32be343b94f860124dc4fee278fdcbd38c102d88",
            "to": "0x78293691c74717191d1d417b531f398350d54e89",
            "value": "0x5fc1b97136320000",
            "gasPrice": "0xde197ae65",
            "gas": "0x5208",
            "input": "0x",
            "v": "0x1c",
            "r": "0xc8fc04e29b0859a7f265b67af7d4c5c6bc9e3d5a8de4950f89fa71a12a3cf8ae",
            "s": "0x7dd15a10f9f2c8d1519a6044d880d04756798fc23923ff94f4823df8dc5b987a",
            "type": "0x0"
        }"#;
        let expected = RpcTransaction {
            hash: hex!("831a62a594cb62b250a606a63d3a762300815c8d3765c6192d46d6bca440faa6").into(),
            nonce: 810.into(),
            block_hash: Some(
                hex!("dbdb6ab6ef116b498ceab7141a8ab1646960e2550bafbe3e8e22f1daffacc7cf").into(),
            ),
            block_number: Some(87936.into()),
            transaction_index: Some(U64::zero()),
            gas_price: Some(59_619_389_029u128.into()),
            gas_limit: 21000.into(),
            from: Address::from(hex!("32be343b94f860124dc4fee278fdcbd38c102d88")),
            to: Some(Address::from(hex!("78293691c74717191d1d417b531f398350d54e89"))),
            value: 6_900_000_000_000_000_000u128.into(),
            input: Bytes::default(),
            chain_id: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            creates: None,
            raw: None,
            public_key: None,
            v: 0x1c.into(),
            r: hex!("c8fc04e29b0859a7f265b67af7d4c5c6bc9e3d5a8de4950f89fa71a12a3cf8ae").into(),
            s: hex!("7dd15a10f9f2c8d1519a6044d880d04756798fc23923ff94f4823df8dc5b987a").into(),
            access_list: AccessList::default(),
            transaction_type: Some(U256::zero()),
        };
        let actual = serde_json::from_str::<RpcTransaction>(json).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn decode_eip1559_json_works() {
        let json = r#"
        {
            "blockHash": "0xfdee00b60ddb4fd465426871a247ca905ff2acd5425b2222ab495157038772f3",
            "blockNumber": "0x11abc28",
            "from": "0x1e8c05fa1e52adcb0b66808fa7b843d106f506d5",
            "gas": "0x2335e",
            "gasPrice": "0xb9c7097c0",
            "maxPriorityFeePerGas": "0x5f5e100",
            "maxFeePerGas": "0xbdee918d2",
            "hash": "0x24cce1f28e0462c26ece316d6ae808a972d41161a237f14d31ab22c11edfb122",
            "input": "0x161ac21f0000000000000000000000001fe1ffffef6b4dca417d321ccd37e081f604d1c70000000000000000000000000000a26b00c1f0df003000390027140000faa71900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002360c6ebe",
            "nonce": "0x1cca",
            "to": "0x00005ea00ac477b1030ce78506496e8c2de24bf5",
            "transactionIndex": "0x5f",
            "value": "0x38d7ea4c680000",
            "type": "0x2",
            "accessList": [],
            "chainId": "0x1",
            "v": "0x0",
            "r": "0x8623bae9c86fb05f96cebd0f07247afc363f0ed3e1cf381ef99277ebf2b6c84a",
            "s": "0x766ba586a5aac2769cf5ce9e3c6fccf01ad6c57eeefc3770e4a2f49516837ae2"
        }
        "#;
        let expected = RpcTransaction {
            hash: hex!("24cce1f28e0462c26ece316d6ae808a972d41161a237f14d31ab22c11edfb122").into(),
            nonce: 7370.into(),
            block_hash: Some(hex!("fdee00b60ddb4fd465426871a247ca905ff2acd5425b2222ab495157038772f3").into()),
            block_number: Some(18_529_320.into()),
            transaction_index: Some(95.into()),
            gas_price: Some(49_869_264_832_u64.into()),
            gas_limit: 0x2335e.into(),
            from: Address::from(hex!("1e8c05fa1e52adcb0b66808fa7b843d106f506d5")),
            to: Some(Address::from(hex!("00005ea00ac477b1030ce78506496e8c2de24bf5"))),
            value: 16_000_000_000_000_000u128.into(),
            input: hex!("161ac21f0000000000000000000000001fe1ffffef6b4dca417d321ccd37e081f604d1c70000000000000000000000000000a26b00c1f0df003000390027140000faa71900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002360c6ebe").into(),
            chain_id: Some(U64::one()),
            max_priority_fee_per_gas: Some(100_000_000.into()),
            max_fee_per_gas: Some(50_984_458_450_u64.into()),
            creates: None,
            raw: None,
            public_key: None,
            v: 0x0.into(),
            r: hex!("8623bae9c86fb05f96cebd0f07247afc363f0ed3e1cf381ef99277ebf2b6c84a").into(),
            s: hex!("766ba586a5aac2769cf5ce9e3c6fccf01ad6c57eeefc3770e4a2f49516837ae2").into(),
            access_list: AccessList::default(),
            transaction_type: Some(2.into()),
        };
        let actual = serde_json::from_str::<RpcTransaction>(json).unwrap();
        assert_eq!(expected, actual);
    }
}
