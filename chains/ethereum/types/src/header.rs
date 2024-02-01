#[cfg(feature = "serde")]
use crate::serde_utils::{bytes_to_hex, uint_to_hex};
use crate::{
    bytes::Bytes,
    constants::{EMPTY_OMMER_ROOT_HASH, EMPTY_ROOT_HASH},
    eth_hash::{Address, H256},
    eth_uint::U256,
};
#[cfg(feature = "with-rlp")]
use crate::{crypto::Crypto, eth_hash::H64, rlp_utils::RlpExt, transactions::SignedTransactionT};
pub use ethbloom;
use ethbloom::Bloom;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct Header {
    /// The Keccak 256-bit hash of the parent
    /// block’s header, in its entirety; formally Hp.
    pub parent_hash: H256,
    /// The Keccak 256-bit hash of the ommers list portion of this block; formally Ho.
    #[cfg_attr(feature = "serde", serde(rename = "sha3Uncles"))]
    pub ommers_hash: H256,
    /// The 160-bit address to which all fees collected from the successful mining of this block
    /// be transferred; formally Hc.
    #[cfg_attr(feature = "serde", serde(rename = "miner", alias = "beneficiary"))]
    pub beneficiary: Address,
    /// The Keccak 256-bit hash of the root node of the state trie, after all transactions are
    /// executed and finalisations applied; formally Hr.
    pub state_root: H256,
    /// The Keccak 256-bit hash of the root node of the trie structure populated with each
    /// transaction in the transactions list portion of the block; formally Ht.
    pub transactions_root: H256,
    /// The Keccak 256-bit hash of the root node of the trie structure populated with the receipts
    /// of each transaction in the transactions list portion of the block; formally He.
    pub receipts_root: H256,
    /// The Bloom filter composed from indexable information (logger address and log topics)
    /// contained in each log entry from the receipt of each transaction in the transactions list;
    /// formally Hb.
    pub logs_bloom: Bloom,
    /// A scalar value corresponding to the difficulty level of this block. This can be calculated
    /// from the previous block’s difficulty level and the timestamp; formally Hd.
    pub difficulty: U256,
    /// A scalar value equal to the number of ancestor blocks. The genesis block has a number of
    /// zero; formally Hi.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub number: u64,
    /// A scalar value equal to the current limit of gas expenditure per block; formally Hl.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub gas_limit: u64,
    /// A scalar value equal to the total gas used in transactions in this block; formally Hg.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub gas_used: u64,
    /// A scalar value equal to the reasonable output of Unix’s time() at this block’s inception;
    /// formally Hs.
    #[cfg_attr(feature = "serde", serde(with = "uint_to_hex"))]
    pub timestamp: u64,
    /// An arbitrary byte array containing data relevant to this block. This must be 32 bytes or
    /// fewer; formally Hx.
    #[cfg_attr(feature = "serde", serde(default))]
    pub extra_data: Bytes,
    /// A 256-bit hash which, combined with the
    /// nonce, proves that a sufficient amount of computation has been carried out on this block;
    /// formally Hm.
    #[cfg_attr(feature = "serde", serde(default))]
    pub mix_hash: H256,
    /// A 64-bit value which, combined with the mixhash, proves that a sufficient amount of
    /// computation has been carried out on this block; formally Hn.
    #[cfg_attr(
        feature = "serde",
        serde(
            deserialize_with = "uint_to_hex::deserialize",
            serialize_with = "bytes_to_hex::serialize"
        )
    )]
    pub nonce: u64,
    /// A scalar representing EIP1559 base fee which can move up or down each block according
    /// to a formula which is a function of gas used in parent block and gas target
    /// (block gas limit divided by elasticity multiplier) of parent block.
    /// The algorithm results in the base fee per gas increasing when blocks are
    /// above the gas target, and decreasing when blocks are below the gas target. The base fee per
    /// gas is burned.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none", with = "uint_to_hex",)
    )]
    pub base_fee_per_gas: Option<u64>,
    /// The Keccak 256-bit hash of the withdrawals list portion of this block.
    /// <https://eips.ethereum.org/EIPS/eip-4895>
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub withdrawals_root: Option<H256>,
    /// The total amount of blob gas consumed by the transactions within the block, added in
    /// EIP-4844.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none", with = "uint_to_hex",)
    )]
    pub blob_gas_used: Option<u64>,
    /// A running total of blob gas consumed in excess of the target, prior to the block. Blocks
    /// with above-target blob gas consumption increase this value, blocks with below-target blob
    /// gas consumption decrease it (bounded at 0). This was added in EIP-4844.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none", with = "uint_to_hex",)
    )]
    pub excess_blob_gas: Option<u64>,
    /// The hash of the parent beacon block's root is included in execution blocks, as proposed by
    /// EIP-4788.
    ///
    /// This enables trust-minimized access to consensus state, supporting staking pools, bridges,
    /// and more.
    ///
    /// The beacon roots contract handles root storage, enhancing Ethereum's functionalities.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub parent_beacon_block_root: Option<H256>,
}

