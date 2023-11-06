#![allow(clippy::missing_errors_doc)]

use super::{access_list::AccessList, signature::Signature};
use crate::{
    bytes::Bytes,
    eth_hash::Address,
    eth_uint::{U256, U64},
};

#[cfg(feature = "with-rlp")]
use crate::rlp_utils::{RlpDecodableTransaction, RlpEncodableTransaction, RlpExt, RlpStreamExt};

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
pub struct Eip1559Transaction {
    /// The chain ID of the transaction. It is mandatory for EIP-1559 transactions.
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    /// [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    pub chain_id: U64,

    /// The nonce of the transaction.
    pub nonce: U64,

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
    #[cfg_attr(feature = "with-serde", serde(rename = "gas"))]
    pub gas_limit: U64,

    /// Recipient address (None for contract creation)
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Option::is_none"))]
    pub to: Option<Address>,

    /// Transferred value
    pub value: U256,

    /// The data of the transaction.
    #[cfg_attr(feature = "with-serde", serde(skip_serializing_if = "Bytes::is_empty"))]
    pub data: Bytes,

    /// Optional access list introduced in EIP-2930.
    /// [EIP-2930]: https://eips.ethereum.org/EIPS/eip-2930
    #[cfg_attr(
        feature = "with-serde",
        serde(default, skip_serializing_if = "AccessList::is_empty")
    )]
    pub access_list: AccessList,
}

#[cfg(feature = "with-rlp")]
impl RlpDecodableTransaction for Eip1559Transaction {
    fn rlp_decode(
        rlp: &rlp::Rlp,
        decode_signature: bool,
    ) -> Result<(Self, Option<Signature>), rlp::DecoderError> {
        let first = *rlp.data()?.first().ok_or(rlp::DecoderError::RlpIsTooShort)?;

        // Verify EIP-1559 transaction type (0x02)
        if first != 0x02 {
            return Err(rlp::DecoderError::Custom("invalid transaction type"));
        }

        let rest = rlp::Rlp::new(
            rlp.as_raw()
                .get(1..)
                .ok_or(rlp::DecoderError::Custom("missing transaction payload"))?,
        );

        // Check if is signed
        let is_signed = match rest.item_count()? {
            9 => false,
            12 => true,
            _ => return Err(rlp::DecoderError::RlpIncorrectListLen),
        };

        // Decode transaction
        let tx = Self {
            chain_id: rest.val_at(0usize)?,
            nonce: rest.val_at(1usize)?,
            max_priority_fee_per_gas: rest.val_at(2usize)?,
            max_fee_per_gas: rest.val_at(3usize)?,
            gas_limit: rest.val_at(4usize)?,
            to: rest.opt_at(5usize)?,
            value: rest.val_at(6usize)?,
            data: rest.val_at(7usize)?,
            access_list: rest.val_at(8usize)?,
        };

        // Decode signature
        let signature = if is_signed && decode_signature {
            Some(Signature {
                v: rest.val_at(9usize)?,
                r: rest.val_at(10usize)?,
                s: rest.val_at(11usize)?,
            })
        } else {
            None
        };

        Ok((tx, signature))
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Decodable for Eip1559Transaction {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        <Self as RlpDecodableTransaction>::rlp_decode_unsigned(rlp)
    }
}

#[cfg(feature = "with-rlp")]
impl RlpEncodableTransaction for Eip1559Transaction {
    fn rlp_append(&self, stream: &mut rlp::RlpStream, signature: Option<&Signature>) {
        // Append EIP-1559 transaction type (0x02)
        stream.append_internal(&2u8);
        let mut num_fields = 9;
        if signature.is_some() {
            num_fields += 3;
        }

        stream
            .begin_list(num_fields)
            .append(&self.chain_id)
            .append(&self.nonce)
            .append(&self.max_priority_fee_per_gas)
            .append(&self.max_fee_per_gas)
            .append(&self.gas_limit)
            .append_opt(self.to.as_ref())
            .append(&self.value)
            .append(&self.data)
            .append(&self.access_list);

        if let Some(sig) = signature {
            let v = sig.v.y_parity();
            stream.append(&v).append(&sig.r).append(&sig.s);
        }
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Encodable for Eip1559Transaction {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        <Self as RlpEncodableTransaction>::rlp_append(self, s, None);
    }
}

#[cfg(all(feature = "with-rlp", feature = "with-crypto"))]
impl super::TransactionT for Eip1559Transaction {
    type ExtraFields = ();

