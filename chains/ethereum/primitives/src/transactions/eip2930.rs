#![allow(clippy::missing_errors_doc)]

use super::{access_list::AccessList, signature::Signature};
use crate::{
    bytes::Bytes,
    eth_hash::Address,
    eth_uint::{U256, U64},
};

#[cfg(feature = "with-rlp")]
use crate::rlp_utils::{RlpDecodableTransaction, RlpEncodableTransaction, RlpExt, RlpStreamExt};

/// Transactions with type 0x1 are transactions introduced in EIP-2930. They contain, along with the
/// legacy parameters, an access list which specifies an array of addresses and storage keys that
/// the transaction plans to access (an access list)
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
pub struct Eip2930Transaction {
    /// The chain ID of the transaction. It is mandatory for EIP-2930 transactions.
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    /// [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    pub chain_id: U64,

    /// The nonce of the transaction.
    pub nonce: U64,

    /// Gas price
    pub gas_price: U256,

    /// Supplied gas
    #[cfg_attr(feature = "with-serde", serde(rename = "gas"))]
    pub gas_limit: U64,

    /// Recipient address (None for contract creation)
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub to: Option<Address>,

    /// Transferred value
    pub value: U256,

    /// The data of the transaction.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Bytes::is_empty"))]
    pub data: Bytes,

    /// Optional access list introduced in EIP-2930.
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    #[cfg_attr(
        feature = "with-serde",
        serde(default, skip_serializing_if = "AccessList::is_empty")
    )]
    pub access_list: AccessList,
}

#[cfg(feature = "with-rlp")]
impl RlpDecodableTransaction for Eip2930Transaction {
    fn rlp_decode(
        rlp: &rlp::Rlp,
        decode_signature: bool,
    ) -> Result<(Self, Option<Signature>), rlp::DecoderError> {
        let first = *rlp.data()?.first().ok_or(rlp::DecoderError::RlpIsTooShort)?;

        // Verify EIP-2930 transaction type (0x01)
        if first != 0x01 {
            return Err(rlp::DecoderError::Custom("invalid transaction type"));
        }

        let rest = rlp::Rlp::new(
            rlp.as_raw()
                .get(1..)
                .ok_or(rlp::DecoderError::Custom("missing transaction payload"))?,
        );

        // Check if is signed
        let is_signed = match rest.item_count()? {
            8 => false,
            11 => true,
            _ => return Err(rlp::DecoderError::RlpIncorrectListLen),
        };

        // Decode transaction
        let tx = Self {
            chain_id: rest.val_at(0usize)?,
            nonce: rest.val_at(1usize)?,
            gas_price: rest.val_at(2usize)?,
            gas_limit: rest.val_at(3usize)?,
            to: rest.opt_at(4usize)?,
            value: rest.val_at(5usize)?,
            data: rest.val_at(6usize)?,
            access_list: rest.val_at(7usize)?,
        };

        // Decode signature
        let signature = if is_signed && decode_signature {
            Some(Signature {
                v: rest.val_at(8usize)?,
                r: rest.val_at(9usize)?,
                s: rest.val_at(10usize)?,
            })
        } else {
            None
        };

        Ok((tx, signature))
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Decodable for Eip2930Transaction {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        <Self as RlpDecodableTransaction>::rlp_decode_unsigned(rlp)
    }
}

#[cfg(feature = "with-rlp")]
impl RlpEncodableTransaction for Eip2930Transaction {
    fn rlp_append(&self, stream: &mut rlp::RlpStream, signature: Option<&Signature>) {
        // Append EIP-2930 transaction type (0x01)
        stream.append_internal(&1u8);
        let mut num_fields = 8;
        if signature.is_some() {
            num_fields += 3;
        }

        stream
            .begin_list(num_fields)
            .append(&self.chain_id)
            .append(&self.nonce)
            .append(&self.gas_price)
            .append(&self.gas_limit)
            .append_opt(self.to.as_ref())
            .append(&self.value)
            .append(&self.data)
            .append(&self.access_list);

        if let Some(sig) = signature {
            let v = sig.v.y_parity();
            stream.append(&v).append(&sig.r).append(&sig.s);
        }
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Encodable for Eip2930Transaction {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        RlpEncodableTransaction::rlp_append(self, s, None);
    }
}

#[cfg(all(feature = "with-rlp", feature = "with-crypto"))]
impl super::TransactionT for Eip2930Transaction {
    type ExtraFields = ();