#[cfg(feature = "with-rlp")]
impl Header {
    /// Seal the block with a known hash.
    ///
    /// WARNING: This method does not perform validation whether the hash is correct.
    #[must_use]
    pub const fn seal(self, hash: H256) -> SealedHeader {
        SealedHeader::new(self, hash)
    }

    /// Compute the block hash and seal the header.
    #[must_use]
    pub fn seal_slow<C: Crypto>(self) -> SealedHeader {
        let hash = self.compute_hash::<C>();
        SealedHeader::new(self, hash)
    }

    /// Compute the block hash.
    #[must_use]
    pub fn compute_hash<C: Crypto>(&self) -> H256 {
        let bytes = rlp::Encodable::rlp_bytes(self).freeze();
        C::keccak256(bytes)
    }

    /// Decode header from bytes.
    /// # Errors
    /// Returns an error if the header cannot be decoded.
    pub fn decode(bytes: &[u8]) -> Result<Self, rlp::DecoderError> {
        let rlp = rlp::Rlp::new(bytes);
        <Self as rlp::Decodable>::decode(&rlp)
    }

    /// RLP encoded header.
    pub fn encode(&self) -> Bytes {
        let bytes = rlp::Encodable::rlp_bytes(self).freeze();
        Bytes(bytes)
    }

