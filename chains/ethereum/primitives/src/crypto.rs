pub use crate::{
    eth_hash::{Address, H256},
    transactions::signature::Signature,
};
#[cfg(feature = "with-crypto")]
use alloc::vec::Vec;
#[cfg(feature = "with-crypto")]
use core::iter::Iterator;
use core::{convert::AsRef, iter::IntoIterator};

/// cryptographic hash function and secp256k1 ECDSA signature recovery implementation
pub trait Crypto {
    type Error;

    fn keccak256_to(data: impl AsRef<[u8]>, output: &mut [u8; 32]);

    fn keccak256(data: impl AsRef<[u8]>) -> H256 {
        let mut hash = [0u8; 32];
        Self::keccak256_to(data, &mut hash);
        hash.into()
    }

    /// Verify and recover a `SECP256k1` ECDSA signature.
    ///
    /// - `signature` is signature passed in RSV format.
    /// - `message_hash` is the keccak256 hash of the message.
    ///
    /// # Errors
    /// Returns `Err` if the signature is bad, otherwise the recovered address.
    fn secp256k1_ecdsa_recover(
        signature: &Signature,
        message_hash: H256,
    ) -> Result<Address, Self::Error>;

    fn trie_root<I, V>(input: I) -> H256
    where
        I: IntoIterator<Item = V>,
        V: AsRef<[u8]>;
}

#[cfg(feature = "with-crypto")]
pub struct DefaultCrypto;

#[cfg(feature = "with-crypto")]
impl DefaultCrypto {
    fn keccak256_to(data: impl AsRef<[u8]>, output: &mut [u8; 32]) {
        use sha3::Digest;
        let mut hasher = sha3::Keccak256::new();
        hasher.update(data);
        hasher.finalize_into(output.into());
    }

    fn keccak256(data: impl AsRef<[u8]>) -> H256 {
        use sha3::Digest;
        let hash: [u8; 32] = sha3::Keccak256::digest(data).into();
        hash.into()
    }

    fn secp256k1_ecdsa_recover(
        signature: &Signature,
        message_hash: H256,
    ) -> Result<Address, libsecp256k1::Error> {
        let mut sig = [0u8; 65];
        signature.to_raw_signature(&mut sig);
        let rid = libsecp256k1::RecoveryId::parse(sig[64])?;
        let sig = libsecp256k1::Signature::parse_overflowing_slice(&sig[0..64])?;
        let msg = libsecp256k1::Message::parse(message_hash.as_fixed_bytes());
        let pubkey = libsecp256k1::recover(&msg, &sig, &rid)?;
        // uncompress the key
        let uncompressed = pubkey.serialize();
        let hash = Self::keccak256(&uncompressed[1..]);
        Ok(Address::from(hash))
    }

    fn trie_root<I, V>(input: I) -> H256
    where
        I: IntoIterator<Item = V>,
        V: AsRef<[u8]>,
    {
        trie_root::trie_root::<KeccakHasher, Hash256RlpTrieStream, _, _, _>(
            input.into_iter().enumerate().map(|(i, v)| (rlp::encode(&i), v)),
            None,
        )
    }
}

/// Concrete `TrieStream` impl for the ethereum trie.
#[cfg(feature = "with-crypto")]
#[derive(Default)]
pub struct Hash256RlpTrieStream {
    stream: rlp::RlpStream,
}

#[cfg(feature = "with-crypto")]
impl trie_root::TrieStream for Hash256RlpTrieStream {
    fn new() -> Self {
        Self { stream: rlp::RlpStream::new() }
    }

    fn append_empty_data(&mut self) {
        self.stream.append_empty_data();
    }

    fn begin_branch(
        &mut self,
        _maybe_key: Option<&[u8]>,
        _maybe_value: Option<trie_root::Value>,
        _has_children: impl Iterator<Item = bool>,
    ) {
        // an item for every possible nibble/suffix
        // + 1 for data
        self.stream.begin_list(17);
    }

    fn append_empty_child(&mut self) {
        self.stream.append_empty_data();
    }

