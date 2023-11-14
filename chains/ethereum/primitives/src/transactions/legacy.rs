#![allow(clippy::missing_errors_doc)]
#[cfg(feature = "with-serde")]
use crate::serde_utils::{deserialize_uint, serialize_uint};

use crate::{bytes::Bytes, eth_hash::Address, eth_uint::U256};

#[cfg(feature = "with-crypto")]
use crate::{
    crypto::{Crypto, DefaultCrypto},
    eth_hash::H256,
};

#[cfg(feature = "with-rlp")]
use crate::{
    rlp_utils::{RlpDecodableTransaction, RlpEncodableTransaction, RlpExt, RlpStreamExt},
    transactions::signature::{RecoveryId, Signature},
};

/// Legacy transaction that use the transaction format existing before typed transactions were
/// introduced in EIP-2718. Legacy transactions don’t use access lists or incorporate EIP-1559 fee
/// market changes.
#[derive(Clone, Default, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct LegacyTransaction {
    /// The nonce of the transaction. If set to `None`, no checks are performed.
    #[cfg_attr(
        feature = "with-serde",
        serde(deserialize_with = "deserialize_uint", serialize_with = "serialize_uint",)
    )]
    pub nonce: u64,

    /// Gas price
    pub gas_price: U256,

    /// Supplied gas
    #[cfg_attr(
        feature = "with-serde",
        serde(
            rename = "gas",
            deserialize_with = "deserialize_uint",
            serialize_with = "serialize_uint",
        )
    )]
    pub gas_limit: u64,

    /// Recipient address (None for contract creation)
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub to: Option<Address>,

    /// Transferred value
    pub value: U256,

    /// The data of the transaction.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Bytes::is_empty"))]
    pub data: Bytes,

    /// The chain ID of the transaction. If set to `None`, no checks are performed.
    ///
    /// Incorporated as part of the Spurious Dragon upgrade via [EIP-155].
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    #[cfg_attr(
        feature = "with-serde",
        serde(
            skip_serializing_if = "Option::is_none",
            deserialize_with = "deserialize_uint",
            serialize_with = "serialize_uint",
        )
    )]
    pub chain_id: Option<u64>,
}

#[cfg(feature = "with-rlp")]
impl RlpDecodableTransaction for LegacyTransaction {
    fn rlp_decode(
        rlp: &rlp::Rlp,
        _decode_signature: bool,
    ) -> Result<(Self, Option<Signature>), rlp::DecoderError> {
        let is_signed = match rlp.item_count()? {
            6 => false,
            9 => true,
            _ => return Err(rlp::DecoderError::RlpIncorrectListLen),
        };

        // Decode transaction
        let mut tx = Self {
            nonce: rlp.val_at(0usize)?,
            gas_price: rlp.val_at(1usize)?,
            gas_limit: rlp.val_at(2usize)?,
            to: rlp.opt_at(3usize)?,
            value: rlp.val_at(4usize)?,
            data: rlp.val_at(5usize)?,
            chain_id: None,
        };

        // The last 3 are the chain ID and signature
        let signature = if is_signed {
            let v = rlp.at(6usize)?;
            let r = rlp.at(7usize)?;
            let s = rlp.at(8usize)?;

            // r and s is empty, then v is the chain_id
            // [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
            if r.is_empty() && s.is_empty() {
                tx.chain_id = Some(<u64 as rlp::Decodable>::decode(&v)?);
                None
            } else {
                let signature = Signature {
                    v: <RecoveryId as rlp::Decodable>::decode(&v)?,
                    r: <U256 as rlp::Decodable>::decode(&r)?,
                    s: <U256 as rlp::Decodable>::decode(&s)?,
                };
                tx.chain_id = signature.v.chain_id();
                Some(signature)
            }
        } else {
            None
        };

        Ok((tx, signature))
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Decodable for LegacyTransaction {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        RlpDecodableTransaction::rlp_decode_unsigned(rlp)
    }
}

#[cfg(feature = "with-rlp")]
impl RlpEncodableTransaction for LegacyTransaction {
    fn rlp_append(&self, stream: &mut rlp::RlpStream, signature: Option<&Signature>) {
        let mut num_fields = 6;
        if self.chain_id.is_some() || signature.is_some() {
            num_fields += 3;
        }

        stream
            .begin_list(num_fields)
            .append(&self.nonce)
            .append(&self.gas_price)
            .append(&self.gas_limit)
            .append_opt(self.to.as_ref())
            .append(&self.value)
            .append(&self.data);

        match (self.chain_id, signature) {
            (Some(chain_id), Some(sig)) => {
                debug_assert_eq!(Some(chain_id), sig.v.chain_id());
                // let v = sig.v.as_eip155(chain_id.as_u64());
                stream.append(&sig.v).append(&sig.r).append(&sig.s);
            },
            (None, Some(sig)) => {
                debug_assert_eq!(sig.v.chain_id(), None);
                stream.append(&sig.v).append(&sig.r).append(&sig.s);
            },
            (Some(chain_id), None) => {
                stream.append(&chain_id).append(&0u8).append(&0u8);
            },
            (None, None) => {},
        }
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Encodable for LegacyTransaction {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        RlpEncodableTransaction::rlp_append(self, s, None);
    }
}

#[cfg(feature = "with-crypto")]
impl super::TransactionT for LegacyTransaction {
    type ExtraFields = ();

