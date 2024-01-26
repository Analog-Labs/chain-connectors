#[cfg(feature = "serde")]
use crate::serde_utils::{deserialize_null_default, uint_to_hex};
use crate::{
    bytes::Bytes,
    eth_hash::{Address, TxHash, H256, H512},
    eth_uint::U256,
    transactions::{
        access_list::AccessList, eip1559::Eip1559Transaction, eip2930::Eip2930Transaction,
        legacy::LegacyTransaction, signature::Signature, signed_transaction::SignedTransaction,
        typed_transaction::TypedTransaction,
    },
};

/// Transaction
#[derive(Clone, Default, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct RpcTransaction {
    /// Hash
    pub hash: TxHash,
    /// Nonce
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub nonce: u64,
    /// Block hash
    #[cfg_attr(feature = "serde", serde(default))]
    pub block_hash: Option<H256>,
    /// Block number
    #[cfg_attr(feature = "serde", serde(default, with = "uint_to_hex"))]
    pub block_number: Option<u64>,
    /// Transaction Index
    #[cfg_attr(feature = "serde", serde(default, with = "uint_to_hex"))]
    pub transaction_index: Option<u64>,
    /// Sender
    pub from: Address,
    /// Recipient
    #[cfg_attr(feature = "serde", serde(default))]
    pub to: Option<Address>,
    /// Transfered value
    pub value: U256,
    /// Gas Price
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub gas_price: Option<U256>,
    /// Max BaseFeePerGas the user is willing to pay.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub max_fee_per_gas: Option<U256>,
    /// The miner's tip.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub max_priority_fee_per_gas: Option<U256>,
    /// Gas limit
    #[cfg_attr(feature = "serde", serde(default, rename = "gas"))]
    pub gas_limit: U256,
    /// Data
    #[cfg_attr(feature = "serde", serde(default))]
    pub input: Bytes,
    /// Creates contract
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub creates: Option<Address>,
    /// Raw transaction data
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub raw: Option<Bytes>,
    /// Public key of the signer.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub public_key: Option<H512>,
    /// The network id of the transaction, if any.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none", with = "uint_to_hex",)
    )]
    pub chain_id: Option<u64>,
    /// The V field of the signature.
    #[cfg_attr(feature = "serde", serde(default, flatten))]
    pub signature: Signature,
    /// Pre-pay to warm storage access.
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "AccessList::is_empty",
            deserialize_with = "deserialize_null_default"
        )
    )]
    pub access_list: AccessList,
    /// EIP-2718 type
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            rename = "type",
            skip_serializing_if = "Option::is_none",
            with = "uint_to_hex",
        )
    )]
    pub transaction_type: Option<u64>,
}

impl TryFrom<RpcTransaction> for LegacyTransaction {
    type Error = &'static str;

    fn try_from(tx: RpcTransaction) -> Result<Self, Self::Error> {
        if let Some(transaction_type) = tx.transaction_type {
            if transaction_type != 0 {
                return Err("transaction type is not 0");
            }
        }

        if !tx.access_list.is_empty() {
            return Err("legacy tx doesn't support access list");
        }
        if tx.max_fee_per_gas.is_some() {
            return Err("legacy tx doesn't support max_fee_per_gas");
        }
        if tx.max_priority_fee_per_gas.is_some() {
            return Err("legacy tx doesn't support max_priority_fee_per_gas");
        }
        let Some(gas_price) = tx.gas_price else {
            return Err("legacy tx gas_price is mandatory");
        };

        let chain_id = if tx.signature.r.is_zero() && tx.signature.s.is_zero() {
            tx.chain_id.or_else(|| tx.signature.v.chain_id())
        } else {
            tx.signature.v.chain_id()
        };

        Ok(Self {
            nonce: tx.nonce,
            gas_price,
            gas_limit: u64::try_from(tx.gas_limit).unwrap_or(u64::MAX),
            to: tx.to,
            value: tx.value,
            data: tx.input,
            chain_id,
        })
    }
}

impl TryFrom<RpcTransaction> for Eip2930Transaction {
    type Error = &'static str;

    fn try_from(tx: RpcTransaction) -> Result<Self, Self::Error> {
        if let Some(transaction_type) = tx.transaction_type {
            if transaction_type != 1 {
                return Err("transaction type is not 0");
            }
        }

        if tx.max_fee_per_gas.is_some() {
            return Err("EIP2930 Tx doesn't support max_fee_per_gas");
        }
        if tx.max_priority_fee_per_gas.is_some() {
            return Err("EIP2930 Tx doesn't support max_priority_fee_per_gas");
        }
        let Some(chain_id) = tx.chain_id else {
            return Err("chain_id is mandatory for EIP2930 transactions");
        };
        let Some(gas_price) = tx.gas_price else {
            return Err("gas_price is mandatory for EIP2930 transactions");
        };

        Ok(Self {
            nonce: tx.nonce,
            gas_price,
            gas_limit: u64::try_from(tx.gas_limit).unwrap_or(u64::MAX),
            to: tx.to,
            value: tx.value,
            data: tx.input,
            chain_id,
            access_list: tx.access_list,
        })
    }
}

