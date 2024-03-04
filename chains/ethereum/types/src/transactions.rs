pub mod access_list;
pub mod eip1559;
pub mod eip2930;
pub mod legacy;
pub mod signature;
pub mod signed_transaction;
pub mod typed_transaction;

use core::default::Default;

use crate::{
    bytes::Bytes,
    eth_hash::{Address, H256},
    eth_uint::U256,
};
pub use access_list::AccessList;
pub use eip1559::Eip1559Transaction;
pub use eip2930::Eip2930Transaction;
pub use legacy::LegacyTransaction;
pub use signature::Signature;
pub use signed_transaction::SignedTransaction;
pub use typed_transaction::TypedTransaction;
pub type Transaction = SignedTransaction<TypedTransaction>;

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum GasPrice {
    Legacy(U256),
    Eip1559 { max_priority_fee_per_gas: U256, max_fee_per_gas: U256 },
}

impl Default for GasPrice {
    fn default() -> Self {
        Self::Legacy(U256::zero())
    }
}

pub trait TransactionT {
    type ExtraFields: Send + Sync + Clone + PartialEq + Eq;

    // Encode the transaction
    fn encode(&self, signature: Option<&Signature>) -> Bytes;

    /// The hash of the transaction without signature
    fn sighash(&self) -> H256;

    // Compute the tx-hash using the provided signature
    fn compute_tx_hash(&self, signature: &Signature) -> H256;

    // chain id, is only None for Legacy Transactions
    fn chain_id(&self) -> Option<u64>;
    fn nonce(&self) -> u64;
    fn gas_price(&self) -> GasPrice;
    fn gas_limit(&self) -> u64;
    fn to(&self) -> Option<Address>;
    fn value(&self) -> U256;
    fn data(&self) -> &[u8];

    /// EIP-2930 access list
    fn access_list(&self) -> Option<&AccessList>;
    /// EIP-2718 transaction type
    fn transaction_type(&self) -> Option<u8>;
    fn extra_fields(&self) -> Option<Self::ExtraFields>;
}

pub trait SignedTransactionT: TransactionT {
    fn tx_hash(&self) -> H256;
    fn signature(&self) -> Signature;
    fn encode_signed(&self) -> Bytes;
}
