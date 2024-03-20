use rosetta_ethereum_types::{Address, AtBlock, BlockIdentifier, H256};

/// Represents the target range of blocks for the filter
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub enum FilterBlockOption {
    Range { from_block: Option<AtBlock>, to_block: Option<AtBlock> },
    AtBlockHash(H256),
}

impl From<H256> for FilterBlockOption {
    fn from(hash: H256) -> Self {
        Self::AtBlockHash(hash)
    }
}

impl From<u64> for FilterBlockOption {
    fn from(block_number: u64) -> Self {
        Self::Range {
            from_block: Some(AtBlock::At(BlockIdentifier::Number(block_number))),
            to_block: Some(AtBlock::At(BlockIdentifier::Number(block_number))),
        }
    }
}

impl From<AtBlock> for FilterBlockOption {
    fn from(at: AtBlock) -> Self {
        match at {
            AtBlock::At(BlockIdentifier::Hash(hash)) => Self::AtBlockHash(hash),
            _ => Self::Range { from_block: Some(at), to_block: Some(at) },
        }
    }
}

impl From<BlockIdentifier> for FilterBlockOption {
    fn from(identifier: BlockIdentifier) -> Self {
        match identifier {
            BlockIdentifier::Hash(hash) => Self::AtBlockHash(hash),
            BlockIdentifier::Number(_) => Self::Range {
                from_block: Some(AtBlock::At(identifier)),
                to_block: Some(AtBlock::At(identifier)),
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
pub struct BlockRange {
    /// A list of addresses from which logs should originate.
    pub address: Vec<Address>,

    /// Array of topics. topics are order-dependent.
    pub topics: Vec<H256>,

    /// Array of topics. topics are order-dependent.
    pub filter: FilterBlockOption,
}

impl Default for BlockRange {
    fn default() -> Self {
        Self {
            address: Vec::new(),
            topics: Vec::new(),
            filter: FilterBlockOption::Range { from_block: None, to_block: None },
        }
    }
}

#[cfg(feature = "serde")]
use serde::ser::SerializeStruct;

#[cfg(feature = "serde")]
impl serde::Serialize for BlockRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Filter", 5)?;
        match self.filter {
            FilterBlockOption::Range { from_block, to_block } => {
                if let Some(ref from_block) = from_block {
                    s.serialize_field("fromBlock", from_block)?;
                }

                if let Some(ref to_block) = to_block {
                    s.serialize_field("toBlock", to_block)?;
                }
            },
            FilterBlockOption::AtBlockHash(ref h) => s.serialize_field("blockHash", h)?,
        }

        match self.address.len() {
            // Empty array is serialized as `None`
            0 => {},
            // Single element is serialized as the element itself
            1 => s.serialize_field("address", &self.address[0])?,
            // Multiple elements are serialized as an array
            _ => s.serialize_field("address", &self.address)?,
        }
        if !self.topics.is_empty() {
            s.serialize_field("topics", &self.topics)?;
        }
        s.end()
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;
    use hex_literal::hex;
    use serde_json::json;

    #[test]
    fn block_range_with_one_address_works() {
        let expected = BlockRange {
            address: vec![Address::from(hex!("1a94fce7ef36bc90959e206ba569a12afbc91ca1"))],
            topics: vec![H256(hex!(
                "241ea03ca20251805084d27d4440371c34a0b85ff108f6bb5611248f73818b80"
            ))],
            filter: FilterBlockOption::AtBlockHash(H256(hex!(
                "7c5a35e9cb3e8ae0e221ab470abae9d446c3a5626ce6689fc777dcffcab52c70"
            ))),
        };
        let json = json!({
            "address": "0x1a94fce7ef36bc90959e206ba569a12afbc91ca1",
            "topics":["0x241ea03ca20251805084d27d4440371c34a0b85ff108f6bb5611248f73818b80"],
            "blockHash": "0x7c5a35e9cb3e8ae0e221ab470abae9d446c3a5626ce6689fc777dcffcab52c70",
        });

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
            topics: vec![H256(hex!(
                "241ea03ca20251805084d27d4440371c34a0b85ff108f6bb5611248f73818b80"
            ))],
            filter: FilterBlockOption::AtBlockHash(H256(hex!(
                "7c5a35e9cb3e8ae0e221ab470abae9d446c3a5626ce6689fc777dcffcab52c70"
            ))),
        };
        let json = json!({
            "address": ["0x1a94fce7ef36bc90959e206ba569a12afbc91ca1", "0x86e4dc95c7fbdbf52e33d563bbdb00823894c287"],
            "topics":["0x241ea03ca20251805084d27d4440371c34a0b85ff108f6bb5611248f73818b80"],
            "blockHash": "0x7c5a35e9cb3e8ae0e221ab470abae9d446c3a5626ce6689fc777dcffcab52c70",
        });

        // Encode works
        let encoded = serde_json::to_value(expected).unwrap();
        assert_eq!(json, encoded);
    }
}