    /// Calculate transaction root.
    pub fn compute_transaction_root<'a, C, T, I>(transactions: I) -> H256
    where
        C: Crypto,
        T: SignedTransactionT + 'a,
        I: Iterator<Item = &'a T> + 'a,
    {
        C::trie_root(transactions.map(SignedTransactionT::encode_signed))
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            parent_hash: H256::zero(),
            ommers_hash: EMPTY_OMMER_ROOT_HASH,
            beneficiary: Address::zero(),
            state_root: EMPTY_ROOT_HASH,
            transactions_root: EMPTY_ROOT_HASH,
            receipts_root: EMPTY_ROOT_HASH,
            logs_bloom: Bloom::zero(),
            difficulty: U256::zero(),
            number: 0,
            gas_limit: 0,
            gas_used: 0,
            timestamp: 0,
            extra_data: Bytes::default(),
            mix_hash: H256::zero(),
            nonce: 0,
            base_fee_per_gas: None,
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
        }
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Decodable for Header {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let total_items = rlp.item_count()?;
        if !(15..=20).contains(&total_items) {
            return Err(rlp::DecoderError::RlpIncorrectListLen);
        }

        let mut this = Self {
            parent_hash: rlp.val_at(0)?,
            ommers_hash: rlp.val_at(1)?,
            beneficiary: rlp.val_at(2)?,
            state_root: rlp.val_at(3)?,
            transactions_root: rlp.val_at(4)?,
            receipts_root: rlp.val_at(5)?,
            logs_bloom: rlp.val_at(6)?,
            difficulty: rlp.val_at(7)?,
            number: rlp.val_at(8)?,
            gas_limit: rlp.val_at(9)?,
            gas_used: rlp.val_at(10)?,
            timestamp: rlp.val_at(11)?,
            extra_data: rlp.val_at(12)?,
            mix_hash: rlp.val_at(13)?,
            nonce: u64::from_be_bytes(rlp.val_at::<H64>(14)?.0),
            base_fee_per_gas: None,
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
        };

        if total_items > 15 {
            let rlp = rlp.at(15)?;
            if !rlp.is_empty() {
                this.base_fee_per_gas = Some(rlp.as_val()?);
            }
        }

        // Withdrawals root for post-shanghai headers
        if total_items > 16 {
            this.withdrawals_root = rlp.opt_at(16)?;
        }

        // Blob gas used and excess blob gas for post-cancun headers
        if total_items > 17 {
            this.blob_gas_used = Some(rlp.val_at(17)?);
        }

        if total_items > 18 {
            this.excess_blob_gas = Some(rlp.val_at(18)?);
        }

        // Decode parent beacon block root. If new fields are added, the above pattern will need to
        // be repeated and placeholders decoded. Otherwise, it's impossible to tell _which_
        // fields are missing. This is mainly relevant for contrived cases where a header is
        // created at random, for example:
        //  * A header is created with a withdrawals root, but no base fee. Shanghai blocks are
        //    post-London, so this is technically not valid. However, a tool like proptest would
        //    generate a block like this.
        if total_items > 19 {
            this.parent_beacon_block_root = Some(rlp.val_at(19)?);
        }

        Ok(this)
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Encodable for Header {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let mut size = 15;
        if self.base_fee_per_gas.is_some() ||
            self.withdrawals_root.is_some() ||
            self.blob_gas_used.is_some() ||
            self.excess_blob_gas.is_some() ||
            self.parent_beacon_block_root.is_some()
        {
            size += 1;
        }
        if self.withdrawals_root.is_some() ||
            self.blob_gas_used.is_some() ||
            self.excess_blob_gas.is_some() ||
            self.parent_beacon_block_root.is_some()
        {
            size += 1;
        }
        if self.blob_gas_used.is_some() ||
            self.excess_blob_gas.is_some() ||
            self.parent_beacon_block_root.is_some()
        {
            size += 1;
        }
        if self.excess_blob_gas.is_some() || self.parent_beacon_block_root.is_some() {
            size += 1;
        }
        if self.parent_beacon_block_root.is_some() {
            size += 1;
        }

        s.begin_list(size);
        s.append(&self.parent_hash);
        s.append(&self.ommers_hash);
        s.append(&self.beneficiary);
        s.append(&self.state_root);
        s.append(&self.transactions_root);
        s.append(&self.receipts_root);
        s.append(&self.logs_bloom);
        s.append(&self.difficulty);
        s.append(&U256::from(self.number));
        s.append(&U256::from(self.gas_limit));
        s.append(&U256::from(self.gas_used));
        s.append(&self.timestamp);
        s.append(&self.extra_data);
        s.append(&self.mix_hash);
        s.append(&H64(self.nonce.to_be_bytes()));

        // Encode base fee. Put empty list if base fee is missing,
        // but withdrawals root is present.
        if let Some(ref base_fee) = self.base_fee_per_gas {
            s.append(&U256::from(*base_fee));
        } else if self.withdrawals_root.is_some() ||
            self.blob_gas_used.is_some() ||
            self.excess_blob_gas.is_some() ||
            self.parent_beacon_block_root.is_some()
        {
            s.begin_list(0);
        }

        // Encode withdrawals root. Put empty string if withdrawals root is missing,
        // but blob gas used is present.
        if let Some(ref root) = self.withdrawals_root {
            s.append(root);
        } else if self.blob_gas_used.is_some() ||
            self.excess_blob_gas.is_some() ||
            self.parent_beacon_block_root.is_some()
        {
            s.append_empty_data();
        }

        // Encode blob gas used. Put empty list if blob gas used is missing,
        // but excess blob gas is present.
        if let Some(ref blob_gas_used) = self.blob_gas_used {
            s.append(&U256::from(*blob_gas_used));
        } else if self.excess_blob_gas.is_some() || self.parent_beacon_block_root.is_some() {
            s.begin_list(0);
        }

        // Encode excess blob gas. Put empty list if excess blob gas is missing,
        // but parent beacon block root is present.
        if let Some(ref excess_blob_gas) = self.excess_blob_gas {
            s.append(&U256::from(*excess_blob_gas));
        } else if self.parent_beacon_block_root.is_some() {
            s.begin_list(0);
        }

        // Encode parent beacon block root. If new fields are added, the above pattern will need to
        // be repeated and placeholders added. Otherwise, it's impossible to tell _which_
        // fields are missing. This is mainly relevant for contrived cases where a header is
        // created at random, for example:
        //  * A header is created with a withdrawals root, but no base fee. Shanghai blocks are
        //    post-London, so this is technically not valid. However, a tool like proptest would
        //    generate a block like this.
        if let Some(ref parent_beacon_block_root) = self.parent_beacon_block_root {
            s.append(parent_beacon_block_root);
        }
    }
}

/// A [`Header`] that is sealed at a precalculated hash, use [`SealedHeader::unseal()`] if you want
/// to modify header.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "with-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct SealedHeader {
    /// Locked Header hash.
    hash: H256,