    fn end_branch(&mut self, value: Option<trie_root::Value>) {
        match value {
            Some(value) => match value {
                trie_root::Value::Inline(value) => self.stream.append(&value),
                trie_root::Value::Node(value) => self.stream.append(&value),
            },
            None => self.stream.append_empty_data(),
        };
    }

    fn append_leaf(&mut self, key: &[u8], value: trie_root::Value) {
        self.stream.begin_list(2);
        self.stream.append_iter(hex_prefix_encode(key, true));
        match value {
            trie_root::Value::Inline(value) => self.stream.append(&value),
            trie_root::Value::Node(value) => self.stream.append(&value),
        };
    }

    fn append_extension(&mut self, key: &[u8]) {
        self.stream.begin_list(2);
        self.stream.append_iter(hex_prefix_encode(key, false));
    }

    fn append_substream<H: trie_root::Hasher>(&mut self, other: Self) {
        let out = other.out();
        match out.len() {
            0..=31 => self.stream.append_raw(&out, 1),
            _ => self.stream.append(&H::hash(&out).as_ref()),
        };
    }

    fn out(self) -> Vec<u8> {
        self.stream.out().freeze().into()
    }
}

// Copy from `triehash` crate.
/// Hex-prefix Notation. First nibble has flags: oddness = 2^0 & termination = 2^1.
///
/// The "termination marker" and "leaf-node" specifier are completely equivalent.
///
/// Input values are in range `[0, 0xf]`.
///
/// ```markdown
///  [0,0,1,2,3,4,5]   0x10012345 // 7 > 4
///  [0,1,2,3,4,5]     0x00012345 // 6 > 4
///  [1,2,3,4,5]       0x112345   // 5 > 3
///  [0,0,1,2,3,4]     0x00001234 // 6 > 3
///  [0,1,2,3,4]       0x101234   // 5 > 3
///  [1,2,3,4]         0x001234   // 4 > 3
///  [0,0,1,2,3,4,5,T] 0x30012345 // 7 > 4
///  [0,0,1,2,3,4,T]   0x20001234 // 6 > 4
///  [0,1,2,3,4,5,T]   0x20012345 // 6 > 4
///  [1,2,3,4,5,T]     0x312345   // 5 > 3
///  [1,2,3,4,T]       0x201234   // 4 > 3
/// ```
#[cfg(feature = "with-crypto")]
fn hex_prefix_encode(nibbles: &[u8], leaf: bool) -> impl Iterator<Item = u8> + '_ {
    let inlen = nibbles.len();
    let oddness_factor = inlen % 2;

    let first_byte = {
        #[allow(clippy::cast_possible_truncation)]
        let mut bits = ((inlen as u8 & 1) + (2 * u8::from(leaf))) << 4;
        if oddness_factor == 1 {
            bits += nibbles[0];
        }
        bits
    };
    core::iter::once(first_byte)
        .chain(nibbles[oddness_factor..].chunks(2).map(|ch| ch[0] << 4 | ch[1]))
}

#[cfg(feature = "with-crypto")]
impl Crypto for DefaultCrypto {
    type Error = libsecp256k1::Error;

    fn keccak256_to(data: impl AsRef<[u8]>, output: &mut [u8; 32]) {
        Self::keccak256_to(data, output);
    }

    fn keccak256(data: impl AsRef<[u8]>) -> H256 {
        Self::keccak256(data)
    }

    fn secp256k1_ecdsa_recover(
        signature: &Signature,
        message_hash: H256,
    ) -> Result<Address, Self::Error> {
        Self::secp256k1_ecdsa_recover(signature, message_hash)
    }

    fn trie_root<I, V>(input: I) -> H256
    where
        I: IntoIterator<Item = V>,
        V: AsRef<[u8]>,
    {
        Self::trie_root::<I, V>(input)
    }
}

/// Concrete `Hasher` impl for the Keccak-256 hash
#[cfg(feature = "with-crypto")]
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct KeccakHasher;

