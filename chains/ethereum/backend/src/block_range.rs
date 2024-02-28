use rosetta_ethereum_types::{Address, AtBlock, H256};

#[cfg(feature = "serde")]
use crate::serde_util::opt_value_or_array;

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
pub struct BlockRange {
    /// A list of addresses from which logs should originate.
    #[cfg_attr(
        feature = "serde",
        serde(with = "opt_value_or_array", skip_serializing_if = "Vec::is_empty")
    )]
    pub address: Vec<Address>,
    /// Array of topics. topics are order-dependent.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub topics: Vec<H256>,
    /// Array of topics. topics are order-dependent.
    #[cfg_attr(
        feature = "serde",
        serde(default, rename = "fromBlock", skip_serializing_if = "Option::is_none")
    )]
    pub from: Option<AtBlock>,
    /// A hexadecimal block number, or the string latest, earliest or pending
    #[cfg_attr(
        feature = "serde",
        serde(default, rename = "toBlock", skip_serializing_if = "Option::is_none")
    )]
    pub to: Option<AtBlock>,
    #[cfg_attr(
        feature = "serde",
        serde(default, rename = "blockHash", skip_serializing_if = "Option::is_none")
    )]
    pub blockhash: Option<AtBlock>,
}

impl Default for BlockRange {
    fn default() -> Self {
        Self {
            address: Vec::new(),
            from: Some(AtBlock::Latest),
            to: Some(AtBlock::Latest),
            topics: Vec::new(),
            blockhash: None,
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;
    use hex_literal::hex;
    use rosetta_ethereum_types::BlockIdentifier;
    use serde_json::json;

    #[test]
    fn block_range_with_one_address_works() {
        let expected = BlockRange {
            address: vec![Address::from(hex!("1a94fce7ef36bc90959e206ba569a12afbc91ca1"))],
            from: None,
            to: None,
            topics: vec![H256(hex!(
                "241ea03ca20251805084d27d4440371c34a0b85ff108f6bb5611248f73818b80"
            ))],
            blockhash: Some(AtBlock::At(BlockIdentifier::Hash(H256(hex!(
                "7c5a35e9cb3e8ae0e221ab470abae9d446c3a5626ce6689fc777dcffcab52c70"
            ))))),
        };
        let json = json!({
            "address": "0x1a94fce7ef36bc90959e206ba569a12afbc91ca1",
            "topics":["0x241ea03ca20251805084d27d4440371c34a0b85ff108f6bb5611248f73818b80"],
            "blockHash": "0x7c5a35e9cb3e8ae0e221ab470abae9d446c3a5626ce6689fc777dcffcab52c70",
        });
        // Decode works
        let actual = serde_json::from_value::<BlockRange>(json.clone()).unwrap();
        assert_eq!(expected, actual);

        // Encode works
        let encoded = serde_json::to_value(expected).unwrap();
        assert_eq!(json, encoded);
    }

    #[test]
    fn block_range_with_many_addresses_works() {
        let expected = BlockRange {
            address: vec![
                Address::from(hex!("1a94fce7ef36bc90959e206ba569a12afbc91ca1")),
                Address::from(hex!("86e4dc95c7fbdbf52e33d563bbdb00823894c287")),
            ],
            from: None,
            to: None,
            topics: vec![H256(hex!(
                "241ea03ca20251805084d27d4440371c34a0b85ff108f6bb5611248f73818b80"
            ))],
            blockhash: Some(AtBlock::At(BlockIdentifier::Hash(H256(hex!(
                "7c5a35e9cb3e8ae0e221ab470abae9d446c3a5626ce6689fc777dcffcab52c70"
            ))))),
        };
        let json = json!({
            "address": ["0x1a94fce7ef36bc90959e206ba569a12afbc91ca1", "0x86e4dc95c7fbdbf52e33d563bbdb00823894c287"],
            "topics":["0x241ea03ca20251805084d27d4440371c34a0b85ff108f6bb5611248f73818b80"],
            "blockHash": "0x7c5a35e9cb3e8ae0e221ab470abae9d446c3a5626ce6689fc777dcffcab52c70",
        });

        // Decode works
        let actual = serde_json::from_value::<BlockRange>(json.clone()).unwrap();
        assert_eq!(expected, actual);

        // Encode works
        let encoded = serde_json::to_value(actual).unwrap();
        assert_eq!(json, encoded);
    }
}
