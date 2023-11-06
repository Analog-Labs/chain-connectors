use super::{eip1559::Eip1559Transaction, eip2930::Eip2930Transaction, legacy::LegacyTransaction};

#[cfg(all(feature = "with-rlp", feature = "with-crypto"))]
use crate::{
    eth_hash::{Address, H256},
    eth_uint::U256,
    transactions::{access_list::AccessList, signature::Signature, GasPrice, TransactionT},
};

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

#[cfg(all(feature = "with-rlp", feature = "with-crypto"))]
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

    fn gas_limit(&self) -> U256 {
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
}