impl TryFrom<RpcTransaction> for Eip1559Transaction {
    type Error = &'static str;

    fn try_from(tx: RpcTransaction) -> Result<Self, Self::Error> {
        if let Some(transaction_type) = tx.transaction_type {
            if transaction_type != 2 {
                return Err("transaction type is not 0");
            }
        }

        let Some(chain_id) = tx.chain_id else {
            return Err("chain_id is mandatory for EIP1559 transactions");
        };
        let Some(max_fee_per_gas) = tx.max_fee_per_gas else {
            return Err("max_fee_per_gas is mandatory for EIP1559 transactions");
        };
        let Some(max_priority_fee_per_gas) = tx.max_priority_fee_per_gas else {
            return Err("max_priority_fee_per_gas is mandatory for EIP1559 transactions");
        };

        Ok(Self {
            nonce: tx.nonce,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            gas_limit: u64::try_from(tx.gas_limit).unwrap_or(u64::MAX),
            to: tx.to,
            value: tx.value,
            data: tx.input,
            chain_id,
            access_list: tx.access_list,
        })
    }
}

impl TryFrom<RpcTransaction> for TypedTransaction {
    type Error = &'static str;

    fn try_from(tx: RpcTransaction) -> Result<Self, Self::Error> {
        let typed_tx = match tx.transaction_type {
            Some(0) => Self::Legacy(tx.try_into()?),
            Some(1) => Self::Eip2930(tx.try_into()?),
            Some(2) => Self::Eip1559(tx.try_into()?),
            Some(_) => return Err("unknown transaction type"),
            None => {
                if tx.max_fee_per_gas.is_some() || tx.max_priority_fee_per_gas.is_some() {
                    Self::Eip1559(tx.try_into()?)
                } else {
                    Self::Legacy(tx.try_into()?)
                }
            },
        };
        Ok(typed_tx)
    }
}

impl TryFrom<RpcTransaction> for SignedTransaction<TypedTransaction> {
    type Error = &'static str;

    fn try_from(tx: RpcTransaction) -> Result<Self, Self::Error> {
        let tx_hash = tx.hash;
        let signature = tx.signature;
        let payload = match tx.transaction_type {
            Some(0) => TypedTransaction::Legacy(tx.try_into()?),
            Some(1) => TypedTransaction::Eip2930(tx.try_into()?),
            Some(2) => TypedTransaction::Eip1559(tx.try_into()?),
            Some(_) => return Err("unknown transaction type"),
            None => {
                if tx.max_fee_per_gas.is_some() || tx.max_priority_fee_per_gas.is_some() {
                    TypedTransaction::Eip1559(tx.try_into()?)
                } else if tx.access_list.is_empty() {
                    TypedTransaction::Legacy(tx.try_into()?)
                } else {
                    TypedTransaction::Eip2930(tx.try_into()?)
                }
            },
        };
        Ok(Self { tx_hash, payload, signature })
    }
}