    fn encode(&self, signature: Option<&Signature>) -> Bytes {
        let bytes = signature.map_or_else(
            || RlpEncodableTransaction::rlp_unsigned(self),
            |signature| RlpEncodableTransaction::rlp_signed(self, signature),
        );
        Bytes(bytes)
    }

    /// The hash of the transaction without signature
    fn sighash(&self) -> H256 {
        let bytes = RlpEncodableTransaction::rlp_unsigned(self);
        DefaultCrypto::keccak256(bytes.as_ref())
    }

    // Compute the tx-hash using the provided signature
    fn compute_tx_hash(&self, signature: &Signature) -> H256 {
        let bytes = RlpEncodableTransaction::rlp_signed(self, signature);
        DefaultCrypto::keccak256(bytes.as_ref())
    }

    fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn gas_price(&self) -> super::GasPrice {
        super::GasPrice::Legacy(self.gas_price)
    }

    fn gas_limit(&self) -> U256 {
        self.gas_limit.into()
    }

    fn to(&self) -> Option<Address> {
        self.to
    }

    fn value(&self) -> U256 {
        self.value
    }

    fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    fn access_list(&self) -> Option<&super::AccessList> {
        None
    }

    fn transaction_type(&self) -> Option<u8> {
        Some(0x00)
    }

    fn extra_fields(&self) -> Option<Self::ExtraFields> {
        None
    }
}

#[cfg(all(test, any(feature = "with-serde", feature = "with-rlp")))]
mod tests {
    use super::LegacyTransaction;
    #[cfg(feature = "with-crypto")]
    use crate::eth_hash::H256;
    use crate::{
        bytes::Bytes,
        transactions::signature::{RecoveryId, Signature},
    };
    use hex_literal::hex;

    #[cfg(feature = "with-rlp")]
    static RLP_LEGACY_TX_SIGNED: &[u8] = &hex!("f86b820c5e850df8475800830493e0946b92c944c82c694725acbd1c000c277ea1a44f00808441c0e1b51ca0989506185a9ae63f316a850ecba0c2446a8d42bd77afcddbdd001118194f5d79a02c8e3dd2b351426b735c8e818ea975887957b05fb591017faad7d75add9feb0f");
    #[cfg(feature = "with-rlp")]
    static RLP_LEGACY_TX_UNSIGNED: &[u8] =
        &hex!("e8820c5e850df8475800830493e0946b92c944c82c694725acbd1c000c277ea1a44f00808441c0e1b5");

