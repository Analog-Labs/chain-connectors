use super::{eip1559::Eip1559Transaction, eip2930::Eip2930Transaction, legacy::LegacyTransaction};

#[cfg(feature = "with-rlp")]
use crate::{
    rlp_utils::{RlpDecodableTransaction, RlpEncodableTransaction},
    transactions::signature::Signature,
};

#[cfg(feature = "with-crypto")]
use crate::{
    bytes::Bytes,
    eth_hash::{Address, H256},
    eth_uint::U256,
    transactions::{access_list::AccessList, GasPrice, TransactionT},
};

/// The [`TypedTransaction`] enum represents all Ethereum transaction types.
///
/// Its variants correspond to specific allowed transactions:
/// 1. Legacy (pre-EIP2718) [`LegacyTransaction`]
/// 2. EIP2930 (state access lists) [`Eip2930Transaction`]
/// 3. EIP1559 [`Eip1559Transaction`]
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
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
    #[cfg_attr(feature = "with-serde", serde(rename = "0x0"))]
    Legacy(LegacyTransaction),
    #[cfg_attr(feature = "with-serde", serde(rename = "0x1"))]
    Eip2930(Eip2930Transaction),
    #[cfg_attr(feature = "with-serde", serde(rename = "0x2"))]
    Eip1559(Eip1559Transaction),
}

#[cfg(feature = "with-rlp")]
impl RlpEncodableTransaction for TypedTransaction {
    fn rlp_append(&self, s: &mut rlp::RlpStream, signature: Option<&Signature>) {
        match self {
            Self::Legacy(tx) => RlpEncodableTransaction::rlp_append(tx, s, signature),
            Self::Eip2930(tx) => RlpEncodableTransaction::rlp_append(tx, s, signature),
            Self::Eip1559(tx) => RlpEncodableTransaction::rlp_append(tx, s, signature),
        };
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Encodable for TypedTransaction {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        <Self as RlpEncodableTransaction>::rlp_append(self, s, None);
    }
}

#[cfg(feature = "with-rlp")]
impl RlpDecodableTransaction for TypedTransaction {
    fn rlp_decode(
        rlp: &rlp::Rlp,
        decode_signature: bool,
    ) -> Result<(Self, Option<Signature>), rlp::DecoderError> {
        // The first byte of the RLP-encoded transaction is the transaction type.
        // [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
        let first = *rlp.as_raw().first().ok_or(rlp::DecoderError::RlpIsTooShort)?;
        match first {
            0x01 => {
                <Eip2930Transaction as RlpDecodableTransaction>::rlp_decode(rlp, decode_signature)
                    .map(|(tx, sig)| (Self::Eip2930(tx), sig))
            },
            0x02 => {
                <Eip1559Transaction as RlpDecodableTransaction>::rlp_decode(rlp, decode_signature)
                    .map(|(tx, sig)| (Self::Eip1559(tx), sig))
            },
            // legacy transaction types always start with a byte >= 0xc0.
            v if v >= 0xc0 => {
                <LegacyTransaction as RlpDecodableTransaction>::rlp_decode(rlp, decode_signature)
                    .map(|(tx, sig)| (Self::Legacy(tx), sig))
            },
            _ => Err(rlp::DecoderError::Custom("unknown transaction type")),
        }
    }

    fn rlp_decode_unsigned(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        // The first byte of the RLP-encoded transaction is the transaction type.
        // [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
        let first = *rlp.as_raw().first().ok_or(rlp::DecoderError::RlpIsTooShort)?;
        match first {
            0x01 => <Eip2930Transaction as RlpDecodableTransaction>::rlp_decode_unsigned(rlp)
                .map(Self::Eip2930),
            0x02 => <Eip1559Transaction as RlpDecodableTransaction>::rlp_decode_unsigned(rlp)
                .map(Self::Eip1559),
            // legacy transaction types always start with a byte >= 0xc0.
            v if v >= 0xc0 => {
                <LegacyTransaction as RlpDecodableTransaction>::rlp_decode_unsigned(rlp)
                    .map(Self::Legacy)
            },
            _ => Err(rlp::DecoderError::Custom("unknown transaction type")),
        }
    }

