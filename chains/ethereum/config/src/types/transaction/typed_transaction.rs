use ethereum_types::{H160, U256};

use super::{
    eip1559::Eip1559Transaction, eip2930::Eip2930Transaction, legacy::LegacyTransaction, AccessList,
};

/// The [`TypedTransaction`] enum represents all Ethereum transaction types.
///
/// Its variants correspond to specific allowed transactions:
/// 1. Legacy (pre-EIP2718) [`LegacyTransaction`]
/// 2. EIP2930 (state access lists) [`Eip2930Transaction`]
/// 3. EIP1559 [`Eip1559Transaction`]
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(tag = "type"))]
pub enum TypedTransaction {
    #[cfg_attr(feature = "serde", serde(rename = "0x0"))]
    Legacy(LegacyTransaction),
    #[cfg_attr(feature = "serde", serde(rename = "0x1"))]
    Eip2930(Eip2930Transaction),
    #[cfg_attr(feature = "serde", serde(rename = "0x2"))]
    Eip1559(Eip1559Transaction),
}

impl TypedTransaction {
    #[must_use]
    pub fn data(&self) -> &[u8] {
        match self {
            Self::Legacy(tx) => tx.data.as_ref(),
            Self::Eip2930(tx) => tx.data.as_ref(),
            Self::Eip1559(tx) => tx.data.as_ref(),
        }
    }

    #[must_use]
    pub const fn to(&self) -> Option<H160> {
        match self {
            Self::Legacy(tx) => tx.to,
            Self::Eip2930(tx) => tx.to,
            Self::Eip1559(tx) => tx.to,
        }
    }

    #[must_use]
    pub const fn nonce(&self) -> u64 {
        match self {
            Self::Legacy(tx) => tx.nonce,
            Self::Eip2930(tx) => tx.nonce,
            Self::Eip1559(tx) => tx.nonce,
        }
    }

    #[must_use]
    pub const fn gas_limit(&self) -> u64 {
        match self {
            Self::Legacy(tx) => tx.gas_limit,
            Self::Eip2930(tx) => tx.gas_limit,
            Self::Eip1559(tx) => tx.gas_limit,
        }
    }

    #[must_use]
    pub const fn value(&self) -> U256 {
        match self {
            Self::Legacy(tx) => tx.value,
            Self::Eip2930(tx) => tx.value,
            Self::Eip1559(tx) => tx.value,
        }
    }

    #[must_use]
    pub const fn chain_id(&self) -> Option<u64> {
        match self {
            Self::Legacy(tx) => tx.chain_id,
            Self::Eip2930(tx) => Some(tx.chain_id),
            Self::Eip1559(tx) => Some(tx.chain_id),
        }
    }

    #[must_use]
    pub const fn access_list(&self) -> Option<&AccessList> {
        match self {
            Self::Legacy(_) => None,
            Self::Eip2930(tx) => Some(&tx.access_list),
            Self::Eip1559(tx) => Some(&tx.access_list),
        }
    }

    #[must_use]
    pub const fn tx_type(&self) -> u8 {
        match self {
            Self::Legacy(_) => 0x0,
            Self::Eip2930(_) => 0x1,
            Self::Eip1559(_) => 0x2,
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

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::TypedTransaction;
    use crate::transaction::{
        eip1559::tests::build_eip1559, eip2930::tests::build_eip2930, legacy::tests::build_legacy,
    };

    #[allow(clippy::unwrap_used)]
    fn build_typed_transaction<T: Into<TypedTransaction>>(
        builder: fn() -> (T, serde_json::Value),
    ) -> (TypedTransaction, serde_json::Value) {
        let (tx, mut expected) = builder();
        let tx: TypedTransaction = tx.into();
        let tx_type = match &tx {
            TypedTransaction::Legacy(_) => "0x0",
            TypedTransaction::Eip2930(_) => "0x1",
            TypedTransaction::Eip1559(_) => "0x2",
        };
        // Add the type field to the json
        let old_value = expected
            .as_object_mut()
            .unwrap()
            .insert("type".to_string(), serde_json::json!(tx_type));

        // Guarantee that the type field was not already present
        assert_eq!(old_value, None);
        (tx, expected)
    }

    #[test]
    fn can_encode_eip1559() {
        let (tx, expected) = build_typed_transaction(build_eip1559);
        let actual = serde_json::to_value(&tx).unwrap();
        assert_eq!(expected, actual);

        // can decode json
        let json = serde_json::to_value(&tx).unwrap();
        let decoded = serde_json::from_value::<TypedTransaction>(json).unwrap();
        assert_eq!(tx, decoded);
    }

    #[test]
    fn can_encode_eip2930() {
        let (tx, expected) = build_typed_transaction(build_eip2930);
        let actual = serde_json::to_value(&tx).unwrap();
        assert_eq!(expected, actual);

        // can decode json
        let json_str = serde_json::to_string(&tx).unwrap();
        let decoded = serde_json::from_str::<TypedTransaction>(&json_str).unwrap();
        assert_eq!(tx, decoded);
    }

    #[test]
    fn can_encode_legacy() {
        let (tx, expected) = build_typed_transaction(|| build_legacy(false));
        let actual = serde_json::to_value(&tx).unwrap();
        assert_eq!(expected, actual);

        // can decode json
        let json_str = serde_json::to_string(&tx).unwrap();
        let decoded = serde_json::from_str::<TypedTransaction>(&json_str).unwrap();
        assert_eq!(tx, decoded);
    }

    #[test]
    fn can_encode_legacy_eip155() {
        let (tx, expected) = build_typed_transaction(|| build_legacy(true));
        let actual = serde_json::to_value(&tx).unwrap();
        assert_eq!(expected, actual);

        // can decode json
        let json_str = serde_json::to_string(&tx).unwrap();
        let decoded = serde_json::from_str::<TypedTransaction>(&json_str).unwrap();
        assert_eq!(tx, decoded);
    }
}
