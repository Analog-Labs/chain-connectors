#[cfg(feature = "with-crypto")]
use crate::rstd::vec::Vec;
pub use crate::{
    eth_hash::{Address, H256},
    transactions::signature::{RecoveryId, Signature},
};
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

pub trait Signer {
    type Error;

    /// Sign a message an arbitrary message.
    ///
    /// # Errors
    /// Returns `Err` if the message can't be signed.
    fn sign<I: AsRef<[u8]>>(
        &self,
        message: I,
        chain_id: Option<u64>,
    ) -> Result<Signature, Self::Error>;

    /// Attempt to sign the given message digest, returning a digital signature
    /// on success, or an error if something went wrong.
    ///
    /// # Errors
    /// Returns `Err` if the message can't be signed.
    fn sign_prehash(&self, prehash: H256, chain_id: Option<u64>) -> Result<Signature, Self::Error>;
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

#[cfg(feature = "with-crypto")]
pub struct Keypair {
    keypair: secp256k1::Keypair,
}

#[cfg(feature = "with-crypto")]
impl Keypair {
    /// Create a new private key from a slice of bytes.
    ///
    /// # Errors
    /// Returns `Err` if the slice is greater than secp256k1 curve order.
    pub fn from_bytes<I: AsRef<[u8]>>(bytes: I) -> Result<Self, secp256k1::Error> {
        Self::from_slice(bytes.as_ref())
    }

    /// Create a new private key from a slice of bytes.
    ///
    /// # Errors
    /// Returns `Err` if the slice is greater than secp256k1 curve order.
    pub fn from_slice(slice: &[u8]) -> Result<Self, secp256k1::Error> {
        let secret = secp256k1::SecretKey::from_slice(slice)?;
        #[cfg(feature = "std")]
        let keypair = secret.keypair(secp256k1::SECP256K1);
        #[cfg(not(feature = "std"))]
        let keypair = secret.keypair(&secp256k1::Secp256k1::new());
        Ok(Self { keypair })
    }

    #[must_use]
    pub fn pubkey(&self) -> [u8; 33] {
        self.keypair.public_key().serialize()
    }

    #[must_use]
    pub fn pubkey_uncompressed(&self) -> [u8; 65] {
        self.keypair.public_key().serialize_uncompressed()
    }

    #[must_use]
    pub fn address(&self) -> Address {
        // uncompress the key
        let uncompressed = self.keypair.public_key().serialize_uncompressed();
        let hash = DefaultCrypto::keccak256(&uncompressed[1..]);
        Address::from(hash)
    }
}

#[cfg(feature = "with-crypto")]
impl Signer for Keypair {
    type Error = secp256k1::Error;

    fn sign<I: AsRef<[u8]>>(
        &self,
        msg: I,
        chain_id: Option<u64>,
    ) -> Result<Signature, Self::Error> {
        self.sign_prehash(DefaultCrypto::keccak256(msg.as_ref()), chain_id)
    }

    /// Sign a pre-hashed message
    fn sign_prehash(&self, prehash: H256, chain_id: Option<u64>) -> Result<Signature, Self::Error> {
        use crate::U256;
        use secp256k1::Message;

        #[cfg(feature = "std")]
        let context = secp256k1::SECP256K1;
        #[cfg(not(feature = "std"))]
        let context = secp256k1::Secp256k1::signing_only();

        let msg = Message::from_digest(prehash.0);
        let (v, r, s) = unsafe {
            // The recovery id is a byte that is either 0, 1, 2 or 3 where the first bit indicates
            // if the y is even or odd, and the second bit indicates if an overflow occured or not.
            // reference: https://github.com/bitcoin-core/secp256k1/blob/v0.4.1/src/ecdsa_impl.h#L280-L285
            let sig = context.sign_ecdsa_recoverable(&msg, &self.keypair.secret_key());
            let (recovery_id, _) = sig.serialize_compact();
            let mut sig = sig.to_standard();
            sig.normalize_s();
            let [r, s] = core::mem::transmute::<[u8; 64], [[u8; 32]; 2]>(sig.serialize_compact());
            (recovery_id.to_i32(), U256::from_big_endian(&r), U256::from_big_endian(&s))
        };
        let v = u8::try_from(v)
            .map_err(|_| secp256k1::Error::InvalidRecoveryId)
            .map(u64::from)?
            & 1;

        // All transaction signatures whose s-value is greater than secp256k1n/2 are invalid.
        // - https://github.com/ethereum/EIPs/blob/master/EIPS/eip-2.md
        // - https://github.com/ethereum/go-ethereum/blob/v1.13.14/crypto/crypto.go#L260-L273
        let secp256k1_half_n = U256::from_big_endian(&secp256k1::constants::CURVE_ORDER) >> 1;
        if s >= secp256k1_half_n {
            return Err(secp256k1::Error::IncorrectSignature);
        }
        let v = chain_id.map_or_else(|| v, |chain_id| RecoveryId::new(v).as_eip155(chain_id));
        Ok(Signature { v: RecoveryId::new(v), r, s })
    }
}

#[cfg(all(test, feature = "with-crypto", feature = "with-rlp"))]
mod tests {
    use super::DefaultCrypto;
    use crate::{
        crypto::{Keypair, Signer},
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

    #[test]
    fn sign_ecdsa_works() {
        let test_cases: [([u8; 32], Address, &[u8], Signature); 2] = [
            (
                hex!("fad9c8855b740a0b7ed4c221dbad0f33a83a49cad6b3fe8d5817ac83d38b6a19"),
                hex!("96216849c49358b10257cb55b28ea603c874b05e").into(),
                hex!("e9808501ec5b05eb8301f6d194645d7d9f679a3b8aa4e7eedad709db14f6d3f44182dead808205398080").as_ref(),
                Signature {
                    r: hex!("e138cf75eb34e837cf7cec412a89f48792e49f5a9c8693df722c7705584d813f")
                        .into(),
                    s: hex!("2a1ff44833e17fd7439b2aff374c6fbe9fb1d4353c84188f6467891dfce409c5")
                        .into(),
                    v: 0xa96.into(),
                },
            ),
            (
                hex!("349593acb529f4bd0cda7ac620fab960130e248fd18e55a08df70d87263cf5af"),
                hex!("2729b52d0214282beb1f37eb147f3ec32ad1da91").into(),
                hex!("e5198256788212349496216849c49358b10257cb55b28ea603c874b05e84deadbeef80018080").as_ref(),
                Signature {
                    r: hex!("70611b6d9c5437004c9b7448c982a5f9e88cf32f949141f57ebd188d763123c0")
                        .into(),
                    s: hex!("017d8c488413794b0b0f0b918f8947e13e4faab39e04ba6901b2db436d92ff41")
                        .into(),
                    v: 37.into(),
                },
            ),
        ];

        for (secret_key, expected_addr, msg, expected_sig) in test_cases {
            let prehash = DefaultCrypto::keccak256(msg);
            let wallet = Keypair::from_bytes(secret_key).unwrap();
            let signature = wallet.sign(msg, expected_sig.v.chain_id()).unwrap();
            assert_eq!(signature, expected_sig);
            assert_eq!(signature, wallet.sign_prehash(prehash, expected_sig.v.chain_id()).unwrap());
            let actual_addr = DefaultCrypto::secp256k1_ecdsa_recover(&signature, prehash).unwrap();
            assert_eq!(expected_addr, actual_addr);
        }
    }
}
