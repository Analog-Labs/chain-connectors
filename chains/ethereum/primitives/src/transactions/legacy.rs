#![allow(clippy::missing_errors_doc)]
use primitive_types::H256;

use super::signature::{RecoveryId, Signature};
use crate::{
    bytes::Bytes,
    eth_hash::Address,
    eth_uint::{U256, U64},
};

#[cfg(feature = "with-rlp")]
use crate::rlp_utils::{RlpExt, RlpStreamExt};

/// Legacy transaction that use the transaction format existing before typed transactions were
/// introduced in EIP-2718. Legacy transactions don’t use access lists or incorporate EIP-1559 fee
/// market changes.
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
pub struct LegacyTransaction {
    /// The nonce of the transaction. If set to `None`, no checks are performed.
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

    /// The chain ID of the transaction. If set to `None`, no checks are performed.
    ///
    /// Incorporated as part of the Spurious Dragon upgrade via [EIP-155].
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub chain_id: Option<U64>,

    /// The signature of the transaction. If set to `None`, no checks are performed.
    #[cfg_attr(feature = "with-serde", serde(flatten, skip_serializing_if = "Option::is_none"))]
    pub signature: Option<Signature>,
}

#[cfg(feature = "with-rlp")]
impl LegacyTransaction {
    fn rlp_encode_internal(
        &self,
        stream: &mut rlp::RlpStream,
        chain_id: Option<U64>,
        sig: Option<Signature>,
    ) {
        let mut num_fields = 6;
        if chain_id.is_some() | sig.is_some() {
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

        match (chain_id, sig.as_ref()) {
            (Some(chain_id), Some(sig)) => {
                let v = sig.v.as_eip155(chain_id.as_u64());
                stream.append(&v).append(&sig.r).append(&sig.s);
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

    pub fn rlp_unsigned(&self, stream: &mut rlp::RlpStream) {
        self.rlp_encode_internal(stream, self.chain_id, None);
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Encodable for LegacyTransaction {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        self.rlp_encode_internal(s, self.chain_id, self.signature);
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Decodable for LegacyTransaction {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let items = rlp.item_count()?;
        if items != 6 && items != 9 {
            return Err(rlp::DecoderError::RlpIncorrectListLen);
        }

        let mut tx = Self {
            nonce: rlp.val_at(0usize)?,
            gas_price: rlp.val_at(1usize)?,
            gas_limit: rlp.val_at(2usize)?,
            to: rlp.opt_at(3usize)?,
            value: rlp.val_at(4usize)?,
            data: rlp.val_at(5usize)?,
            chain_id: None,
            signature: None,
        };

        // If there are 9 items, then the last 3 are the chain ID and signature
        if items == 9 {
            let v = rlp.at(6usize)?;
            let r = rlp.at(7usize)?;
            let s = rlp.at(8usize)?;

            if r.is_empty() && s.is_empty() {
                tx.chain_id = Some(<U64 as rlp::Decodable>::decode(&v)?);
            } else {
                let signature = Signature {
                    v: <RecoveryId as rlp::Decodable>::decode(&v)?,
                    r: <H256 as rlp::Decodable>::decode(&r)?,
                    s: <H256 as rlp::Decodable>::decode(&s)?,
                };
                tx.chain_id = signature.v.chain_id().map(U64::from);
                tx.signature = Some(signature);
            }
        }

        Ok(tx)
    }
}

#[cfg(all(test, any(feature = "with-serde", feature = "with-rlp")))]
mod tests {
    use super::{super::signature::RecoveryId, Bytes, LegacyTransaction, Signature};

    #[cfg(feature = "with-serde")]
    #[test]
    fn serde_encode_works() {
        let tx = LegacyTransaction {
            nonce: 235.into(),
            gas_price: 25_670_917_490u128.into(),
            gas_limit: 114_756.into(),
            to: Some(hex_literal::hex!("8e5660b4ab70168b5a6feea0e0315cb49c8cd539").into()),
            value: 0.into(),
            data: hex_literal::hex!("6f652e1a000000000000000000000000959e104e1a4db6317fa58f8295f586e1a978c29700000000000000000000000000000000000000000000000000000000000008b800000000000000000000000000000000000000000000054b40b1f852bda000000000000000000000000000000000000000000000000000000000017723be2580").into(),
            chain_id: Some(1.into()),
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
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_codec_works() {
        use rlp::{Decodable, Encodable};

        let tx = LegacyTransaction {
            nonce: 235.into(),
            gas_price: 25_670_917_490u128.into(),
            gas_limit: 114_756.into(),
            to: Some(hex_literal::hex!("8e5660b4ab70168b5a6feea0e0315cb49c8cd539").into()),
            value: 0.into(),
            data: hex_literal::hex!("6f652e1a000000000000000000000000959e104e1a4db6317fa58f8295f586e1a978c29700000000000000000000000000000000000000000000000000000000000008b800000000000000000000000000000000000000000000054b40b1f852bda000000000000000000000000000000000000000000000000000000000017723be2580").into(),
            chain_id: Some(1.into()),
            signature: Some(Signature {
                v: RecoveryId::try_from(0x26).unwrap(),
                r: hex_literal::hex!("a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6").into(),
                s: hex_literal::hex!("4b88362cca36cc9742366ca474fd777328cb6ee012ceee2da9aa147761e17cbf").into(),
            }),
        };

        // RLP encode works
        let expected = hex_literal::hex!("f8eb81eb8505fa1b1d728301c044948e5660b4ab70168b5a6feea0e0315cb49c8cd53980b8846f652e1a000000000000000000000000959e104e1a4db6317fa58f8295f586e1a978c29700000000000000000000000000000000000000000000000000000000000008b800000000000000000000000000000000000000000000054b40b1f852bda000000000000000000000000000000000000000000000000000000000017723be258026a0a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6a04b88362cca36cc9742366ca474fd777328cb6ee012ceee2da9aa147761e17cbf");
        let actual = Bytes::from(Encodable::rlp_bytes(&tx).freeze());
        assert_eq!(Bytes::from(expected), actual);

        // RLP decode works
        let rlp = rlp::Rlp::new(expected.as_ref());
        let decoded = <LegacyTransaction as Decodable>::decode(&rlp).unwrap();
        assert_eq!(tx, decoded);
    }
}
