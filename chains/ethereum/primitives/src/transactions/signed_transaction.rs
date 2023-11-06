use rlp::Rlp;

use super::{
    access_list::AccessList, signature::Signature, GasPrice, SignedTransactionT, TransactionT,
};
#[cfg(feature = "with-rlp")]
use crate::rlp_utils::{RlpDecodableTransaction, RlpEncodableTransaction};
use crate::{
    eth_hash::{Address, H256},
    eth_uint::U256,
};

pub struct SignedTransaction<T: TransactionT> {
    tx_hash: H256,
    payload: T,
    signature: Signature,
}

impl<T> SignedTransaction<T>
where
    T: TransactionT,
{
    pub fn new(payload: T, signature: Signature) -> Self {
        let tx_hash = payload.compute_tx_hash(&signature);
        Self { tx_hash, payload, signature }
    }
}

#[cfg(feature = "with-rlp")]
impl<T> RlpEncodableTransaction for SignedTransaction<T>
where
    T: RlpEncodableTransaction + TransactionT,
{
    fn rlp_append(&self, s: &mut rlp::RlpStream, signature: Option<&Signature>) {
        <T as RlpEncodableTransaction>::rlp_append(&self.payload, s, signature);
    }
}

#[cfg(feature = "with-rlp")]
impl<T> rlp::Encodable for SignedTransaction<T>
where
    T: RlpEncodableTransaction + TransactionT,
{
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        <T as RlpEncodableTransaction>::rlp_append(&self.payload, s, Some(&self.signature));
    }
}

#[cfg(feature = "with-rlp")]
impl<T> RlpDecodableTransaction for SignedTransaction<T>
where
    T: RlpDecodableTransaction + TransactionT,
{
    // For SignedTransaction we always decode the signature
    fn rlp_decode_unsigned(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        let (payload, signature) = <T as RlpDecodableTransaction>::rlp_decode(rlp, true)?;
        let signature = signature.ok_or(rlp::DecoderError::Custom("tx signature is missing"))?;
        let tx_hash = payload.compute_tx_hash(&signature);
        Ok(Self { tx_hash, payload, signature })
    }

    fn rlp_decode(
        rlp: &Rlp,
        _decode_signature: bool,
    ) -> Result<(Self, Option<Signature>), rlp::DecoderError> {
        let signed_tx = <Self as RlpDecodableTransaction>::rlp_decode_unsigned(rlp)?;
        let signature = signed_tx.signature;
        Ok((signed_tx, Some(signature)))
    }
}

#[cfg(feature = "with-rlp")]
impl<T> rlp::Decodable for SignedTransaction<T>
where
    T: RlpDecodableTransaction + TransactionT,
{
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        let (payload, signature) = <T as RlpDecodableTransaction>::rlp_decode(rlp, true)?;
        let signature = signature.ok_or(rlp::DecoderError::Custom("tx signature is missing"))?;
        let tx_hash = payload.compute_tx_hash(&signature);
        Ok(Self { tx_hash, payload, signature })
    }
}

impl<T> TransactionT for SignedTransaction<T>
where
    T: TransactionT,
{
    type ExtraFields = <T as TransactionT>::ExtraFields;

    // Compute the tx-hash using the provided signature
    fn compute_tx_hash(&self, signature: &Signature) -> H256 {
        self.payload.compute_tx_hash(signature)
    }
    fn chain_id(&self) -> Option<u64> {
        self.payload.chain_id()
    }
    fn nonce(&self) -> u64 {
        self.payload.nonce()
    }
    fn gas_price(&self) -> GasPrice {
        self.payload.gas_price()
    }
    fn gas_limit(&self) -> U256 {
        self.payload.gas_limit()
    }
    fn to(&self) -> Option<Address> {
        self.payload.to()
    }
    fn value(&self) -> U256 {
        self.payload.value()
    }
    fn data(&self) -> &[u8] {
        self.payload.data()
    }
    fn sighash(&self) -> H256 {
        self.payload.sighash()
    }
    fn access_list(&self) -> Option<&AccessList> {
        self.payload.access_list()
    }
    fn transaction_type(&self) -> Option<u8> {
        self.payload.transaction_type()
    }
    fn extra_fields(&self) -> Option<Self::ExtraFields> {
        self.payload.extra_fields()
    }
}

impl<T> SignedTransactionT for SignedTransaction<T>
where
    T: TransactionT,
{
    fn tx_hash(&self) -> H256 {
        self.tx_hash
    }

    fn signature(&self) -> Signature {
        self.signature
    }
}
