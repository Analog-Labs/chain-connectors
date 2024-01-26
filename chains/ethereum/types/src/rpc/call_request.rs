#![allow(clippy::missing_errors_doc)]
#[cfg(feature = "with-serde")]
use crate::serde_utils::uint_to_hex;
use crate::{
    bytes::Bytes,
    eth_hash::Address,
    eth_uint::U256,
    transactions::{
        access_list::AccessList, eip1559::Eip1559Transaction, eip2930::Eip2930Transaction,
        legacy::LegacyTransaction, typed_transaction::TypedTransaction,
    },
};

/// Call request for `eth_call` and adjacent methods.
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
pub struct CallRequest {
    /// Sender address
    #[cfg_attr(feature = "with-serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub from: Option<Address>,

    /// Recipient address (None for contract creation)
    #[cfg_attr(feature = "with-serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub to: Option<Address>,

    /// Supplied gas (None for sensible default)
    #[cfg_attr(
        feature = "with-serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "gas",
            with = "uint_to_hex",
        )
    )]
    pub gas_limit: Option<u64>,

    /// Gas price (None for sensible default)
    #[cfg_attr(feature = "with-serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub gas_price: Option<U256>,

    /// Transferred value (None for no transfer)
    #[cfg_attr(feature = "with-serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub value: Option<U256>,

    /// The data of the transaction.
    #[cfg_attr(feature = "with-serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub data: Option<Bytes>,

    /// The nonce of the transaction. If set to `None`, no checks are performed.
    #[cfg_attr(
        feature = "with-serde",
        serde(default, skip_serializing_if = "Option::is_none", with = "uint_to_hex",)
    )]
    pub nonce: Option<u64>,

    /// The chain ID of the transaction. If set to `None`, no checks are performed.
    ///
    /// Incorporated as part of the Spurious Dragon upgrade via [EIP-155].
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    #[cfg_attr(
        feature = "with-serde",
        serde(default, skip_serializing_if = "Option::is_none", with = "uint_to_hex",)
    )]
    pub chain_id: Option<u64>,

    /// The priority fee per gas.
    ///
    /// Incorporated as part of the London upgrade via [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    #[cfg_attr(feature = "with-serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub max_priority_fee_per_gas: Option<U256>,

    /// A list of addresses and storage keys that the transaction plans to access.
    ///
    /// Added in [EIP-2930].
    ///
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    #[cfg_attr(
        feature = "with-serde",
        serde(
            default,
            skip_serializing_if = "AccessList::is_empty",
            deserialize_with = "deserialize_null_default"
        )
    )]
    pub access_list: AccessList,

    /// The max fee per gas.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    #[cfg_attr(feature = "with-serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub max_fee_per_gas: Option<U256>,

    /// EIP-2718 type
    #[cfg_attr(
        feature = "with-serde",
        serde(
            default,
            rename = "type",
            skip_serializing_if = "Option::is_none",
            with = "uint_to_hex",
        )
    )]
    pub transaction_type: Option<u64>,
}

#[cfg(feature = "with-serde")]
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    let opt = <Option<T> as serde::Deserialize<'de>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

impl From<LegacyTransaction> for CallRequest {
    fn from(tx: LegacyTransaction) -> Self {
        Self {
            from: None,
            to: tx.to,
            gas_limit: Some(tx.gas_limit),
            gas_price: Some(tx.gas_price),
            value: Some(tx.value),
            data: Some(tx.data.clone()),
            nonce: Some(tx.nonce),
            chain_id: tx.chain_id,
            max_priority_fee_per_gas: None,
            access_list: AccessList::default(),
            max_fee_per_gas: None,
            transaction_type: Some(0x00),
        }
    }
}

impl From<Eip2930Transaction> for CallRequest {
    fn from(tx: Eip2930Transaction) -> Self {
        Self {
            from: None,
            to: tx.to,
            gas_limit: Some(tx.gas_limit),
            gas_price: Some(tx.gas_price),
            value: Some(tx.value),
            data: Some(tx.data.clone()),
            nonce: Some(tx.nonce),
            chain_id: Some(tx.chain_id),
            max_priority_fee_per_gas: None,
            access_list: tx.access_list,
            max_fee_per_gas: None,
            transaction_type: Some(0x01),
        }
    }
}

impl From<Eip1559Transaction> for CallRequest {
    fn from(tx: Eip1559Transaction) -> Self {
        Self {
            from: None,
            to: tx.to,
            gas_limit: Some(tx.gas_limit),
            gas_price: None,
            max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
            max_fee_per_gas: Some(tx.max_fee_per_gas),
            value: Some(tx.value),
            data: Some(tx.data.clone()),
            nonce: Some(tx.nonce),
            chain_id: Some(tx.chain_id),
            access_list: tx.access_list,
            transaction_type: Some(0x02),
        }
    }
}

impl From<TypedTransaction> for CallRequest {
    fn from(tx: TypedTransaction) -> Self {
        match tx {
            TypedTransaction::Legacy(tx) => tx.into(),
            TypedTransaction::Eip2930(tx) => tx.into(),
            TypedTransaction::Eip1559(tx) => tx.into(),
        }
    }
}