    fn compute_tx_hash(&self, signature: &Signature) -> primitive_types::H256 {
        use sha3::Digest;
        let input = self.rlp_signed(signature);
        let hash: [u8; 32] = sha3::Keccak256::digest(input.as_ref()).into();
        crate::eth_hash::H256(hash)
    }

    fn chain_id(&self) -> Option<u64> {
        Some(self.chain_id.as_u64())
    }

    fn nonce(&self) -> u64 {
        self.nonce.as_u64()
    }

    fn gas_price(&self) -> super::GasPrice {
        super::GasPrice::Eip1559 {
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
            max_fee_per_gas: self.max_fee_per_gas,
        }
    }

    fn gas_limit(&self) -> U256 {
        self.gas_limit.into()
    }

    fn to(&self) -> Option<Address> {
        self.to
    }

    fn value(&self) -> U256 {
        self.value
    }

    fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    fn sighash(&self) -> crate::eth_hash::H256 {
        use sha3::Digest;
        let input = self.rlp_unsigned();
        let hash: [u8; 32] = sha3::Keccak256::digest(input.as_ref()).into();
        crate::eth_hash::H256(hash)
    }

    fn access_list(&self) -> Option<&AccessList> {
        Some(&self.access_list)
    }

    fn transaction_type(&self) -> Option<u8> {
        Some(0x02)
    }

    fn extra_fields(&self) -> Option<Self::ExtraFields> {
        None
    }
}

#[cfg(all(test, any(feature = "with-serde", feature = "with-rlp")))]
mod tests {
    use super::{super::signature::RecoveryId, Address, Bytes, Eip1559Transaction, Signature};
    use crate::{
        eth_hash::H256,
        transactions::access_list::{AccessList, AccessListItem},
    };
    use hex_literal::hex;

    #[cfg(feature = "with-rlp")]
    static RLP_EIP1559_SIGNED: &[u8] = &hex!("02f9037701758405f5e10085069b8cf27b8302db9d943fc91a3afd70395cd496c647d5a6cc9d4b2b7fad8832a767a9562d0000b902843593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000f87cf87a943fc91a3afd70395cd496c647d5a6cc9d4b2b7fadf863a00000000000000000000000000000000000000000000000000000000000000000a0a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6a0ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01a0bde8e920a9acce0c9950f112d02d457d517835297b2610b4d0bcd56df114010fa066ee7972cde2c5bd85fdb06aa358da04944b3ad5e56fe3e06d8fcb1137a52939");
    #[cfg(feature = "with-rlp")]
    static RLP_EIP1559_UNSIGNED: &[u8] = &hex!("02f9033401758405f5e10085069b8cf27b8302db9d943fc91a3afd70395cd496c647d5a6cc9d4b2b7fad8832a767a9562d0000b902843593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000f87cf87a943fc91a3afd70395cd496c647d5a6cc9d4b2b7fadf863a00000000000000000000000000000000000000000000000000000000000000000a0a19fd53308a1c44a3ed22d3f20ed4229aa8909e0d0a90510ca482367ad42caa6a0ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");