    fn rlp_decode_signed(rlp: &rlp::Rlp) -> Result<(Self, Option<Signature>), rlp::DecoderError> {
        // The first byte of the RLP-encoded transaction is the transaction type.
        // [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
        let first = *rlp.as_raw().first().ok_or(rlp::DecoderError::RlpIsTooShort)?;
        match first {
            0x01 => <Eip2930Transaction as RlpDecodableTransaction>::rlp_decode_signed(rlp)
                .map(|(tx, sig)| (Self::Eip2930(tx), sig)),
            0x02 => <Eip1559Transaction as RlpDecodableTransaction>::rlp_decode_signed(rlp)
                .map(|(tx, sig)| (Self::Eip1559(tx), sig)),
            // legacy transaction types always start with a byte >= 0xc0.
            v if v >= 0xc0 => {
                <LegacyTransaction as RlpDecodableTransaction>::rlp_decode_signed(rlp)
                    .map(|(tx, sig)| (Self::Legacy(tx), sig))
            },
            _ => Err(rlp::DecoderError::Custom("unknown transaction type")),
        }
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Decodable for TypedTransaction {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        <Self as RlpDecodableTransaction>::rlp_decode_unsigned(rlp)
    }
}

impl From<LegacyTransaction> for TypedTransaction {
    fn from(tx: LegacyTransaction) -> Self {
        Self::Legacy(tx)
    }
}

impl From<Eip2930Transaction> for TypedTransaction {
    fn from(tx: Eip2930Transaction) -> Self {
        Self::Eip2930(tx)
    }
}

impl From<Eip1559Transaction> for TypedTransaction {
    fn from(tx: Eip1559Transaction) -> Self {
        Self::Eip1559(tx)
    }
}

#[cfg(feature = "with-crypto")]
impl TransactionT for TypedTransaction {
    type ExtraFields = ();

    fn compute_tx_hash(&self, signature: &Signature) -> H256 {
        match self {
            Self::Legacy(tx) => TransactionT::compute_tx_hash(tx, signature),
            Self::Eip2930(tx) => TransactionT::compute_tx_hash(tx, signature),
            Self::Eip1559(tx) => TransactionT::compute_tx_hash(tx, signature),
        }
    }

    fn chain_id(&self) -> Option<u64> {
        match self {
            Self::Legacy(tx) => TransactionT::chain_id(tx),
            Self::Eip2930(tx) => TransactionT::chain_id(tx),
            Self::Eip1559(tx) => TransactionT::chain_id(tx),
        }
    }

    fn nonce(&self) -> u64 {
        match self {
            Self::Legacy(tx) => TransactionT::nonce(tx),
            Self::Eip2930(tx) => TransactionT::nonce(tx),
            Self::Eip1559(tx) => TransactionT::nonce(tx),
        }
    }

    fn gas_price(&self) -> GasPrice {
        match self {
            Self::Legacy(tx) => TransactionT::gas_price(tx),
            Self::Eip2930(tx) => TransactionT::gas_price(tx),
            Self::Eip1559(tx) => TransactionT::gas_price(tx),
        }
    }

    fn gas_limit(&self) -> u64 {
        match self {
            Self::Legacy(tx) => TransactionT::gas_limit(tx),
            Self::Eip2930(tx) => TransactionT::gas_limit(tx),
            Self::Eip1559(tx) => TransactionT::gas_limit(tx),
        }
    }

    fn to(&self) -> Option<Address> {
        match self {
            Self::Legacy(tx) => TransactionT::to(tx),
            Self::Eip2930(tx) => TransactionT::to(tx),
            Self::Eip1559(tx) => TransactionT::to(tx),
        }
    }

    fn value(&self) -> U256 {
        match self {
            Self::Legacy(tx) => TransactionT::value(tx),
            Self::Eip2930(tx) => TransactionT::value(tx),
            Self::Eip1559(tx) => TransactionT::value(tx),
        }
    }

    fn data(&self) -> &[u8] {
        match self {
            Self::Legacy(tx) => TransactionT::data(tx),
            Self::Eip2930(tx) => TransactionT::data(tx),
            Self::Eip1559(tx) => TransactionT::data(tx),
        }
    }

    fn sighash(&self) -> H256 {
        match self {
            Self::Legacy(tx) => TransactionT::sighash(tx),
            Self::Eip2930(tx) => TransactionT::sighash(tx),
            Self::Eip1559(tx) => TransactionT::sighash(tx),
        }
    }

    fn access_list(&self) -> Option<&AccessList> {
        match self {
            Self::Legacy(tx) => TransactionT::access_list(tx),
            Self::Eip2930(tx) => TransactionT::access_list(tx),
            Self::Eip1559(tx) => TransactionT::access_list(tx),
        }
    }

    fn transaction_type(&self) -> Option<u8> {
        match self {
            Self::Legacy(tx) => TransactionT::transaction_type(tx),
            Self::Eip2930(tx) => TransactionT::transaction_type(tx),
            Self::Eip1559(tx) => TransactionT::transaction_type(tx),
        }
    }

    fn extra_fields(&self) -> Option<Self::ExtraFields> {
        None
    }

    fn encode(&self, signature: Option<&Signature>) -> Bytes {
        match self {
            Self::Legacy(tx) => TransactionT::encode(tx, signature),
            Self::Eip2930(tx) => TransactionT::encode(tx, signature),
            Self::Eip1559(tx) => TransactionT::encode(tx, signature),
        }
    }
}
