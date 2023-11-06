#![allow(clippy::missing_errors_doc)]
use crate::{
    eth_hash::{Address, H256},
    eth_uint::U256,
};

#[derive(Clone, Default, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "with-rlp",
    derive(rlp_derive::RlpEncodableWrapper, rlp_derive::RlpDecodableWrapper)
)]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct AccessList(pub Vec<AccessListItem>);

impl AccessList {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-rlp", derive(rlp_derive::RlpEncodable, rlp_derive::RlpDecodable))]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct AccessListWithGasUsed {
    pub access_list: AccessList,
    pub gas_used: U256,
}

impl From<Vec<AccessListItem>> for AccessList {
    fn from(src: Vec<AccessListItem>) -> Self {
        Self(src)
    }
}

/// Access list item
#[derive(Clone, Default, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-rlp", derive(rlp_derive::RlpEncodable, rlp_derive::RlpDecodable))]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct AccessListItem {
    /// Accessed address
    pub address: Address,
    /// Accessed storage keys
    pub storage_keys: Vec<H256>,
}

#[cfg(all(test, any(feature = "with-serde", feature = "with-rlp")))]
mod tests {
    use super::{AccessList, AccessListItem, Address, H256};

    #[cfg(feature = "with-serde")]
    #[test]
    fn serde_encode_works() {
        let access_list = AccessList(vec![AccessListItem {
            address: Address::from(hex_literal::hex!("8e5660b4ab70168b5a6feea0e0315cb49c8cd539")),
            storage_keys: vec![
                H256::zero(),
                H256::from(hex_literal::hex!(
                    "a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6"
                )),
                H256::from(hex_literal::hex!(
                    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )),
            ],
        }]);

        // can encode as json
        let actual = serde_json::to_value(access_list.clone()).unwrap();
        let expected = serde_json::json!([
            {
                "address": "0x8e5660b4ab70168b5a6feea0e0315cb49c8cd539",
                "storageKeys": [
                    "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "0xa19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6",
                    "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                ],
            },
        ]);
        assert_eq!(expected, actual);

        // can decode json
        let json_str = serde_json::to_string(&access_list).unwrap();
        let decoded = serde_json::from_str::<AccessList>(&json_str).unwrap();
        assert_eq!(access_list, decoded);
    }
}