    fn compute_tx_hash(&self, signature: &Signature) -> primitive_types::H256 {
        use sha3::Digest;
        let input = self.rlp_signed(signature);
        let hash: [u8; 32] = sha3::Keccak256::digest(input.as_ref()).into();
        crate::eth_hash::H256(hash)
    }

    fn chain_id(&self) -> Option<u64> {
        Some(self.chain_id.as_u64())
    }

    fn nonce(&self) -> u64 {
        self.nonce.as_u64()
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

    fn sighash(&self) -> crate::eth_hash::H256 {
        use sha3::Digest;
        let input = self.rlp_unsigned();
        let hash: [u8; 32] = sha3::Keccak256::digest(input.as_ref()).into();
        crate::eth_hash::H256(hash)
    }

    fn access_list(&self) -> Option<&AccessList> {
        Some(&self.access_list)
    }

    fn transaction_type(&self) -> Option<u8> {
        Some(0x01)
    }

    fn extra_fields(&self) -> Option<Self::ExtraFields> {
        None
    }
}

#[cfg(all(test, any(feature = "with-serde", feature = "with-rlp")))]
mod tests {
    use super::{super::signature::RecoveryId, Address, Bytes, Eip2930Transaction, Signature};
    use crate::{
        eth_hash::H256,
        transactions::access_list::{AccessList, AccessListItem},
    };
    use hex_literal::hex;

    #[cfg(feature = "with-rlp")]
    static RLP_EIP2930_SIGNED: &[u8] = &hex!("01f90372017585069b8cf27b8302db9d943fc91a3afd70395cd496c647d5a6cc9d4b2b7fad8832a767a9562d0000b902843593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000f87cf87a943fc91a3afd70395cd496c647d5a6cc9d4b2b7fadf863a00000000000000000000000000000000000000000000000000000000000000000a0a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6a0ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01a05fe8eb06ac27f44de3e8d1c7214f750b9fc8291ab63d71ea6a4456cfd328deb9a041425cc35a5ed1c922c898cb7fda5cf3b165b4792ada812700bf55cbc21a75a1");
    #[cfg(feature = "with-rlp")]
    static RLP_EIP2930_UNSIGNED: &[u8] = &hex!("01f9032f017585069b8cf27b8302db9d943fc91a3afd70395cd496c647d5a6cc9d4b2b7fad8832a767a9562d0000b902843593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000f87cf87a943fc91a3afd70395cd496c647d5a6cc9d4b2b7fadf863a00000000000000000000000000000000000000000000000000000000000000000a0a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6a0ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");