    #[cfg(feature = "with-rlp")]
    static RLP_EIP155_TX_SIGNED: &[u8] = &hex!("f904b481898504bfef4c00830f424094dc6c91b569c98f9f6f74d90f9beff99fdaf4248b8803dd2c5609333800b90444288b8133920339b815ee42a02099dcca27c01d192418334751613a1eea786a0c3a673cec000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003c0000000000000000000000000000000000000000000000000000000000000032464a3bc15000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000003dd2c560933380000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000b14232b0204b2f7bb6ba5aff59ef36030f7fe38b00000000000000000000000041f8d14c9475444f30a80431c68cf24dc9a8369a000000000000000000000000b9e29984fe50602e7a619662ebed4f90d93824c7000000000000000000000000dc6c91b569c98f9f6f74d90f9beff99fdaf4248b0000000000000000000000000000000000000000000000000000000002faf08000000000000000000000000000000000000000000000000003dd2c560933380000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005d7625241afaa81faf3c2bd525f64f6e0ec3af39c1053d672b65c2f64992521e6f454e67000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000000000000000000000000024f47261b0000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024f47261b0000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000581b96d32320d6cb174e807b585a41f4faa8ba7da95e117f2abbcadbb257d37a5fcc16c2ba6db86200888ed85dd5eba547bb07fa0f9910950d3133026abafdd5c09e1f3896a7abb3c99e1f38f77be69448ee7770d18c001e0400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000561c7dd7d43db98e3b7a6e18b4e97f0e254a5a6bb9b373d49d7e6676ccb1b02d50f131bca36928cb48cd0daec0499e9e93a390253c733607437bbebf0aa13b7080911f3896a7abb3c99e1f38f77be69448ee7770d18c040000000000000000000025a0020d7064f0b3c956e603c994fd83247499ede5a1209d6c997d2b2ea29b5627a7a06f6c3ceb0a57952386cbb9ceb3e4d05f1d4bc8d30b67d56281d89775f972a34d");
    #[cfg(feature = "with-rlp")]
    static RLP_EIP155_TX_UNSIGNED: &[u8] = &hex!("f9047481898504bfef4c00830f424094dc6c91b569c98f9f6f74d90f9beff99fdaf4248b8803dd2c5609333800b90444288b8133920339b815ee42a02099dcca27c01d192418334751613a1eea786a0c3a673cec000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003c0000000000000000000000000000000000000000000000000000000000000032464a3bc15000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000003dd2c560933380000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000b14232b0204b2f7bb6ba5aff59ef36030f7fe38b00000000000000000000000041f8d14c9475444f30a80431c68cf24dc9a8369a000000000000000000000000b9e29984fe50602e7a619662ebed4f90d93824c7000000000000000000000000dc6c91b569c98f9f6f74d90f9beff99fdaf4248b0000000000000000000000000000000000000000000000000000000002faf08000000000000000000000000000000000000000000000000003dd2c560933380000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005d7625241afaa81faf3c2bd525f64f6e0ec3af39c1053d672b65c2f64992521e6f454e67000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000000000000000000000000024f47261b0000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024f47261b0000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000581b96d32320d6cb174e807b585a41f4faa8ba7da95e117f2abbcadbb257d37a5fcc16c2ba6db86200888ed85dd5eba547bb07fa0f9910950d3133026abafdd5c09e1f3896a7abb3c99e1f38f77be69448ee7770d18c001e0400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000561c7dd7d43db98e3b7a6e18b4e97f0e254a5a6bb9b373d49d7e6676ccb1b02d50f131bca36928cb48cd0daec0499e9e93a390253c733607437bbebf0aa13b7080911f3896a7abb3c99e1f38f77be69448ee7770d18c0400000000000000000000018080");