    /// Locked Header fields.
    #[cfg_attr(feature = "serde", serde(flatten))]
    header: Header,
}

impl SealedHeader {
    /// Creates the sealed header with the corresponding block hash.
    #[must_use]
    #[inline]
    pub const fn new(header: Header, hash: H256) -> Self {
        Self { hash, header }
    }

    /// Unseal the header
    #[must_use]
    pub fn unseal(self) -> Header {
        self.header
    }

    /// Returns the sealed Header fields.
    #[must_use]
    #[inline]
    pub const fn header(&self) -> &Header {
        &self.header
    }

    /// Returns header/block hash.
    #[must_use]
    #[inline]
    pub const fn hash(&self) -> H256 {
        self.hash
    }
}

#[cfg(all(feature = "with-rlp", feature = "with-crypto"))]
impl rlp::Decodable for SealedHeader {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        use crate::crypto::DefaultCrypto;
        let header = <Header as rlp::Decodable>::decode(rlp)?;
        let hash = header.compute_hash::<DefaultCrypto>();
        Ok(Self::new(header, hash))
    }
}

#[cfg(feature = "with-rlp")]
impl rlp::Encodable for SealedHeader {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        self.header.rlp_append(s);
    }
}

#[cfg(all(test, feature = "with-rlp", feature = "with-crypto"))]
mod tests {
    use super::Header;
    use crate::{
        bytes::Bytes, constants::EMPTY_OMMER_ROOT_HASH, crypto::DefaultCrypto, eth_hash::H256,
        eth_uint::U256,
    };
    use ethbloom::Bloom;
    use hex_literal::hex;

    // Test vector from: https://eips.ethereum.org/EIPS/eip-2481
    #[test]
    fn test_encode_block_header() {
        let expected = hex!("f901f9a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000940000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008208ae820d0582115c8215b3821a0a827788a00000000000000000000000000000000000000000000000000000000000000000880000000000000000");
        let header = Header {
            difficulty: U256::from(0x8ae_u64),
            number: 0xd05_u64,
            gas_limit: 0x115c_u64,
            gas_used: 0x15b3_u64,
            timestamp: 0x1a0a_u64,
            extra_data: Bytes::from_static(&hex!("7788")),
            ommers_hash: H256::zero(),
            state_root: H256::zero(),
            transactions_root: H256::zero(),
            receipts_root: H256::zero(),
            ..Default::default()
        };
        // make sure encode works
        let encoded = header.encode();
        assert_eq!(encoded.as_ref(), expected.as_ref());

        // make sure the decode works
        let decoded = Header::decode(&expected).unwrap();
        assert_eq!(header, decoded);
    }

