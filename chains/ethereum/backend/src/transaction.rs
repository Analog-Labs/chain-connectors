use alloc::vec::Vec;
use rosetta_ethereum_primitives::{Address, Bytes, H256, U256, U64};

/// Parameters for sending a transaction
#[derive(Clone, Default, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "with-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct TransactionCall {
    /// Sender address
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub from: Option<Address>,

    /// Recipient address (None for contract creation)
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub to: Option<Address>,

    /// Supplied gas (None for sensible default)
    #[cfg_attr(
        feature = "with-serde",
        serde(skip_serializing_if = "Option::is_none", rename = "gas")
    )]
    pub gas_limit: Option<U64>,

    /// Gas price (None for sensible default)
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub gas_price: Option<U256>,

    /// Transferred value (None for no transfer)
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub value: Option<U256>,

    /// The data of the transaction.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub data: Option<Bytes>,

    /// The nonce of the transaction. If set to `None`, no checks are performed.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nonce: Option<U256>,

    /// The chain ID of the transaction. If set to `None`, no checks are performed.
    ///
    /// Incorporated as part of the Spurious Dragon upgrade via [EIP-155].
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub chain_id: Option<U256>,

    /// The priority fee per gas.
    ///
    /// Incorporated as part of the London upgrade via [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub max_priority_fee_per_gas: Option<U256>,

    /// A list of addresses and storage keys that the transaction plans to access.
    ///
    /// Added in [EIP-2930].
    ///
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    #[cfg_attr(
        feature = "with-serde",
        serde(
            skip_serializing_if = "Vec::is_empty",
            deserialize_with = "deserialize_null_default"
        )
    )]
    pub access_list: Vec<(Address, Vec<H256>)>,

    /// The max fee per gas.
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub max_fee_per_gas: Option<U256>,
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
