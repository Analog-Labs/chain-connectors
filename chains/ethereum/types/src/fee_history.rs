use crate::{
    // bytes::Bytes,
    // eth_hash::H256,
    eth_uint::U256,
    // header::{Header, SealedHeader},
    rstd::vec::Vec,
};
use num_rational::Rational64;

#[cfg(feature = "serde")]
use crate::serde_utils::numeric_to_rational;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
// #[cfg_attr(
//     feature = "with-codec",
//     derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
// )]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct FeeHistory {
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub base_fee_per_gas: Vec<U256>,

    #[cfg_attr(feature = "serde", serde(default, with = "numeric_to_rational"))]
    pub gas_used_ratio: Vec<Rational64>,

    pub oldest_block: U256,

    /// An (optional) array of effective priority fee per gas data points from a single block. All
    /// zeroes are returned if the block is empty.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub reward: Vec<Vec<U256>>,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    #[test]
    fn fee_history_serialization_works() {
        let fee_history_json = serde_json::json!({
            "baseFeePerGas": [
                "0x3da8e7618",
                "0x3e1ba3b1b",
                "0x3dfd72b90",
                "0x3d64eee76",
                "0x3d4da2da0",
                "0x3ccbcac6b"
            ],
            "gasUsedRatio": [
                0.529_074_766_666_666_6,
                0.492_404_533_333_333_34,
                0.461_557_6,
                0.494_070_833_333_333_35,
                0.466_905_3
            ],
            "oldestBlock": "0xfab8ac",
            "reward": [
                [
                    "0x59682f00",
                    "0x59682f00"
                ],
                [
                    "0x59682f00",
                    "0x59682f00"
                ],
                [
                    "0x3b9aca00",
                    "0x59682f00"
                ],
                [
                    "0x510b0870",
                    "0x59682f00"
                ],
                [
                    "0x3b9aca00",
                    "0x59682f00"
                ]
            ]
        });
        let expect = FeeHistory {
            base_fee_per_gas: vec![
                U256::from(0x0003_da8e_7618u64),
                U256::from(0x0003_e1ba_3b1bu64),
                U256::from(0x0003_dfd7_2b90u64),
                U256::from(0x0003_d64e_ee76u64),
                U256::from(0x0003_d4da_2da0u64),
                U256::from(0x0003_ccbc_ac6bu64),
            ],
            gas_used_ratio: vec![
                Rational64::approximate_float(0.529_074_766_666_666_6).unwrap(),
                Rational64::approximate_float(0.492_404_533_333_333_34).unwrap(),
                Rational64::approximate_float(0.461_557_6).unwrap(),
                Rational64::approximate_float(0.494_070_833_333_333_35).unwrap(),
                Rational64::approximate_float(0.466_905_3).unwrap(),
            ],
            oldest_block: U256::from(0x00fa_b8ac),
            reward: vec![
                vec![U256::from(0x5968_2f00), U256::from(0x5968_2f00)],
                vec![U256::from(0x5968_2f00), U256::from(0x5968_2f00)],
                vec![U256::from(0x3b9a_ca00), U256::from(0x5968_2f00)],
                vec![U256::from(0x510b_0870), U256::from(0x5968_2f00)],
                vec![U256::from(0x3b9a_ca00), U256::from(0x5968_2f00)],
            ],
        };

        let deserialized: FeeHistory = serde_json::from_value(fee_history_json.clone()).unwrap();
        assert_eq!(deserialized, expect);

        let serialized = serde_json::to_value(&deserialized).unwrap();
        assert_eq!(serialized, fee_history_json);
    }
}