    // Test vector from: https://github.com/ethereum/tests/blob/f47bbef4da376a49c8fc3166f09ab8a6d182f765/BlockchainTests/ValidBlocks/bcEIP1559/baseFee.json#L15-L36
    #[test]
    fn test_eip1559_block_header_hash() {
        let expected_hash =
            H256(hex!("6a251c7c3c5dca7b42407a3752ff48f3bbca1fab7f9868371d9918daf1988d1f"));
        let header = Header {
            parent_hash: hex!("e0a94a7a3c9617401586b1a27025d2d9671332d22d540e0af72b069170380f2a").into(),
            ommers_hash: EMPTY_OMMER_ROOT_HASH,
            beneficiary: hex!("ba5e000000000000000000000000000000000000").into(),
            state_root: hex!("ec3c94b18b8a1cff7d60f8d258ec723312932928626b4c9355eb4ab3568ec7f7").into(),
            transactions_root: hex!("50f738580ed699f0469702c7ccc63ed2e51bc034be9479b7bff4e68dee84accf").into(),
            receipts_root: hex!("29b0562f7140574dd0d50dee8a271b22e1a0a7b78fca58f7c60370d8317ba2a9").into(),
            logs_bloom: hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").into(),
            difficulty: U256::from(0x020_000),
            number: 0x01_u64,
            gas_limit: 0x01_63_45_78_5d_8a_00_00_u64,
            gas_used: 0x015_534_u64,
            timestamp: 0x079e,
            extra_data: Bytes::from_static(&hex!("42")),
            mix_hash: hex!("0000000000000000000000000000000000000000000000000000000000000000").into(),
            nonce: 0,
            base_fee_per_gas: Some(0x036b_u64),
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
        };
        let actual_hash = header.compute_hash::<DefaultCrypto>();
        assert_eq!(actual_hash, expected_hash);
    }

    // Test vector from: https://eips.ethereum.org/EIPS/eip-2481
    #[test]
    fn test_decode_block_header() {
        let data = hex!("f901f9a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000940000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008208ae820d0582115c8215b3821a0a827788a00000000000000000000000000000000000000000000000000000000000000000880000000000000000");
        let expected = Header {
            difficulty: U256::from(0x8aeu64),
            number: 0xd05u64,
            gas_limit: 0x115cu64,
            gas_used: 0x15b3u64,
            timestamp: 0x1a0au64,
            extra_data: Bytes::from_static(&[0x77, 0x88]),
            ommers_hash: H256::zero(),
            state_root: H256::zero(),
            transactions_root: H256::zero(),
            receipts_root: H256::zero(),
            ..Default::default()
        };
        let header = Header::decode(&data).unwrap();
        assert_eq!(header, expected);

        // make sure the hash matches
        let expected_hash =
            H256(hex!("8c2f2af15b7b563b6ab1e09bed0e9caade7ed730aec98b70a993597a797579a9"));
        let actual_hash = header.compute_hash::<DefaultCrypto>();
        assert_eq!(actual_hash, expected_hash);
    }