    fn build_eip1559() -> (Eip1559Transaction, Signature) {
        let tx = Eip1559Transaction {
            chain_id: 1.into(),
            nonce: 117.into(),
            max_priority_fee_per_gas: 100_000_000.into(),
            max_fee_per_gas: 28_379_509_371u128.into(),
            gas_limit: 187_293.into(),
            to: Some(hex!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad").into()),
            value: 3_650_000_000_000_000_000u128.into(),
            data: hex!("3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006547d41700000000000000000000000000000000000000000000000000000000000000020b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000032a767a9562d00000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000032a767a9562d000000000000000000000000000000000000000000000021b60af11987fa0670342f00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002bc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000bb8b55ee890426341fe45ee6dc788d2d93d25b59063000000000000000000000000000000000000000000").into(),
            access_list: AccessList(vec![AccessListItem {
                address: Address::from(hex!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad")),
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
        let signature = Signature {
            v: RecoveryId::new(0x1),
            r: hex!("bde8e920a9acce0c9950f112d02d457d517835297b2610b4d0bcd56df114010f").into(),
            s: hex!("66ee7972cde2c5bd85fdb06aa358da04944b3ad5e56fe3e06d8fcb1137a52939").into(),
        };
        (tx, signature)
    }

    #[cfg(feature = "with-serde")]
    #[test]
    fn serde_encode_works() {
        let tx = build_eip1559().0;
        let actual = serde_json::to_value(&tx).unwrap();
        let expected = serde_json::json!({
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
        assert_eq!(expected, actual);

        // can decode json
        let json_str = serde_json::to_string(&tx).unwrap();
        let decoded = serde_json::from_str::<Eip1559Transaction>(&json_str).unwrap();
        assert_eq!(tx, decoded);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_encode_signed_works() {
        use crate::rlp_utils::RlpEncodableTransaction;
        let (tx, sig) = build_eip1559();
        let expected = Bytes::from_static(RLP_EIP1559_SIGNED);
        let actual = Bytes::from(tx.rlp_signed(&sig));
        assert_eq!(expected, actual);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_encode_unsigned_works() {
        use crate::rlp_utils::RlpEncodableTransaction;
        let tx = build_eip1559().0;
        let expected = Bytes::from_static(RLP_EIP1559_UNSIGNED);
        let actual = Bytes::from(tx.rlp_unsigned());
        assert_eq!(expected, actual);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_decode_signed_works() {
        use crate::rlp_utils::RlpDecodableTransaction;
        let (expected_tx, expected_sig) = build_eip1559();
        let (actual_tx, actual_sig) = {
            let rlp = rlp::Rlp::new(RLP_EIP1559_SIGNED);
            Eip1559Transaction::rlp_decode_signed(&rlp).unwrap()
        };
        assert_eq!(expected_tx, actual_tx);
        assert_eq!(Some(expected_sig), actual_sig);
    }

    #[cfg(feature = "with-rlp")]
    #[test]
    fn rlp_decode_unsigned_works() {
        use crate::rlp_utils::RlpDecodableTransaction;
        let expected = build_eip1559().0;

        // Can decode unsigned raw transaction
        let actual = {
            let rlp = rlp::Rlp::new(RLP_EIP1559_UNSIGNED);
            Eip1559Transaction::rlp_decode_unsigned(&rlp).unwrap()
        };
        assert_eq!(expected, actual);

        // Can decode signed raw transaction
        let actual = {
            let rlp = rlp::Rlp::new(RLP_EIP1559_SIGNED);
            Eip1559Transaction::rlp_decode_unsigned(&rlp).unwrap()
        };
        assert_eq!(expected, actual);
    }

    #[cfg(all(feature = "with-rlp", feature = "with-crypto"))]
    #[test]
    fn compute_eip1559_sighash() {
        use super::super::TransactionT;
        let tx = build_eip1559().0;
        let expected =
            H256(hex!("2fedc63a84e92359545438f62f816b374e316b3e15f3b2fd5705a7fc430c002e"));
        assert_eq!(expected, tx.sighash());
    }

    #[cfg(all(feature = "with-rlp", feature = "with-crypto"))]
    #[test]
    fn compute_eip1559_tx_hash() {
        use super::super::TransactionT;
        let (tx, sig) = build_eip1559();
        let expected =
            H256(hex!("20a0f172aaeefc91c346fa0d43a9e56a1058a2a0c0c6fa8a2e9204f8047d1008"));
        assert_eq!(expected, tx.compute_tx_hash(&sig));
    }
}
