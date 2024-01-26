#[cfg(feature = "with-serde")]
use crate::serde_utils::uint_to_hex;
use crate::{eth_hash::H520, eth_uint::U256};

/// An ECDSA signature
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "with-codec", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "with-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Signature {
    /// The ECDSA recovery id, this value encodes the parity of the y-coordinate of the secp256k1
    /// signature. May also encode the chain_id for legacy EIP-155 transactions.
    pub v: RecoveryId,
    /// The ECDSA signature r
    pub r: U256,
    /// The ECDSA signature s
    pub s: U256,
}

impl Signature {
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_raw_signature(&self, output: &mut [u8; 65]) {
        self.r.to_big_endian(&mut output[0..32]);
        self.s.to_big_endian(&mut output[32..64]);
        output[64] = self.v.y_parity() as u8;
    }
}

impl From<Signature> for H520 {
    fn from(value: Signature) -> Self {
        let mut output = [0u8; 65];
        value.to_raw_signature(&mut output);
        Self(output)
    }
}

/// The ECDSA recovery id, encodes the parity of the y-coordinate and for EIP-155 compatible
/// transactions also encodes the chain id
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "with-codec", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "with-codec", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
#[cfg_attr(
    feature = "with-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct RecoveryId(#[cfg_attr(feature = "with-serde", serde(with = "uint_to_hex"))] u64);

impl RecoveryId {
    #[must_use]
    pub fn new(v: u64) -> Self {
        debug_assert!(v >= 35 || matches!(v, 0 | 1 | 27 | 28));
        Self(v)
    }

    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the parity (0 for even, 1 for odd) of the y-value of a secp256k1 signature.
    #[must_use]
    pub const fn y_parity(self) -> u64 {
        let v = self.as_u64();

        // if v is greather or equal to 35, it is an EIP-155 signature
        // [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
        if v >= 35 {
            return (v - 35) & 1;
        }

        // 27 or 28, it is a legacy signature
        if v == 27 || v == 28 {
            return v - 27;
        }

        // otherwise, simply return the parity of the least significant bit
        v & 1
    }

    #[must_use]
    pub const fn chain_id(self) -> Option<u64> {
        let v = self.as_u64();
        if v >= 35 {
            Some((v - 35) >> 1)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn is_eip155(self) -> bool {
        self.chain_id().is_some()
    }

    /// Applies [EIP155](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md)
    #[must_use]
    pub fn as_eip155<I: Into<u64>>(self, chain_id: I) -> u64 {
        let chain_id = chain_id.into();
        self.y_parity() + 35 + (chain_id * 2)
    }

    /// the recovery id is encoded as 0 or 1 for EIP-2930.
    #[must_use]
    pub const fn is_eip2930(self) -> bool {
        self.as_u64() < 2
    }

    /// Returns a legacy signature, with
    #[must_use]
    pub const fn as_legacy(self) -> u64 {
        self.y_parity() + 27
    }
}

impl From<RecoveryId> for u64 {
    fn from(v: RecoveryId) -> Self {
        v.as_u64()
    }
}

impl From<u64> for RecoveryId {
    fn from(v: u64) -> Self {
        Self::new(v)
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Encodable for RecoveryId {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let v = self.as_u64();
        <u64 as rlp::Encodable>::rlp_append(&v, s);
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Decodable for RecoveryId {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let v = <u64 as rlp::Decodable>::decode(rlp)?;
        Ok(Self::new(v))
    }
}