    // Test vector from: https://github.com/ethereum/tests/blob/970503935aeb76f59adfa3b3224aabf25e77b83d/BlockchainTests/ValidBlocks/bcExample/shanghaiExample.json#L15-L34
    #[test]
    fn test_decode_block_header_with_withdrawals() {
        let data = hex!("f9021ca018db39e19931515b30b16b3a92c292398039e31d6c267111529c3f2ba0a26c17a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347942adc25665018aa1fe0e6bc666dac8fc2697ff9baa095efce3d6972874ca8b531b233b7a1d1ff0a56f08b20c8f1b89bef1b001194a5a071e515dd89e8a7973402c2e11646081b4e2209b2d3a1550df5095289dabcb3fba0ed9c51ea52c968e552e370a77a41dac98606e98b915092fb5f949d6452fce1c4b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008001887fffffffffffffff830125b882079e42a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b42188000000000000000009a027f166f1d7c789251299535cb176ba34116e44894476a7886fe5d73d9be5c973");
        let expected = Header {
            parent_hash: hex!("18db39e19931515b30b16b3a92c292398039e31d6c267111529c3f2ba0a26c17")
                .into(),
            beneficiary: hex!("2adc25665018aa1fe0e6bc666dac8fc2697ff9ba").into(),
            state_root: hex!("95efce3d6972874ca8b531b233b7a1d1ff0a56f08b20c8f1b89bef1b001194a5")
                .into(),
            transactions_root: hex!(
                "71e515dd89e8a7973402c2e11646081b4e2209b2d3a1550df5095289dabcb3fb"
            )
            .into(),
            receipts_root: hex!("ed9c51ea52c968e552e370a77a41dac98606e98b915092fb5f949d6452fce1c4")
                .into(),
            number: 0x01,
            gas_limit: 0x7fff_ffff_ffff_ffff,
            gas_used: 0x0001_25b8,
            timestamp: 0x079e,
            extra_data: Bytes::from_static(&[0x42]),
            mix_hash: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421")
                .into(),
            base_fee_per_gas: Some(0x09),
            withdrawals_root: Some(
                hex!("27f166f1d7c789251299535cb176ba34116e44894476a7886fe5d73d9be5c973").into(),
            ),
            ..Default::default()
        };
        let header = Header::decode(&data).unwrap();
        assert_eq!(header, expected);

        let expected_hash =
            H256(hex!("85fdec94c534fa0a1534720f167b899d1fc268925c71c0cbf5aaa213483f5a69"));
        let actual_hash = header.compute_hash::<DefaultCrypto>();
        assert_eq!(actual_hash, expected_hash);
    }