    fn build_legacy(eip155: bool) -> (LegacyTransaction, Signature) {
        if eip155 {
            let tx = LegacyTransaction {
                chain_id: Some(1),
                nonce: 137,
                gas_price: 20_400_000_000u64.into(),
                gas_limit: 1_000_000,
                to: Some(hex!("dc6c91b569c98f9f6f74d90f9beff99fdaf4248b").into()),
                value: 278_427_500_000_000_000u64.into(),
                data: Bytes::from(hex!("288b8133920339b815ee42a02099dcca27c01d192418334751613a1eea786a0c3a673cec000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003c0000000000000000000000000000000000000000000000000000000000000032464a3bc15000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000003dd2c560933380000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000b14232b0204b2f7bb6ba5aff59ef36030f7fe38b00000000000000000000000041f8d14c9475444f30a80431c68cf24dc9a8369a000000000000000000000000b9e29984fe50602e7a619662ebed4f90d93824c7000000000000000000000000dc6c91b569c98f9f6f74d90f9beff99fdaf4248b0000000000000000000000000000000000000000000000000000000002faf08000000000000000000000000000000000000000000000000003dd2c560933380000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005d7625241afaa81faf3c2bd525f64f6e0ec3af39c1053d672b65c2f64992521e6f454e67000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000000000000000000000000024f47261b0000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024f47261b0000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000581b96d32320d6cb174e807b585a41f4faa8ba7da95e117f2abbcadbb257d37a5fcc16c2ba6db86200888ed85dd5eba547bb07fa0f9910950d3133026abafdd5c09e1f3896a7abb3c99e1f38f77be69448ee7770d18c001e0400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000561c7dd7d43db98e3b7a6e18b4e97f0e254a5a6bb9b373d49d7e6676ccb1b02d50f131bca36928cb48cd0daec0499e9e93a390253c733607437bbebf0aa13b7080911f3896a7abb3c99e1f38f77be69448ee7770d18c0400000000000000000000")),
            };
            let signature = Signature {
                v: RecoveryId::try_from(0x25).unwrap(),
                r: hex!("020d7064f0b3c956e603c994fd83247499ede5a1209d6c997d2b2ea29b5627a7").into(),
                s: hex!("6f6c3ceb0a57952386cbb9ceb3e4d05f1d4bc8d30b67d56281d89775f972a34d").into(),
            };
            (tx, signature)
        } else {
            let tx = LegacyTransaction {
                chain_id: None,
                nonce: 3166,
                gas_price: 60_000_000_000u64.into(),
                gas_limit: 300_000,
                to: Some(hex!("6b92c944c82c694725acbd1c000c277ea1a44f00").into()),
                value: 0.into(),
                data: hex!("41c0e1b5").into(),
            };
            let signature = Signature {
                v: RecoveryId::try_from(0x1c).unwrap(),
                r: hex!("989506185a9ae63f316a850ecba0c2446a8d42bd77afcddbdd001118194f5d79").into(),
                s: hex!("2c8e3dd2b351426b735c8e818ea975887957b05fb591017faad7d75add9feb0f").into(),
            };
            (tx, signature)
        }
    }