    fn build_eip2930() -> (Eip2930Transaction, Signature) {
        let tx = Eip2930Transaction {
            chain_id: 1.into(),
            nonce: 117.into(),
            gas_price: 28_379_509_371u128.into(),
            gas_limit: 187_293.into(),
            to: Some(hex!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad").into()),
            value: 3_650_000_000_000_000_000u128.into(),
            data: hex!("3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000").into(),
            access_list: AccessList(vec![AccessListItem {
                address: Address::from(hex!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad")),
                storage_keys: vec![
                    H256::zero(),
                    H256::from(hex!(
                        "a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6"
                    )),
                    H256::from(hex!(
                        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                    )),
                ],
            }]),
        };
        let signature = Signature {
            v: RecoveryId::new(0x01),
            r: hex!("5fe8eb06ac27f44de3e8d1c7214f750b9fc8291ab63d71ea6a4456cfd328deb9").into(),
            s: hex!("41425cc35a5ed1c922c898cb7fda5cf3b165b4792ada812700bf55cbc21a75a1").into(),
        };
        (tx, signature)
    }

    #[cfg(feature = "with-serde")]
    #[test]
    fn serde_encode_works() {
        let tx = build_eip2930().0;
        let actual = serde_json::to_value(&tx).unwrap();
        let expected = serde_json::json!({
            "chainId": "0x1",
            "nonce": "0x75",
            "gasPrice": "0x69b8cf27b",
            "gas": "0x2db9d",
            "to": "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad",
            "value": "0x32a767a9562d0000",
            "data": "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000",
            "accessList": [
                {
                    "address": "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad",
                    "storageKeys": [
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
                        "0xa19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6",
                        "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                    ]
                }
            ],
        });
        assert_eq!(expected, actual);

        // can decode json
        let json_str = serde_json::to_string(&tx).unwrap();
        let decoded = serde_json::from_str::<Eip2930Transaction>(&json_str).unwrap();
        assert_eq!(tx, decoded);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_encode_signed_works() {
        use crate::rlp_utils::RlpEncodableTransaction;
        let (tx, sig) = build_eip2930();
        let expected = Bytes::from_static(RLP_EIP2930_SIGNED);
        let actual = Bytes::from(tx.rlp_signed(&sig));
        assert_eq!(expected, actual);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_encode_unsigned_works() {
        use crate::rlp_utils::RlpEncodableTransaction;
        let tx = build_eip2930().0;
        let expected = Bytes::from_static(RLP_EIP2930_UNSIGNED);
        let actual = Bytes::from(tx.rlp_unsigned());
        assert_eq!(expected, actual);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_decode_signed_works() {
        use crate::rlp_utils::RlpDecodableTransaction;
        let (expected_tx, expected_sig) = build_eip2930();
        let (actual_tx, actual_sig) = {
            let rlp = rlp::Rlp::new(RLP_EIP2930_SIGNED);
            Eip2930Transaction::rlp_decode_signed(&rlp).unwrap()
        };
        assert_eq!(expected_tx, actual_tx);
        assert_eq!(Some(expected_sig), actual_sig);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_decode_unsigned_works() {
        use crate::rlp_utils::RlpDecodableTransaction;
        let expected = build_eip2930().0;

        // Can decode unsigned raw transaction
        let actual = {
            let rlp = rlp::Rlp::new(RLP_EIP2930_UNSIGNED);
            Eip2930Transaction::rlp_decode_unsigned(&rlp).unwrap()
        };
        assert_eq!(expected, actual);

        // Can decode signed raw transaction
        let actual = {
            let rlp = rlp::Rlp::new(RLP_EIP2930_SIGNED);
            Eip2930Transaction::rlp_decode_unsigned(&rlp).unwrap()
        };
        assert_eq!(expected, actual);
    }

    #[cfg(all(feature = "with-rlp", feature = "with-crypto"))]
    #[test]
    fn compute_eip2930_sighash() {
        use super::super::TransactionT;
        let tx = build_eip2930().0;
        let expected =
            H256(hex!("9af0ea823342c8b7755010d69e9c81fd11d487dbbaad02034757ff117f95f522"));
        assert_eq!(expected, tx.sighash());
    }

    #[cfg(all(feature = "with-rlp", feature = "with-crypto"))]
    #[test]
    fn compute_eip2930_tx_hash() {
        use super::super::TransactionT;
        let (tx, sig) = build_eip2930();
        let expected =
            H256(hex!("a777326ad77731344d00263b06843be6ef05cbe9ab699e2ed0d1448f8b2b50a3"));
        assert_eq!(expected, tx.compute_tx_hash(&sig));
    }
}
