#![allow(clippy::missing_errors_doc)]

use super::{access_list::AccessList, signature::Signature};
use crate::{
    bytes::Bytes,
    eth_hash::Address,
    eth_uint::{U256, U64},
};

#[cfg(feature = "with-rlp")]
use crate::rlp_utils::{RlpExt, RlpStreamExt};

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
    pub nonce: U256,

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

    /// The signature of the transaction. If set to `None`, no checks are performed.
    #[cfg_attr(feature = "with-serde", serde(flatten, skip_serializing_if = "Option::is_none"))]
    pub signature: Option<Signature>,
}

#[cfg(feature = "with-rlp")]
impl Eip2930Transaction {
    fn rlp_encode_internal(&self, stream: &mut rlp::RlpStream, signature: Option<&Signature>) {
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

    pub fn rlp_unsigned(&self, stream: &mut rlp::RlpStream) {
        self.rlp_encode_internal(stream, None);
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Encodable for Eip2930Transaction {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        self.rlp_encode_internal(s, self.signature.as_ref());
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Decodable for Eip2930Transaction {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
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

        // Decode signature
        let signature = match rest.item_count()? {
            8 => None,
            11 => {
                let sig = Signature {
                    v: rest.val_at(8usize)?,
                    r: rest.val_at(9usize)?,
                    s: rest.val_at(10usize)?,
                };
                debug_assert_eq!(
                    sig.v.y_parity(),
                    sig.v.as_u64(),
                    "invalid signature v value, must be 0 or 1"
                );
                Some(sig)
            },
            _ => return Err(rlp::DecoderError::RlpIncorrectListLen),
        };

        // Decode transaction
        Ok(Self {
            chain_id: rest.val_at(0usize)?,
            nonce: rest.val_at(1usize)?,
            gas_price: rest.val_at(2usize)?,
            gas_limit: rest.val_at(3usize)?,
            to: rest.opt_at(4usize)?,
            value: rest.val_at(5usize)?,
            data: rest.val_at(6usize)?,
            access_list: rest.val_at(7usize)?,
            signature,
        })
    }
}

#[cfg(all(test, any(feature = "with-serde", feature = "with-rlp")))]
mod tests {
    use super::{super::signature::RecoveryId, Address, Bytes, Eip2930Transaction, Signature};
    use crate::transactions::access_list::{AccessList, AccessListItem};

    #[cfg(feature = "with-serde")]
    #[test]
    fn serde_encode_works() {
        let tx = Eip2930Transaction {
            chain_id: 1.into(),
            nonce: 235.into(),
            gas_price: 25_670_917_490u128.into(),
            gas_limit: 114_756.into(),
            to: Some(hex_literal::hex!("8e5660b4ab70168b5a6feea0e0315cb49c8cd539").into()),
            value: 0.into(),
            data: hex_literal::hex!("6f652e1a000000000000000000000000959e104e1a4db6317fa58f8295f586e1a978c29700000000000000000000000000000000000000000000000000000000000008b800000000000000000000000000000000000000000000054b40b1f852bda000000000000000000000000000000000000000000000000000000000017723be2580").into(),
            access_list: AccessList::default(),
            signature: Some(Signature {
                v: RecoveryId::try_from(0x26).unwrap(),
                r: hex_literal::hex!("a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6").into(),
                s: hex_literal::hex!("4b88362cca36cc9742366ca474fd777328cb6ee012ceee2da9aa147761e17cbf").into(),
            }),
        };

        let actual = serde_json::to_value(&tx).unwrap();
        let expected = serde_json::json!({
            "gas": "0x1c044",
            "gasPrice": "0x5fa1b1d72",
            "data": "0x6f652e1a000000000000000000000000959e104e1a4db6317fa58f8295f586e1a978c29700000000000000000000000000000000000000000000000000000000000008b800000000000000000000000000000000000000000000054b40b1f852bda000000000000000000000000000000000000000000000000000000000017723be2580",
            "nonce": "0xeb",
            "to": "0x8e5660b4ab70168b5a6feea0e0315cb49c8cd539",
            "value": "0x0",
            "chainId": "0x1",
            "v": "0x26",
            "r": "0xa19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6",
            "s": "0x4b88362cca36cc9742366ca474fd777328cb6ee012ceee2da9aa147761e17cbf"
        });
        assert_eq!(expected, actual);

        // can decode json
        let json_str = serde_json::to_string(&tx).unwrap();
        let decoded = serde_json::from_str::<Eip2930Transaction>(&json_str).unwrap();
        assert_eq!(tx, decoded);

        // can decoded without signature
        let mut tx = tx;
        tx.signature = None;
        let json_str = serde_json::to_string(&tx).unwrap();
        let decoded = serde_json::from_str::<Eip2930Transaction>(&json_str).unwrap();
        assert_eq!(tx, decoded);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_codec_works() {
        use crate::eth_hash::H256;
        use rlp::{Decodable, Encodable};

        let tx = Eip2930Transaction {
            chain_id: 1.into(),
            nonce: 117.into(),
            gas_price: 28_379_509_371u128.into(),
            gas_limit: 187_293.into(),
            to: Some(hex_literal::hex!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad").into()),
            value: 3_650_000_000_000_000_000u128.into(),
            data: hex_literal::hex!("3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000").into(),
            access_list: AccessList(vec![AccessListItem {
                address: Address::from(hex_literal::hex!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad")),
                storage_keys: vec![
                    H256::zero(),
                    H256::from(hex_literal::hex!(
                        "a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6"
                    )),
                    H256::from(hex_literal::hex!(
                        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                    )),
                ],
            }]),
            signature: Some(Signature {
                v: RecoveryId::new(0x01),
                r: hex_literal::hex!("5fe8eb06ac27f44de3e8d1c7214f750b9fc8291ab63d71ea6a4456cfd328deb9").into(),
                s: hex_literal::hex!("41425cc35a5ed1c922c898cb7fda5cf3b165b4792ada812700bf55cbc21a75a1").into(),
            }),
        };

        // RLP encode works
        let expected = hex_literal::hex!("01f90372017585069b8cf27b8302db9d943fc91a3afd70395cd496c647d5a6cc9d4b2b7fad8832a767a9562d0000b902843593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000f87cf87a943fc91a3afd70395cd496c647d5a6cc9d4b2b7fadf863a00000000000000000000000000000000000000000000000000000000000000000a0a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6a0ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01a05fe8eb06ac27f44de3e8d1c7214f750b9fc8291ab63d71ea6a4456cfd328deb9a041425cc35a5ed1c922c898cb7fda5cf3b165b4792ada812700bf55cbc21a75a1");
        let actual = Bytes::from(Encodable::rlp_bytes(&tx).freeze());
        assert_eq!(Bytes::from(expected), actual);

        // RLP decode works
        let rlp = rlp::Rlp::new(expected.as_ref());
        let decoded = <Eip2930Transaction as Decodable>::decode(&rlp).unwrap();
        assert_eq!(tx, decoded);
    }
}