    #[cfg(feature = "with-serde")]
    #[test]
    fn serde_encode_works() {
        let tx = build_legacy(true).0;
        let actual = serde_json::to_value(&tx).unwrap();
        let expected = serde_json::json!({
            "nonce": "0x89",
            "gas": "0xf4240",
            "gasPrice": "0x4bfef4c00",
            "to": "0xdc6c91b569c98f9f6f74d90f9beff99fdaf4248b",
            "value": "0x3dd2c5609333800",
            "data": "0x288b8133920339b815ee42a02099dcca27c01d192418334751613a1eea786a0c3a673cec000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003c0000000000000000000000000000000000000000000000000000000000000032464a3bc15000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000003dd2c560933380000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000b14232b0204b2f7bb6ba5aff59ef36030f7fe38b00000000000000000000000041f8d14c9475444f30a80431c68cf24dc9a8369a000000000000000000000000b9e29984fe50602e7a619662ebed4f90d93824c7000000000000000000000000dc6c91b569c98f9f6f74d90f9beff99fdaf4248b0000000000000000000000000000000000000000000000000000000002faf08000000000000000000000000000000000000000000000000003dd2c560933380000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005d7625241afaa81faf3c2bd525f64f6e0ec3af39c1053d672b65c2f64992521e6f454e67000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000000000000000000000000024f47261b0000000000000000000000000dac17f958d2ee523a2206206994597c13d831ec7000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000024f47261b0000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000581b96d32320d6cb174e807b585a41f4faa8ba7da95e117f2abbcadbb257d37a5fcc16c2ba6db86200888ed85dd5eba547bb07fa0f9910950d3133026abafdd5c09e1f3896a7abb3c99e1f38f77be69448ee7770d18c001e0400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000561c7dd7d43db98e3b7a6e18b4e97f0e254a5a6bb9b373d49d7e6676ccb1b02d50f131bca36928cb48cd0daec0499e9e93a390253c733607437bbebf0aa13b7080911f3896a7abb3c99e1f38f77be69448ee7770d18c0400000000000000000000",
            "chainId": "0x1",
        });
        assert_eq!(expected, actual);

        let tx = build_legacy(false).0;
        let actual = serde_json::to_value(&tx).unwrap();
        let expected: serde_json::Value = serde_json::json!({
            "gas": "0x493e0",
            "gasPrice": "0xdf8475800",
            "data": "0x41c0e1b5",
            "nonce": "0xc5e",
            "to": "0x6b92c944c82c694725acbd1c000c277ea1a44f00",
            "value": "0x0",
        });
        assert_eq!(expected, actual);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_encode_signed_works() {
        use crate::rlp_utils::RlpEncodableTransaction;
        let (tx, sig) = build_legacy(false);
        let expected = Bytes::from_static(RLP_LEGACY_TX_SIGNED);
        let actual = Bytes::from(tx.rlp_signed(&sig));
        assert_eq!(expected, actual);

        let (tx, sig) = build_legacy(true);
        let expected = Bytes::from_static(RLP_EIP155_TX_SIGNED);
        let actual = Bytes::from(tx.rlp_signed(&sig));
        assert_eq!(expected, actual);
    }

    #[cfg(feature = "with-crypto")]
    #[test]
    fn rlp_encode_astar_tx_works() {
        use crate::{rlp_utils::RlpEncodableTransaction, transactions::TransactionT};
        let expected = hex!("f9022f8271f18503b9aca00083061a8094a55d9ef16af921b70fed1421c1d298ca5a3a18f180b901c43798c7f200000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000006551475800000000000000000000000000000000000000000000000000000000014a139f0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000004415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000054d415449430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000045ff8a5800000000000000000000000000000000000000000000000000000000036e5f4808204c4a04c58b0730a3487da33a44b7b501387fa48d6a6339d32ff520bcefc1da16945c1a062fb6b5c6c631b8d5205d59c0716c973995b47eb1eb329100e790a0957bff72c");
        let tx = LegacyTransaction {
            nonce: 0x71f1,
            gas_price: 0x0003_b9ac_a000u128.into(),
            gas_limit: 0x61a80,
            to: Some(hex!("a55d9ef16af921b70fed1421c1d298ca5a3a18f1").into()),
            value: 0.into(),
            data: hex!("3798c7f200000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000006551475800000000000000000000000000000000000000000000000000000000014a139f0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000004415641580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000054d415449430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000045ff8a5800000000000000000000000000000000000000000000000000000000036e5f480").into(),
            chain_id: Some(0x250),
        };
        let sig = Signature {
            v: 0x4c4.into(),
            r: hex!("4c58b0730a3487da33a44b7b501387fa48d6a6339d32ff520bcefc1da16945c1").into(),
            s: hex!("62fb6b5c6c631b8d5205d59c0716c973995b47eb1eb329100e790a0957bff72c").into(),
        };
        let expected = Bytes::from(&expected);
        let actual = Bytes::from(tx.rlp_signed(&sig));
        assert_eq!(expected, actual);

        let expected =
            H256(hex!("543865875066b0c3b7039866deb8666c7740f83cc8a920b6b261cf30db1e6bdb"));
        let tx_hash = tx.compute_tx_hash(&sig);
        assert_eq!(expected, tx_hash);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_encode_unsigned_works() {
        use crate::rlp_utils::RlpEncodableTransaction;
        let tx = build_legacy(false).0;
        let expected = Bytes::from_static(RLP_LEGACY_TX_UNSIGNED);
        let actual = Bytes::from(tx.rlp_unsigned());
        assert_eq!(expected, actual);

        let tx: LegacyTransaction = build_legacy(true).0;
        let expected = Bytes::from_static(RLP_EIP155_TX_UNSIGNED);
        let actual = Bytes::from(tx.rlp_unsigned());
        assert_eq!(expected, actual);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_decode_signed_works() {
        use crate::rlp_utils::RlpDecodableTransaction;
        let (expected_tx, expected_sig) = build_legacy(false);
        let (actual_tx, actual_sig) = {
            let rlp = rlp::Rlp::new(RLP_LEGACY_TX_SIGNED);
            LegacyTransaction::rlp_decode_signed(&rlp).unwrap()
        };
        assert_eq!(expected_tx, actual_tx);
        assert_eq!(Some(expected_sig), actual_sig);

        let (expected_tx, expected_sig) = build_legacy(true);
        let (actual_tx, actual_sig) = {
            let rlp = rlp::Rlp::new(RLP_EIP155_TX_SIGNED);
            LegacyTransaction::rlp_decode_signed(&rlp).unwrap()
        };
        assert_eq!(expected_tx, actual_tx);
        assert_eq!(Some(expected_sig), actual_sig);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_decode_unsigned_works() {
        use crate::rlp_utils::RlpDecodableTransaction;
        // Can decode unsigned raw transaction
        let expected = build_legacy(false).0;
        let actual = {
            let rlp = rlp::Rlp::new(RLP_LEGACY_TX_UNSIGNED);
            LegacyTransaction::rlp_decode_unsigned(&rlp).unwrap()
        };
        assert_eq!(expected, actual);

        // Can decode eip155 raw transaction
        let expected = build_legacy(true).0;
        let actual = {
            let rlp = rlp::Rlp::new(RLP_EIP155_TX_UNSIGNED);
            LegacyTransaction::rlp_decode_unsigned(&rlp).unwrap()
        };
        assert_eq!(expected, actual);
    }

    #[cfg(feature = "with-crypto")]
    #[test]
    fn compute_legacy_sighash() {
        use super::super::TransactionT;
        use crate::eth_hash::H256;

        let tx = build_legacy(false).0;
        let expected =
            H256(hex!("c8519e5053848e75bc9c6dc20710410d56c9186b486a9b27900eb3355fed085e"));
        assert_eq!(expected, tx.sighash());

        let tx = build_legacy(true).0;
        let expected =
            H256(hex!("bb88aee10d01fe0a01135bf346a6eba268e1c5f3ab3e3045c14a97b02245f90f"));
        assert_eq!(expected, tx.sighash());
    }

    #[cfg(feature = "with-crypto")]
    #[test]
    fn compute_legacy_tx_hash() {
        use super::super::TransactionT;
        use crate::eth_hash::H256;

        let (tx, sig) = build_legacy(false);
        let expected =
            H256(hex!("5a2dbc3b236ddf99c6a380a1a057023ff5d2f35ada1e38b5cbe125ee87cd4777"));
        assert_eq!(expected, tx.compute_tx_hash(&sig));

        let (tx, sig) = build_legacy(true);
        let expected =
            H256(hex!("df99f8176f765d84ed1c00a12bba00206c6da97986c802a532884aca5aaa3809"));
        assert_eq!(expected, tx.compute_tx_hash(&sig));
    }
}