    // Test vector from: https://github.com/ethereum/tests/blob/7e9e0940c0fcdbead8af3078ede70f969109bd85/BlockchainTests/ValidBlocks/bcExample/cancunExample.json
    #[test]
    fn test_decode_block_header_with_blob_fields_ef_tests() {
        let data = hex!("f90221a03a9b485972e7353edd9152712492f0c58d89ef80623686b6bf947a4a6dce6cb6a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347942adc25665018aa1fe0e6bc666dac8fc2697ff9baa03c837fc158e3e93eafcaf2e658a02f5d8f99abc9f1c4c66cdea96c0ca26406aea04409cc4b699384ba5f8248d92b784713610c5ff9c1de51e9239da0dac76de9cea046cab26abf1047b5b119ecc2dda1296b071766c8b1307e1381fcecc90d513d86b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008001887fffffffffffffff8302a86582079e42a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b42188000000000000000009a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4218302000080");
        let expected = Header {
            parent_hash: hex!("3a9b485972e7353edd9152712492f0c58d89ef80623686b6bf947a4a6dce6cb6")
                .into(),
            ommers_hash: EMPTY_OMMER_ROOT_HASH,
            beneficiary: hex!("2adc25665018aa1fe0e6bc666dac8fc2697ff9ba").into(),
            state_root: hex!("3c837fc158e3e93eafcaf2e658a02f5d8f99abc9f1c4c66cdea96c0ca26406ae")
                .into(),
            transactions_root: hex!(
                "4409cc4b699384ba5f8248d92b784713610c5ff9c1de51e9239da0dac76de9ce"
            )
            .into(),
            receipts_root: hex!("46cab26abf1047b5b119ecc2dda1296b071766c8b1307e1381fcecc90d513d86")
                .into(),
            logs_bloom: Bloom::default(),
            difficulty: U256::zero(),
            number: 0x1,
            gas_limit: 0x7fff_ffff_ffff_ffff,
            gas_used: 0x0002_a865,
            timestamp: 0x079e,
            extra_data: Bytes::from(vec![0x42]),
            mix_hash: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421")
                .into(),
            nonce: 0,
            base_fee_per_gas: Some(9),
            withdrawals_root: Some(
                hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
            ),
            blob_gas_used: Some(0x0002_0000),
            excess_blob_gas: Some(0),
            parent_beacon_block_root: None,
        };

        let header = Header::decode(&data).unwrap();
        assert_eq!(header, expected);

        let expected_hash =
            H256(hex!("10aca3ebb4cf6ddd9e945a5db19385f9c105ede7374380c50d56384c3d233785"));
        let actual_hash = header.compute_hash::<DefaultCrypto>();
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn test_decode_block_header_with_blob_fields() {
        // Block from devnet-7
        let data = hex!("f90239a013a7ec98912f917b3e804654e37c9866092043c13eb8eab94eb64818e886cff5a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794f97e180c050e5ab072211ad2c213eb5aee4df134a0ec229dbe85b0d3643ad0f471e6ec1a36bbc87deffbbd970762d22a53b35d068aa056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080830305988401c9c380808464c40d5499d883010c01846765746888676f312e32302e35856c696e7578a070ccadc40b16e2094954b1064749cc6fbac783c1712f1b271a8aac3eda2f232588000000000000000007a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421808401600000");
        let expected = Header {
            parent_hash: hex!("13a7ec98912f917b3e804654e37c9866092043c13eb8eab94eb64818e886cff5")
                .into(),
            ommers_hash: EMPTY_OMMER_ROOT_HASH,
            beneficiary: hex!("f97e180c050e5ab072211ad2c213eb5aee4df134").into(),
            state_root: hex!("ec229dbe85b0d3643ad0f471e6ec1a36bbc87deffbbd970762d22a53b35d068a")
                .into(),
            transactions_root: hex!(
                "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"
            )
            .into(),
            receipts_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421")
                .into(),
            logs_bloom: Bloom::default(),
            difficulty: U256::zero(),
            number: 0x30598,
            gas_limit: 0x1c9_c380,
            gas_used: 0,
            timestamp: 0x64c4_0d54,
            extra_data: Bytes::from_static(&hex!(
                "d883010c01846765746888676f312e32302e35856c696e7578"
            )),
            mix_hash: hex!("70ccadc40b16e2094954b1064749cc6fbac783c1712f1b271a8aac3eda2f2325")
                .into(),
            nonce: 0,
            base_fee_per_gas: Some(7),
            withdrawals_root: Some(
                hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
            ),
            parent_beacon_block_root: None,
            blob_gas_used: Some(0),
            excess_blob_gas: Some(0x0160_0000),
        };

        let header = Header::decode(&data).unwrap();
        assert_eq!(header.blob_gas_used, expected.blob_gas_used);
        assert_eq!(header, expected);

        let expected_hash =
            H256(hex!("539c9ea0a3ca49808799d3964b8b6607037227de26bc51073c6926963127087b"));
        let actual_hash = header.compute_hash::<DefaultCrypto>();
        assert_eq!(actual_hash, expected_hash);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_decode_header_from_json() {
        // Block from devnet-7
        let json = r#"
        {
            "parentHash": "0x80ba4afd82b6b93f091c6a8a6209455b6de13c31ebbf4de2c6a776be79b8d949",
            "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
            "miner": "0x1f9090aae28b8a3dceadf281b0f12828e676c326",
            "stateRoot": "0x75d281c8d343c6dce5f6ecb7a970e0ebb1a4c180fd2a941bfc64c9e0df14e129",
            "transactionsRoot": "0x4a07175e44a34f29d9fac4b1928e720519e9cd728f805ee5775fc371ebd5f1d3",
            "receiptsRoot": "0xd4400c7d7de1b5e91ed88349222639ca6fca8546b803b48b49e355387b4dffdb",
            "withdrawalsRoot": "0x7dab7799b64bd45d1c8681f188b13c5e71bbf4d3a7faf2c4fb175ea121e486a0",
            "logsBloom": "0x122b4332c5f0b90df580290c840032f421800c3e62944b2688090c300e0234878c05d088032ea2a4008027320800030682958c31ba80bf93c005046b292a304d8e8e2529633b2ea86c8546cc3c8280b2d9391bdbb8cc0810a154d16299b180c0fa2348546293b12b74a0d3014095edbda51062a944089ee2cfd108d31a28846d9674a2490061232081c4854e030014ce1292200519aa815977c8404001b11c788a280248180028c093235a94b90fa5889e18845a54468c104cc054d3cd0e926b182545766b1607e2730107da4049c7260cc04e8a555b0111742526422c03a32a6157e00d124632185214302c6b1448dae1809179026f105e030f4a3414811ca1",
            "difficulty": "0x0",
            "number": "0x11b2ad4",
            "gasLimit": "0x1c9c380",
            "gasUsed": "0xc80dd3",
            "timestamp": "0x65511aeb",
            "mixHash": "0x0e8d993ca6766486af47fff56639f7b6d343ef28257295338747faaffb0f71e8",
            "nonce": "0x0000000000000000",
            "baseFeePerGas": "0x7bc79b7ca",
            "extraData": "0x7273796e632d6275696c6465722e78797a",
            "hash": "0x6c2b441fe64b6ab2d4f71142cdce55e5dae57bd45e7f504e4639e2a443ffc15e",
            "size": "0x1e2a4",
            "totalDifficulty": "0xc70d815d562d3cfa955",
            "uncles": []
        }"#;
        let expected = Header {
            parent_hash: hex!("80ba4afd82b6b93f091c6a8a6209455b6de13c31ebbf4de2c6a776be79b8d949")
                .into(),
            ommers_hash: EMPTY_OMMER_ROOT_HASH,
            beneficiary: hex!("1f9090aae28b8a3dceadf281b0f12828e676c326").into(),
            state_root: hex!("75d281c8d343c6dce5f6ecb7a970e0ebb1a4c180fd2a941bfc64c9e0df14e129")
                .into(),
            transactions_root: hex!(
                "4a07175e44a34f29d9fac4b1928e720519e9cd728f805ee5775fc371ebd5f1d3"
            )
            .into(),
            receipts_root: hex!("d4400c7d7de1b5e91ed88349222639ca6fca8546b803b48b49e355387b4dffdb")
                .into(),
            logs_bloom: hex!("122b4332c5f0b90df580290c840032f421800c3e62944b2688090c300e0234878c05d088032ea2a4008027320800030682958c31ba80bf93c005046b292a304d8e8e2529633b2ea86c8546cc3c8280b2d9391bdbb8cc0810a154d16299b180c0fa2348546293b12b74a0d3014095edbda51062a944089ee2cfd108d31a28846d9674a2490061232081c4854e030014ce1292200519aa815977c8404001b11c788a280248180028c093235a94b90fa5889e18845a54468c104cc054d3cd0e926b182545766b1607e2730107da4049c7260cc04e8a555b0111742526422c03a32a6157e00d124632185214302c6b1448dae1809179026f105e030f4a3414811ca1").into(),
            difficulty: U256::zero(),
            number: 0x011b_2ad4,
            gas_limit: 0x1c9_c380,
            gas_used: 0x00c8_0dd3,
            timestamp: 0x6551_1aeb,
            extra_data: Bytes::from_static(&hex!(
                "7273796e632d6275696c6465722e78797a"
            )),
            mix_hash: hex!("0e8d993ca6766486af47fff56639f7b6d343ef28257295338747faaffb0f71e8")
                .into(),
            nonce: 0,
            base_fee_per_gas: Some(0x7_bc79_b7ca_u64),
            withdrawals_root: Some(
                hex!("7dab7799b64bd45d1c8681f188b13c5e71bbf4d3a7faf2c4fb175ea121e486a0").into(),
            ),
            parent_beacon_block_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
        };

        let decoded = serde_json::from_str::<Header>(json).unwrap();
        assert_eq!(decoded, expected);

        let expected_hash =
            H256(hex!("6c2b441fe64b6ab2d4f71142cdce55e5dae57bd45e7f504e4639e2a443ffc15e"));
        let actual_hash = expected.compute_hash::<DefaultCrypto>();
        assert_eq!(actual_hash, expected_hash);
    }
}