#[cfg(feature = "with-crypto")]
impl trie_root::Hasher for KeccakHasher {
    type Out = H256;
    type StdHasher = hash256_std_hasher::Hash256StdHasher;
    const LENGTH: usize = 32;

    fn hash(x: &[u8]) -> Self::Out {
        use sha3::Digest;
        let hash: [u8; 32] = sha3::Keccak256::digest(x).into();
        hash.into()
    }
}

#[cfg(all(test, feature = "with-crypto", feature = "with-rlp"))]
mod tests {
    use super::DefaultCrypto;
    use crate::{
        eth_hash::{Address, H256},
        transactions::signature::Signature,
    };
    use hex_literal::hex;

    #[test]
    fn ecdsa_recover_works() {
        let test_cases: [(Signature, H256, Address); 5] = [
            (
                Signature {
                    v: 0x00.into(),
                    r: hex!("74ce2198225fb75ba25ff998f912ebc7ba8351056b3398a73eb2680cd8a0729a")
                        .into(),
                    s: hex!("426cff41ea4656f1517ebf685bc2841e9156eb5e9119833f822aef5d9ca36491")
                        .into(),
                },
                hex!("2104564ddf4958472ccfa07c340edd45558294f4591a343f91554278eee74689").into(),
                hex!("677de87be1ecc2ba2f4003af7efcdcb406ff4d43").into(),
            ),
            (
                Signature {
                    v: 0x01.into(),
                    r: hex!("7818d886a8ca01a6d80a240d3704090a525bb3440699defde67463d5e7094c2e")
                        .into(),
                    s: hex!("05c537ecebbe16f3203a62ed27d251aecb15e636e816686af7d96fccd1efe628")
                        .into(),
                },
                hex!("9478c96651709feb4e3fea375f921faea701cfb66b5e43bdebde586d1aeb7047").into(),
                hex!("F531c7A28a3492390D4C47dBa6775FA76349DcFF").into(),
            ),
            (
                Signature {
                    v: 0x1b.into(),
                    r: hex!("c58f3fd84bc6cd1633e0b8cba40cd2f6d8c0e4bd25a6c834baca0249666366aa")
                        .into(),
                    s: hex!("7ac31746b8f4542847fd695c93cd90fc0dffee1e0445848d27657d60f0279e31")
                        .into(),
                },
                hex!("f5f18567b0a8dbd2f9c12eecc22545e2150f0683ccb2db2a0b37739dd9cb24e5").into(),
                hex!("2a65aca4d5fc5b5c859090a6c34d164135398226").into(),
            ),
            (
                Signature {
                    v: 0x1c.into(),
                    r: hex!("c8fc04e29b0859a7f265b67af7d4c5c6bc9e3d5a8de4950f89fa71a12a3cf8ae")
                        .into(),
                    s: hex!("7dd15a10f9f2c8d1519a6044d880d04756798fc23923ff94f4823df8dc5b987a")
                        .into(),
                },
                hex!("341467bdde941ac08fc0ced98fbbb0db1d9d393909fda333288843b49525faf0").into(),
                hex!("32be343b94f860124dc4fee278fdcbd38c102d88").into(),
            ),
            (
                Signature {
                    v: 0x1b.into(),
                    r: hex!("67309756a39ca4386f74592044c69742dd0458304bb8418679298f76af6cbf5e")
                        .into(),
                    s: hex!("56d8867966628016388705a5e21ef3ca2d324d948d065c751dc90f2249335b52")
                        .into(),
                },
                hex!("fca4165566a95e9cd47f15583b3b05cee0bd8a469ef5d361e3f40898e73ad1a0").into(),
                hex!("ed059bc543141c8c93031d545079b3da0233b27f").into(),
            ),
        ];

        for (signature, msg_hash, expected_addr) in test_cases {
            let actual_addr = DefaultCrypto::secp256k1_ecdsa_recover(&signature, msg_hash).unwrap();
            assert_eq!(expected_addr, actual_addr);
        }
    }
}
