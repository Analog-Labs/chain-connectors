use super::access_list::AccessList;
use crate::rstd::vec::Vec;
use ethereum_types::{H160, U256};

#[cfg(feature = "serde")]
use crate::serde_utils::{bytes_to_hex, uint_to_hex};

/// Transactions with type 0x2 are transactions introduced in EIP-1559, included in Ethereum's
/// London fork. EIP-1559 addresses the network congestion and overpricing of transaction fees
/// caused by the historical fee market, in which users send transactions specifying a gas price bid
/// using the gasPrice parameter, and miners choose transactions with the highest bids.
///
/// EIP-1559 transactions don’t specify gasPrice, and instead use an in-protocol, dynamically
/// changing base fee per gas. At each block, the base fee per gas is adjusted to address network
/// congestion as measured by a gas target.
///
/// An EIP-1559 transaction always pays the base fee of the block it’s included in, and it pays a
/// priority fee as priced by `max_priority_fee_per_gas` or, if the base fee per gas +
/// `max_priority_fee_per_gas` exceeds `max_fee_per_gas`, it pays a priority fee as priced by
/// `max_fee_per_gas` minus the base fee per gas. The base fee is burned, and the priority fee is
/// paid to the miner that included the transaction. A transaction’s priority fee per gas
/// incentivizes miners to include the transaction over other transactions with lower priority fees
/// per gas.
#[derive(Clone, Default, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "scale-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Eip1559Transaction {
    /// The chain ID of the transaction. It is mandatory for EIP-1559 transactions.
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    /// [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub chain_id: u64,

    /// The nonce of the transaction.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub nonce: u64,

    /// Represents the maximum tx fee that will go to the miner as part of the user's
    /// fee payment. It serves 3 purposes:
    /// 1. Compensates miners for the uncle/ommer risk + fixed costs of including transaction in a
    /// block;
    /// 2. Allows users with high opportunity costs to pay a premium to miners;
    /// 3. In times where demand exceeds the available block space (i.e. 100% full, 30mm gas),
    /// this component allows first price auctions (i.e. the pre-1559 fee model) to happen on the
    /// priority fee.
    ///
    /// Incorporated as part of the London upgrade via [EIP-1559].
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    pub max_priority_fee_per_gas: U256,

    /// Represents the maximum amount that a user is willing to pay for their tx (inclusive of
    /// baseFeePerGas and maxPriorityFeePerGas). The difference between maxFeePerGas and
    /// baseFeePerGas + maxPriorityFeePerGas is “refunded” to the user.
    ///
    /// Incorporated as part of the London upgrade via [EIP-1559].
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    pub max_fee_per_gas: U256,

    /// Supplied gas
    #[cfg_attr(feature = "serde", serde(rename = "gas", with = "uint_to_hex",))]
    pub gas_limit: u64,

    /// Recipient address (None for contract creation)
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub to: Option<H160>,

    /// Transferred value
    pub value: U256,

    /// The data of the transaction.
    #[cfg_attr(
        feature = "serde",
        serde(with = "bytes_to_hex", skip_serializing_if = "Vec::is_empty")
    )]
    pub data: Vec<u8>,

    /// Optional access list introduced in EIP-2930.
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "AccessList::is_empty"))]
    pub access_list: AccessList,
}

#[cfg(all(test, feature = "serde"))]
pub mod tests {
    use super::Eip1559Transaction;
    use crate::transaction::{AccessList, AccessListItem};
    use ethereum_types::{H160, H256};
    use hex_literal::hex;

    pub fn build_eip1559() -> (Eip1559Transaction, serde_json::Value) {
        let tx = Eip1559Transaction {
            chain_id: 1,
            nonce: 117,
            max_priority_fee_per_gas: 100_000_000.into(),
            max_fee_per_gas: 28_379_509_371u128.into(),
            gas_limit: 187_293,
            to: Some(hex!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad").into()),
            value: 3_650_000_000_000_000_000u128.into(),
            data: hex!("3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000").to_vec(),
            access_list: AccessList(vec![AccessListItem {
                address: H160::from(hex!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad")),
                storage_keys: vec![
                    H256::zero(),
                    H256::from(hex!(
                        "a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6"
                    )),
                    H256::from(hex!(
                        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                    )),
                ],
            }]),
        };
        let json = serde_json::json!({
            "chainId": "0x1",
            "nonce": "0x75",
            "maxPriorityFeePerGas": "0x5f5e100",
            "maxFeePerGas": "0x69b8cf27b",
            "gas": "0x2db9d",
            "to": "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad",
            "value": "0x32a767a9562d0000",
            "data": "0x3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000",
            "accessList": [
                {
                    "address": "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad",
                    "storageKeys": [
                        "0x0000000000000000000000000000000000000000000000000000000000000000",
                        "0xa19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6",
                        "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                    ]
                }
            ],
            // "v": "0x1",
            // "r": "0xbde8e920a9acce0c9950f112d02d457d517835297b2610b4d0bcd56df114010f",
            // "s": "0x66ee7972cde2c5bd85fdb06aa358da04944b3ad5e56fe3e06d8fcb1137a52939"
        });
        (tx, json)
    }

    #[test]
    fn serde_encode_works() {
        let (tx, expected) = build_eip1559();
        let actual = serde_json::to_value(&tx).unwrap();
        assert_eq!(expected, actual);

        // can decode json
        let json_str = serde_json::to_string(&tx).unwrap();
        let decoded = serde_json::from_str::<Eip1559Transaction>(&json_str).unwrap();
        assert_eq!(tx, decoded);
    }
}
