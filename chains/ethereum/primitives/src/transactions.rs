pub mod access_list;
pub mod eip1559;
pub mod eip2930;
pub mod legacy;
pub mod signature;
pub mod signed_transaction;
pub mod typed_transaction;

use core::default::Default;

use crate::{
    eth_hash::{Address, H256},
    eth_uint::U256,
};
use access_list::AccessList;
use signature::Signature;

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "with-serde",
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

    // Compute the tx-hash using the provided signature
    fn compute_tx_hash(&self, signature: &Signature) -> H256;

    fn chain_id(&self) -> Option<u64>;
    fn nonce(&self) -> u64;
    fn gas_price(&self) -> GasPrice;
    fn gas_limit(&self) -> U256;
    fn to(&self) -> Option<Address>;
    fn value(&self) -> U256;
    fn data(&self) -> &[u8];
    /// The hash of the transaction without signature
    fn sighash(&self) -> H256;
    /// EIP-2930 access list
    fn access_list(&self) -> Option<&AccessList>;
    /// EIP-2718 transaction type
    fn transaction_type(&self) -> Option<u8>;
    fn extra_fields(&self) -> Option<Self::ExtraFields>;
}

pub trait SignedTransactionT: TransactionT {
    fn tx_hash(&self) -> H256;
    fn signature(&self) -> Signature;
}