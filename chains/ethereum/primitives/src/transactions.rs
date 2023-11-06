pub mod access_list;
pub mod eip1559;
pub mod eip2930;
pub mod legacy;
pub mod signature;

use eip1559::Eip1559Transaction;
use eip2930::Eip2930Transaction;
use legacy::LegacyTransaction;

/// The [`TypedTransaction`] enum represents all Ethereum transaction types.
///
/// Its variants correspond to specific allowed transactions:
/// 1. Legacy (pre-EIP2718) [`LegacyTransaction`]
/// 2. EIP2930 (state access lists) [`Eip2930Transaction`]
/// 3. EIP1559 [`Eip1559Transaction`]
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
pub enum TypedTransaction {
    #[cfg_attr(feature = "with-serde", serde(rename = "0x00"))]
    Legacy(LegacyTransaction),
    #[cfg_attr(feature = "with-serde", serde(rename = "0x01"))]
    Eip2930(Eip2930Transaction),
    #[cfg_attr(feature = "with-serde", serde(rename = "0x02"))]
    Eip1559(Eip1559Transaction),
}

#[cfg(feature = "with-rlp")]
impl rlp::Encodable for TypedTransaction {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        match self {
            Self::Legacy(tx) => tx.rlp_append(s),
            Self::Eip2930(tx) => tx.rlp_append(s),
            Self::Eip1559(tx) => tx.rlp_append(s),
        }
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Decodable for TypedTransaction {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        // The first byte of the RLP-encoded transaction is the transaction type.
        // [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
        let first = *rlp.data()?.first().ok_or(rlp::DecoderError::RlpIsTooShort)?;
        match first {
            0x01 => Ok(Self::Eip2930(Eip2930Transaction::decode(rlp)?)),
            0x02 => Ok(Self::Eip1559(Eip1559Transaction::decode(rlp)?)),
            // legacy transaction types always start with a byte >= 0xc0.
            v if v >= 0xc0 => Ok(Self::Legacy(LegacyTransaction::decode(rlp)?)),
            _ => Err(rlp::DecoderError::Custom("unknown transaction type")),
        }
    }
}