#[cfg(all(test, feature = "serde", feature = "with-rlp", feature = "with-crypto"))]
mod tests {
    use super::RpcTransaction;
    use crate::{
        bytes::Bytes,
        eth_hash::Address,
        transactions::{access_list::AccessList, signature::Signature},
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
            nonce: 810,
            block_hash: Some(
                hex!("dbdb6ab6ef116b498ceab7141a8ab1646960e2550bafbe3e8e22f1daffacc7cf").into(),
            ),
            block_number: Some(87936),
            transaction_index: Some(0),
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
            signature: Signature {
                v: 0x1c.into(),
                r: hex!("c8fc04e29b0859a7f265b67af7d4c5c6bc9e3d5a8de4950f89fa71a12a3cf8ae").into(),
                s: hex!("7dd15a10f9f2c8d1519a6044d880d04756798fc23923ff94f4823df8dc5b987a").into(),
            },
            access_list: AccessList::default(),
            transaction_type: Some(0),
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
            nonce: 7370,
            block_hash: Some(hex!("fdee00b60ddb4fd465426871a247ca905ff2acd5425b2222ab495157038772f3").into()),
            block_number: Some(18_529_320),
            transaction_index: Some(95),
            gas_price: Some(49_869_264_832_u64.into()),
            gas_limit: 0x2335e.into(),
            from: Address::from(hex!("1e8c05fa1e52adcb0b66808fa7b843d106f506d5")),
            to: Some(Address::from(hex!("00005ea00ac477b1030ce78506496e8c2de24bf5"))),
            value: 16_000_000_000_000_000u128.into(),
            input: hex!("161ac21f0000000000000000000000001fe1ffffef6b4dca417d321ccd37e081f604d1c70000000000000000000000000000a26b00c1f0df003000390027140000faa71900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002360c6ebe").into(),
            chain_id: Some(1),
            max_priority_fee_per_gas: Some(100_000_000.into()),
            max_fee_per_gas: Some(50_984_458_450_u64.into()),
            creates: None,
            raw: None,
            public_key: None,
            signature: Signature {
                v: 0x0.into(),
                r: hex!("8623bae9c86fb05f96cebd0f07247afc363f0ed3e1cf381ef99277ebf2b6c84a").into(),
                s: hex!("766ba586a5aac2769cf5ce9e3c6fccf01ad6c57eeefc3770e4a2f49516837ae2").into(),
            },
            access_list: AccessList::default(),
            transaction_type: Some(2),
        };
        let actual = serde_json::from_str::<RpcTransaction>(json).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn decode_astar_json_works() {
        let json = r#"
        {
            "hash": "0x543865875066b0c3b7039866deb8666c7740f83cc8a920b6b261cf30db1e6bdb",
            "nonce": "0x71f1",
            "blockHash": "0x73f9f64e13cf96569683db7eb494d52dcb52a98feae0b0519663d0c92702f3d2",
            "blockNumber": "0x4a3b18",
            "transactionIndex": "0x0",
            "from": "0x530de54355b619bd9b3b46ab5054933b72ca8cc0",
            "to": "0xa55d9ef16af921b70fed1421c1d298ca5a3a18f1",
            "value": "0x0",
            "gasPrice": "0x3b9aca000",
            "gas": "0x61a80",
            "input": "0x3798c7f200000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000006551475800000000000000000000000000000000000000000000000000000000014a139f0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000004415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000054d415449430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000045ff8a5800000000000000000000000000000000000000000000000000000000036e5f480",
            "creates": null,
            "raw": "0xf9022f8271f18503b9aca00083061a8094a55d9ef16af921b70fed1421c1d298ca5a3a18f180b901c43798c7f200000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000006551475800000000000000000000000000000000000000000000000000000000014a139f0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000004415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000054d415449430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000045ff8a5800000000000000000000000000000000000000000000000000000000036e5f4808204c4a04c58b0730a3487da33a44b7b501387fa48d6a6339d32ff520bcefc1da16945c1a062fb6b5c6c631b8d5205d59c0716c973995b47eb1eb329100e790a0957bff72c",
            "publicKey": "0x75159f240a12daf62cd20487a6dca0093a6e8a139dacf8f8888fe582a1d08ae423f742a04b82579083e86c1b78104c7137e211be1d396a1c3c14fa840d9e094a",
            "chainId": "0x250",
            "standardV": "0x1",
            "v": "0x4c4",
            "r": "0x4c58b0730a3487da33a44b7b501387fa48d6a6339d32ff520bcefc1da16945c1",
            "s": "0x62fb6b5c6c631b8d5205d59c0716c973995b47eb1eb329100e790a0957bff72c",
            "accessList": null,
            "type": "0x0"
        }
        "#;
        let expected = RpcTransaction {
            hash: hex!("543865875066b0c3b7039866deb8666c7740f83cc8a920b6b261cf30db1e6bdb").into(),
            nonce: 0x71f1,
            block_hash: Some(hex!("73f9f64e13cf96569683db7eb494d52dcb52a98feae0b0519663d0c92702f3d2").into()),
            block_number: Some(0x004a_3b18),
            transaction_index: Some(0x0),
            gas_price: Some(0x0003_b9ac_a000_u64.into()),
            gas_limit: 0x61a80.into(),
            from: Address::from(hex!("530de54355b619bd9b3b46ab5054933b72ca8cc0")),
            to: Some(Address::from(hex!("a55d9ef16af921b70fed1421c1d298ca5a3a18f1"))),
            value: 0.into(),
            input: hex!("3798c7f200000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000006551475800000000000000000000000000000000000000000000000000000000014a139f0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000004415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000054d415449430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000045ff8a5800000000000000000000000000000000000000000000000000000000036e5f480").into(),
            chain_id: Some(0x250),
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            creates: None,
            raw: Some(hex!("f9022f8271f18503b9aca00083061a8094a55d9ef16af921b70fed1421c1d298ca5a3a18f180b901c43798c7f200000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000006551475800000000000000000000000000000000000000000000000000000000014a139f0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000004415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000054d415449430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000045ff8a5800000000000000000000000000000000000000000000000000000000036e5f4808204c4a04c58b0730a3487da33a44b7b501387fa48d6a6339d32ff520bcefc1da16945c1a062fb6b5c6c631b8d5205d59c0716c973995b47eb1eb329100e790a0957bff72c").into()),
            public_key: Some(hex!("75159f240a12daf62cd20487a6dca0093a6e8a139dacf8f8888fe582a1d08ae423f742a04b82579083e86c1b78104c7137e211be1d396a1c3c14fa840d9e094a").into()),
            signature: Signature {
                v: 0x4c4.into(),
                r: hex!("4c58b0730a3487da33a44b7b501387fa48d6a6339d32ff520bcefc1da16945c1").into(),
                s: hex!("62fb6b5c6c631b8d5205d59c0716c973995b47eb1eb329100e790a0957bff72c").into(),
            },
            access_list: AccessList::default(),
            transaction_type: Some(0),
        };
        let actual = serde_json::from_str::<RpcTransaction>(json).unwrap();
        assert_eq!(expected, actual);
    }
}
