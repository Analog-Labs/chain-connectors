//! BIP32 implementation.
use crate::bip39::Mnemonic;
use crate::bip44::ChildNumber;
use crate::{Algorithm, PublicKey, SecretKey};
use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::Sha512;

impl Algorithm {
    /// If the algorithm supports BIP32. ECDSA and Ed25519 do, but schnorrkel
    /// uses it's own hierarchical key derivation algorithm.
    fn supports_bip32(self) -> bool {
        !matches!(self, Algorithm::Sr25519)
    }

    /// If the algorithm supports soft key derivations. ECDSA and schnorrkel
    /// do, but ed25519 does not.
    fn supports_non_hardened_derivation(self) -> bool {
        !matches!(self, Algorithm::Ed25519)
    }

    /// BIP32 defines a retry procedure for secp256k1.
    fn uses_bip32_retry(self) -> bool {
        matches!(
            self,
            Algorithm::EcdsaSecp256k1 | Algorithm::EcdsaRecoverableSecp256k1
        )
    }

    /// SLIP0010 defines a retry procedure for secp256r1.
    fn uses_slip10_retry(self) -> bool {
        matches!(self, Algorithm::EcdsaSecp256r1)
    }
}

impl SecretKey {
    fn tweak_add(&self, secret_key: &SecretKey) -> Result<Option<Self>> {
        use ecdsa::elliptic_curve::NonZeroScalar;
        match (self, secret_key) {
            (SecretKey::EcdsaSecp256k1(secret), SecretKey::EcdsaSecp256k1(secret2))
            | (
                SecretKey::EcdsaRecoverableSecp256k1(secret),
                SecretKey::EcdsaRecoverableSecp256k1(secret2),
            ) => {
                let scalar = secret.as_nonzero_scalar().as_ref();
                let tweak = secret2.as_nonzero_scalar().as_ref();
                let scalar: Option<NonZeroScalar<_>> =
                    Option::from(NonZeroScalar::new(scalar + tweak));
                let signing_key = match scalar {
                    Some(scalar) => ecdsa::SigningKey::from(scalar),
                    None => return Ok(None),
                };
                Ok(Some(if self.algorithm().is_recoverable() {
                    SecretKey::EcdsaRecoverableSecp256k1(signing_key)
                } else {
                    SecretKey::EcdsaSecp256k1(signing_key)
                }))
            }
            (SecretKey::EcdsaSecp256r1(secret), SecretKey::EcdsaSecp256r1(secret2)) => {
                let scalar = secret.as_nonzero_scalar().as_ref();
                let tweak = secret2.as_nonzero_scalar().as_ref();
                let scalar: Option<NonZeroScalar<_>> =
                    Option::from(NonZeroScalar::new(scalar + tweak));
                match scalar {
                    Some(scalar) => Ok(Some(SecretKey::EcdsaSecp256r1(ecdsa::SigningKey::from(
                        scalar,
                    )))),
                    None => Ok(None),
                }
            }
            _ => anyhow::bail!("unsupported key type"),
        }
    }
}

impl PublicKey {
    fn tweak_add(&self, tweak: [u8; 32]) -> Result<Option<Self>> {
        match self {
            PublicKey::EcdsaSecp256k1(public) | PublicKey::EcdsaRecoverableSecp256k1(public) => {
                Ok((|| {
                    let parent_key = k256::ProjectivePoint::from(public.as_affine());
                    let tweak = k256::NonZeroScalar::try_from(&tweak[..]).ok()?;
                    let mut tweak_point = k256::ProjectivePoint::GENERATOR * tweak.as_ref();
                    tweak_point += parent_key;
                    let public = ecdsa::VerifyingKey::from_affine(tweak_point.to_affine()).ok()?;
                    Some(if self.algorithm().is_recoverable() {
                        PublicKey::EcdsaRecoverableSecp256k1(public)
                    } else {
                        PublicKey::EcdsaSecp256k1(public)
                    })
                })())
            }
            PublicKey::EcdsaSecp256r1(public) => Ok((|| {
                let parent_key = p256::ProjectivePoint::from(public.as_affine());
                let tweak = p256::NonZeroScalar::try_from(&tweak[..]).ok()?;
                let mut tweak_point = p256::ProjectivePoint::GENERATOR * tweak.as_ref();
                tweak_point += parent_key;
                let public = ecdsa::VerifyingKey::from_affine(tweak_point.to_affine()).ok()?;
                Some(PublicKey::EcdsaSecp256r1(public))
            })()),
            _ => anyhow::bail!("unsupported key type"),
        }
    }
}

/// Secret key and chain code used for hierarchical key derivation.
#[derive(Clone)]
pub struct DerivedSecretKey {
    secret_key: SecretKey,
    chain_code: [u8; 32],
}

impl DerivedSecretKey {
    /// Derives a master key from a mnemonic.
    pub fn new(mnemonic: &Mnemonic, password: &str, algorithm: Algorithm) -> Result<Self> {
        if algorithm == Algorithm::Sr25519 {
            Self::substrate(mnemonic, password, algorithm)
        } else {
            Self::bip39(mnemonic, password, algorithm)
        }
    }

    /// Derives a master key and chain code from a mnemonic using BIP39.
    pub fn bip39(mnemonic: &Mnemonic, password: &str, algorithm: Algorithm) -> Result<Self> {
        let seed = mnemonic.to_seed(password);
        Self::bip32_master_key(&seed[..], algorithm)
    }

    /// Derives a BIP32 master key. See SLIP0010 for extension to secp256r1 and ed25519 curves.
    fn bip32_master_key(seed: &[u8], algorithm: Algorithm) -> Result<Self> {
        let curve_name = match algorithm {
            Algorithm::EcdsaSecp256k1 => &b"Bitcoin seed"[..],
            Algorithm::EcdsaRecoverableSecp256k1 => &b"Bitcoin seed"[..],
            Algorithm::EcdsaSecp256r1 => &b"Nist256p1 seed"[..],
            Algorithm::Ed25519 => &b"ed25519 seed"[..],
            Algorithm::Sr25519 => anyhow::bail!("sr25519 does not support bip32 derivation"),
        };
        let mut retry: Option<[u8; 64]> = None;
        loop {
            let mut hmac: Hmac<Sha512> = Hmac::new_from_slice(curve_name)?;
            if let Some(retry) = &retry {
                hmac.update(retry);
            } else {
                hmac.update(seed);
            }
            let result = hmac.finalize().into_bytes();
            let (secret_key, chain_code) = result.split_at(32);
            let secret_key = if let Ok(secret_key) = SecretKey::from_bytes(algorithm, secret_key) {
                secret_key
            } else if algorithm.uses_slip10_retry() {
                retry = Some(result.into());
                continue;
            } else {
                anyhow::bail!("failed to derive a valid secret key");
            };
            return Ok(Self {
                secret_key,
                chain_code: chain_code.try_into()?,
            });
        }
    }

    /// Derives a master key and chain code from a mnemonic. This avoids the complex BIP39
    /// seed generation algorithm which was intended to support brain wallets. Instead it
    /// uses pbkdf2 using the entropy as the key and password as the salt.
    pub fn substrate(mnemonic: &Mnemonic, password: &str, algorithm: Algorithm) -> Result<Self> {
        let (entropy, len) = mnemonic.to_entropy_array();
        let seed = substrate_bip39::seed_from_entropy(&entropy[..len], password)
            .map_err(|_| anyhow::anyhow!("invalid entropy"))?;
        anyhow::ensure!(matches!(algorithm, Algorithm::Ed25519 | Algorithm::Sr25519));
        let (secret_key, chain_code) = seed.split_at(32);
        Ok(Self {
            secret_key: SecretKey::from_bytes(algorithm, secret_key)?,
            chain_code: chain_code.try_into()?,
        })
    }

    /// The secret key used to sign messages.
    pub fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }

    /// The chain code used to derive child keys.
    pub fn chain_code(&self) -> &[u8; 32] {
        &self.chain_code
    }

    /// Returns the derived public key used for verifying signatures.
    pub fn public_key(&self) -> DerivedPublicKey {
        DerivedPublicKey::new(self.secret_key.public_key(), self.chain_code)
    }

    /// Derive a child key using BIP32. See SLIP0010 for extension to secp256r1 and ed25519
    /// curves.
    fn bip32_derive(&self, child: ChildNumber) -> Result<Self> {
        let algorithm = self.secret_key.algorithm();
        anyhow::ensure!(algorithm.supports_bip32(), "doesn't support bip32");
        let mut retry: Option<[u8; 32]> = None;
        loop {
            let mut hmac: Hmac<Sha512> = Hmac::new_from_slice(&self.chain_code[..])?;
            if let Some(retry) = &retry {
                hmac.update(&[1]);
                hmac.update(retry);
            } else if child.is_hardened() {
                hmac.update(&[0]);
                hmac.update(&self.secret_key.to_bytes()[..]);
            } else {
                anyhow::ensure!(
                    algorithm.supports_non_hardened_derivation(),
                    "doesn't support soft derivation"
                );
                hmac.update(&self.secret_key.public_key().to_bytes()[..]);
            }
            hmac.update(&child.to_bytes());

            let result = hmac.finalize().into_bytes();
            let (secret_key, chain_code) = result.split_at(32);
            let chain_code: [u8; 32] = chain_code.try_into()?;
            retry = Some(chain_code);

            let mut secret_key =
                if let Ok(secret_key) = SecretKey::from_bytes(algorithm, secret_key) {
                    secret_key
                } else if algorithm.uses_slip10_retry() {
                    continue;
                } else if algorithm.uses_bip32_retry() {
                    return self.bip32_derive(child + 1);
                } else {
                    anyhow::bail!("failed to derive a valid secret key");
                };

            if algorithm.supports_non_hardened_derivation() {
                if let Some(tweaked_secret_key) = secret_key.tweak_add(&self.secret_key)? {
                    secret_key = tweaked_secret_key;
                } else if algorithm.uses_slip10_retry() {
                    continue;
                } else if algorithm.uses_bip32_retry() {
                    return self.bip32_derive(child + 1);
                } else {
                    anyhow::bail!("invalid tweak");
                }
            }

            return Ok(Self {
                secret_key,
                chain_code,
            });
        }
    }

    /// Derives a child secret key.
    pub fn derive(&self, child: ChildNumber) -> Result<Self> {
        match &self.secret_key {
            SecretKey::Sr25519(secret, _) => {
                use schnorrkel::derive::Derivation;
                let chain_code = schnorrkel::derive::ChainCode(child.to_substrate_chain_code());
                let (secret, minisecret) = if child.is_hardened() {
                    let (minisecret, _) = secret.hard_derive_mini_secret_key(Some(chain_code), b"");
                    let secret =
                        minisecret.expand_to_keypair(schnorrkel::MiniSecretKey::ED25519_MODE);
                    (secret, Some(minisecret))
                } else {
                    let (secret, _) = secret.derived_key_simple(chain_code, b"");
                    (secret, None)
                };
                Ok(Self {
                    secret_key: SecretKey::Sr25519(secret, minisecret),
                    chain_code: chain_code.0,
                })
            }
            _ => self.bip32_derive(child),
        }
    }
}

/// Public key and chain code used for hierarchical key derivation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DerivedPublicKey {
    public_key: PublicKey,
    chain_code: [u8; 32],
}

impl DerivedPublicKey {
    /// Constructs a derived public key from a public key and a chain code.
    pub fn new(public_key: PublicKey, chain_code: [u8; 32]) -> Self {
        Self {
            public_key,
            chain_code,
        }
    }

    /// The public key used to verify messages.
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// The chain code used to derive child keys.
    pub fn chain_code(&self) -> &[u8; 32] {
        &self.chain_code
    }

    /// Derive a child key using BIP32. See SLIP0010 for extension to secp256r1 and ed25519
    /// curves.
    fn bip32_derive(&self, child: ChildNumber) -> Result<Self> {
        anyhow::ensure!(child.is_normal(), "can't derive a hardened public key");
        let algorithm = self.public_key.algorithm();
        anyhow::ensure!(algorithm.supports_bip32(), "doesn't support bip32");
        anyhow::ensure!(
            algorithm.supports_non_hardened_derivation(),
            "doesn't support soft derivation"
        );
        let mut retry: Option<[u8; 32]> = None;
        loop {
            let mut hmac: Hmac<Sha512> = Hmac::new_from_slice(&self.chain_code[..])?;
            if let Some(retry) = &retry {
                hmac.update(&[1]);
                hmac.update(retry);
            } else {
                hmac.update(&self.public_key.to_bytes()[..]);
            }
            hmac.update(&child.to_bytes());
            let result = hmac.finalize().into_bytes();
            let (public_key, chain_code) = result.split_at(32);
            let public_key: [u8; 32] = public_key.try_into()?;
            let chain_code: [u8; 32] = chain_code.try_into()?;

            let public_key = if let Some(public_key) = self.public_key.tweak_add(public_key)? {
                public_key
            } else if algorithm.uses_slip10_retry() {
                retry = Some(chain_code);
                continue;
            } else if algorithm.uses_bip32_retry() {
                return self.bip32_derive(child + 1);
            } else {
                anyhow::bail!("failed to derive a valid public key");
            };

            return Ok(Self {
                public_key,
                chain_code,
            });
        }
    }

    /// Derives a child public key.
    pub fn derive(&self, child: ChildNumber) -> Result<Self> {
        anyhow::ensure!(child.is_normal(), "can't derive a hardened public key");
        match &self.public_key {
            PublicKey::Sr25519(public) => {
                use schnorrkel::derive::Derivation;
                let chain_code = schnorrkel::derive::ChainCode(child.to_substrate_chain_code());
                let (public, _) = public.derived_key_simple(chain_code, b"");
                Ok(Self {
                    public_key: PublicKey::Sr25519(public),
                    chain_code: chain_code.0,
                })
            }
            _ => self.bip32_derive(child),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip39::Language;

    struct DerivedKey {
        secret: DerivedSecretKey,
        public: DerivedPublicKey,
    }

    impl DerivedKey {
        fn new(algorithm: Algorithm, mnemonic: &str) -> Result<Self> {
            let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)?;
            let secret = DerivedSecretKey::new(&mnemonic, "", algorithm)?;
            let public = secret.public_key();
            Ok(Self { secret, public })
        }

        fn bip32_master_key(algorithm: Algorithm, seed: &str) -> Result<Self> {
            let secret = DerivedSecretKey::bip32_master_key(&hex::decode(seed)?[..], algorithm)?;
            let public = secret.public_key();
            Ok(Self { secret, public })
        }

        fn bip32_derive(&self, child: ChildNumber) -> Result<Self> {
            let secret = self.secret.derive(child)?;
            let public = secret.public_key();
            if child.is_normal() {
                let public2 = self.public.derive(child)?;
                assert_eq!(public, public2);
            }
            Ok(Self { secret, public })
        }

        fn assert(&self, chain_code: &str, private: &str, public: &str) {
            assert_eq!(
                hex::encode(self.secret.chain_code()),
                chain_code,
                "secret chain code"
            );
            assert_eq!(
                // in sr25519 soft derivations the nonce part of the secret key is computed
                // randomly so only the first 32 bytes are deterministic. all other keys are
                // 32 bytes long.
                hex::encode(&self.secret.secret_key().to_bytes()[..32]),
                private,
                "secret key"
            );
            assert_eq!(
                hex::encode(self.public.chain_code()),
                chain_code,
                "secret chain code"
            );
            assert_eq!(
                hex::encode(self.public.public_key().to_bytes()),
                public,
                "public key"
            );
        }

        fn assert_bip32(&self, xprv: &str, xpub: &str) {
            assert_eq!(&xprv[..4], "xprv");
            assert_eq!(&xpub[..4], "xpub");
            let xprv = bs58::decode(xprv)
                .with_alphabet(bs58::Alphabet::BITCOIN)
                .into_vec()
                .unwrap();
            let xpub = bs58::decode(xpub)
                .with_alphabet(bs58::Alphabet::BITCOIN)
                .into_vec()
                .unwrap();
            let chain_code1 = &xprv[13..45];
            let chain_code2 = &xpub[13..45];
            assert_eq!(chain_code1, chain_code2);
            let chain_code = hex::encode(chain_code1);
            let private = hex::encode(&xprv[46..78]);
            let public = hex::encode(&xpub[45..78]);
            self.assert(&chain_code, &private, &public);
        }
    }

    #[test]
    fn bip32_derive_secp256k1_1() -> Result<()> {
        // test vectors from SLIP0010
        for algorithm in [
            Algorithm::EcdsaSecp256k1,
            Algorithm::EcdsaRecoverableSecp256k1,
        ] {
            let key = DerivedKey::bip32_master_key(algorithm, "000102030405060708090a0b0c0d0e0f")?;
            key.assert(
                "873dff81c02f525623fd1fe5167eac3a55a049de3d314bb42ee227ffed37d508",
                "e8f32e723decf4051aefac8e2c93c9c5b214313817cdb01a1494b917c8436b35",
                "0339a36013301597daef41fbe593a02cc513d0b55527ec2df1050e2e8ff49c85c2",
            );
            let key = key.bip32_derive(ChildNumber::hardened_from_u32(0))?;
            key.assert(
                "47fdacbd0f1097043b78c63c20c34ef4ed9a111d980047ad16282c7ae6236141",
                "edb2e14f9ee77d26dd93b4ecede8d16ed408ce149b6cd80b0715a2d911a0afea",
                "035a784662a4a20a65bf6aab9ae98a6c068a81c52e4b032c0fb5400c706cfccc56",
            );
            let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(1))?;
            key.assert(
                "2a7857631386ba23dacac34180dd1983734e444fdbf774041578e9b6adb37c19",
                "3c6cb8d0f6a264c91ea8b5030fadaa8e538b020f0a387421a12de9319dc93368",
                "03501e454bf00751f24b1b489aa925215d66af2234e3891c3b21a52bedb3cd711c",
            );
            let key = key.bip32_derive(ChildNumber::hardened_from_u32(2))?;
            key.assert(
                "04466b9cc8e161e966409ca52986c584f07e9dc81f735db683c3ff6ec7b1503f",
                "cbce0d719ecf7431d88e6a89fa1483e02e35092af60c042b1df2ff59fa424dca",
                "0357bfe1e341d01c69fe5654309956cbea516822fba8a601743a012a7896ee8dc2",
            );
            let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(2))?;
            key.assert(
                "cfb71883f01676f587d023cc53a35bc7f88f724b1f8c2892ac1275ac822a3edd",
                "0f479245fb19a38a1954c5c7c0ebab2f9bdfd96a17563ef28a6a4b1a2a764ef4",
                "02e8445082a72f29b75ca48748a914df60622a609cacfce8ed0e35804560741d29",
            );
            let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(1000000000))?;
            key.assert(
                "c783e67b921d2beb8f6b389cc646d7263b4145701dadd2161548a8b078e65e9e",
                "471b76e389e528d6de6d816857e012c5455051cad6660850e58372a6c3e6e7c8",
                "022a471424da5e657499d1ff51cb43c47481a03b1e77f951fe64cec9f5a48f7011",
            );
        }
        Ok(())
    }

    #[test]
    fn bip32_derive_secp256r1_1() -> Result<()> {
        // test vectors from SLIP0010
        let key = DerivedKey::bip32_master_key(
            Algorithm::EcdsaSecp256r1,
            "000102030405060708090a0b0c0d0e0f",
        )?;
        key.assert(
            "beeb672fe4621673f722f38529c07392fecaa61015c80c34f29ce8b41b3cb6ea",
            "612091aaa12e22dd2abef664f8a01a82cae99ad7441b7ef8110424915c268bc2",
            "0266874dc6ade47b3ecd096745ca09bcd29638dd52c2c12117b11ed3e458cfa9e8",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(0))?;
        key.assert(
            "3460cea53e6a6bb5fb391eeef3237ffd8724bf0a40e94943c98b83825342ee11",
            "6939694369114c67917a182c59ddb8cafc3004e63ca5d3b84403ba8613debc0c",
            "0384610f5ecffe8fda089363a41f56a5c7ffc1d81b59a612d0d649b2d22355590c",
        );
        let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(1))?;
        key.assert(
            "4187afff1aafa8445010097fb99d23aee9f599450c7bd140b6826ac22ba21d0c",
            "284e9d38d07d21e4e281b645089a94f4cf5a5a81369acf151a1c3a57f18b2129",
            "03526c63f8d0b4bbbf9c80df553fe66742df4676b241dabefdef67733e070f6844",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(2))?;
        key.assert(
            "98c7514f562e64e74170cc3cf304ee1ce54d6b6da4f880f313e8204c2a185318",
            "694596e8a54f252c960eb771a3c41e7e32496d03b954aeb90f61635b8e092aa7",
            "0359cf160040778a4b14c5f4d7b76e327ccc8c4a6086dd9451b7482b5a4972dda0",
        );
        let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(2))?;
        key.assert(
            "ba96f776a5c3907d7fd48bde5620ee374d4acfd540378476019eab70790c63a0",
            "5996c37fd3dd2679039b23ed6f70b506c6b56b3cb5e424681fb0fa64caf82aaa",
            "029f871f4cb9e1c97f9f4de9ccd0d4a2f2a171110c61178f84430062230833ff20",
        );
        let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(1000000000))?;
        key.assert(
            "b9b7b82d326bb9cb5b5b121066feea4eb93d5241103c9e7a18aad40f1dde8059",
            "21c4f269ef0a5fd1badf47eeacebeeaa3de22eb8e5b0adcd0f27dd99d34d0119",
            "02216cd26d31147f72427a453c443ed2cde8a1e53c9cc44e5ddf739725413fe3f4",
        );
        Ok(())
    }

    #[test]
    fn bip32_derive_ed25519_1() -> Result<()> {
        // test vectors from SLIP0010
        let key =
            DerivedKey::bip32_master_key(Algorithm::Ed25519, "000102030405060708090a0b0c0d0e0f")?;
        key.assert(
            "90046a93de5380a72b5e45010748567d5ea02bbf6522f979e05c0d8d8ca9fffb",
            "2b4be7f19ee27bbf30c667b642d5f4aa69fd169872f8fc3059c08ebae2eb19e7",
            "a4b2856bfec510abab89753fac1ac0e1112364e7d250545963f135f2a33188ed",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(0))?;
        key.assert(
            "8b59aa11380b624e81507a27fedda59fea6d0b779a778918a2fd3590e16e9c69",
            "68e0fe46dfb67e368c75379acec591dad19df3cde26e63b93a8e704f1dade7a3",
            "8c8a13df77a28f3445213a0f432fde644acaa215fc72dcdf300d5efaa85d350c",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(1))?;
        key.assert(
            "a320425f77d1b5c2505a6b1b27382b37368ee640e3557c315416801243552f14",
            "b1d0bad404bf35da785a64ca1ac54b2617211d2777696fbffaf208f746ae84f2",
            "1932a5270f335bed617d5b935c80aedb1a35bd9fc1e31acafd5372c30f5c1187",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(2))?;
        key.assert(
            "2e69929e00b5ab250f49c3fb1c12f252de4fed2c1db88387094a0f8c4c9ccd6c",
            "92a5b23c0b8a99e37d07df3fb9966917f5d06e02ddbd909c7e184371463e9fc9",
            "ae98736566d30ed0e9d2f4486a64bc95740d89c7db33f52121f8ea8f76ff0fc1",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(2))?;
        key.assert(
            "8f6d87f93d750e0efccda017d662a1b31a266e4a6f5993b15f5c1f07f74dd5cc",
            "30d1dc7e5fc04c31219ab25a27ae00b50f6fd66622f6e9c913253d6511d1e662",
            "8abae2d66361c879b900d204ad2cc4984fa2aa344dd7ddc46007329ac76c429c",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(1000000000))?;
        key.assert(
            "68789923a0cac2cd5a29172a475fe9e0fb14cd6adb5ad98a3fa70333e7afa230",
            "8f94d394a8e8fd6b1bc2f3f49f5c47e385281d5c17e65324b0f62483e37e8793",
            "3c24da049451555d51a7014a37337aa4e12d41e485abccfa46b47dfb2af54b7a",
        );
        Ok(())
    }

    #[test]
    fn bip32_derive_sr25519_1() -> Result<()> {
        // test vectors generated with subkey
        let key = DerivedKey::new(
            Algorithm::Sr25519,
            "clip pulse sausage soap mom era engine trip hammer leg genre figure",
        )?;
        key.assert(
            "7c7bf55246e1fa6905060b061524fc67a827862f7f296d703fdbb8b9f331415c",
            "2c7e70992e4d2490ede2d341669ac3f8ae526bd6e58ba520a5066485cc7a0007",
            "849570b7449e65a5adc6baf48e44720a55aabf567d37e0313b9ed8c93c80736e",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(0))?;
        key.assert(
            "0000000000000000000000000000000000000000000000000000000000000000",
            "4652b03d0cd6e166849641bd08e70334d75a4ec29c7b8320ab269785724a5674",
            "f6bb71fdfdbbc8f20cce86c1e46883d17afe929271768ce5e7c075e420e3e120",
        );
        let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(1))?;
        key.assert(
            "0100000000000000000000000000000000000000000000000000000000000000",
            "7b0c8366993f9c22cf34e099f0b8cfa70fc43dca186d3590651c00f67daa8504",
            "baf3a16c76aaff29d82cf0e5cda6e2af6c780f34a5d2af787591941ec7b5530e",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(2))?;
        key.assert(
            "0200000000000000000000000000000000000000000000000000000000000000",
            "6e8562fe4330e3d106ff914da8d1832255f18da375831eddd61b5542bb466708",
            "38c099de5c3faa4829605532b8a6a2d1731ae823dd1dd90549c813042d3cc23b",
        );
        let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(2))?;
        key.assert(
            "0200000000000000000000000000000000000000000000000000000000000000",
            "5bdcd9d165e7e7f2c27c3e98edd10cc5f50a07dbd69b0e49171f531dae11890e",
            "da7b3bb8a92351f89c4a996561c8323fd534c9a6a6f713ffce316e0e95169c58",
        );
        let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(1000000000))?;
        key.assert(
            "00ca9a3b00000000000000000000000000000000000000000000000000000000",
            "8800b77abcbe366d2afa4c65e1c47884e5cd0ff81824d7cb10b8a3aa2a044a0f",
            "34f13d95d72cbe597ca298eaa18f27f3e4d75217a8ab1230b243f7ebae29e45b",
        );
        Ok(())
    }

    const SEED2: &str = "fffcf9f6f3f0edeae7e4e1dedbd8d5d2cfccc9c6c3c0bdbab7b4b1aeaba8a5a29f9c999693908d8a8784817e7b7875726f6c696663605d5a5754514e4b484542";

    #[test]
    fn bip32_derive_secp256k1_2() -> Result<()> {
        // test vectors from SLIP0010
        for algorithm in [
            Algorithm::EcdsaSecp256k1,
            Algorithm::EcdsaRecoverableSecp256k1,
        ] {
            let key = DerivedKey::bip32_master_key(algorithm, SEED2)?;
            key.assert(
                "60499f801b896d83179a4374aeb7822aaeaceaa0db1f85ee3e904c4defbd9689",
                "4b03d6fc340455b363f51020ad3ecca4f0850280cf436c70c727923f6db46c3e",
                "03cbcaa9c98c877a26977d00825c956a238e8dddfbd322cce4f74b0b5bd6ace4a7",
            );
            let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(0))?;
            key.assert(
                "f0909affaa7ee7abe5dd4e100598d4dc53cd709d5a5c2cac40e7412f232f7c9c",
                "abe74a98f6c7eabee0428f53798f0ab8aa1bd37873999041703c742f15ac7e1e",
                "02fc9e5af0ac8d9b3cecfe2a888e2117ba3d089d8585886c9c826b6b22a98d12ea",
            );
            let key = key.bip32_derive(ChildNumber::hardened_from_u32(2147483647))?;
            key.assert(
                "be17a268474a6bb9c61e1d720cf6215e2a88c5406c4aee7b38547f585c9a37d9",
                "877c779ad9687164e9c2f4f0f4ff0340814392330693ce95a58fe18fd52e6e93",
                "03c01e7425647bdefa82b12d9bad5e3e6865bee0502694b94ca58b666abc0a5c3b",
            );
            let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(1))?;
            key.assert(
                "f366f48f1ea9f2d1d3fe958c95ca84ea18e4c4ddb9366c336c927eb246fb38cb",
                "704addf544a06e5ee4bea37098463c23613da32020d604506da8c0518e1da4b7",
                "03a7d1d856deb74c508e05031f9895dab54626251b3806e16b4bd12e781a7df5b9",
            );
            let key = key.bip32_derive(ChildNumber::hardened_from_u32(2147483646))?;
            key.assert(
                "637807030d55d01f9a0cb3a7839515d796bd07706386a6eddf06cc29a65a0e29",
                "f1c7c871a54a804afe328b4c83a1c33b8e5ff48f5087273f04efa83b247d6a2d",
                "02d2b36900396c9282fa14628566582f206a5dd0bcc8d5e892611806cafb0301f0",
            );
            let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(2))?;
            key.assert(
                "9452b549be8cea3ecb7a84bec10dcfd94afe4d129ebfd3b3cb58eedf394ed271",
                "bb7d39bdb83ecf58f2fd82b6d918341cbef428661ef01ab97c28a4842125ac23",
                "024d902e1a2fc7a8755ab5b694c575fce742c48d9ff192e63df5193e4c7afe1f9c",
            );
        }
        Ok(())
    }

    #[test]
    fn bip32_derive_secp256r1_2() -> Result<()> {
        // test vectors from SLIP0010
        let key = DerivedKey::bip32_master_key(Algorithm::EcdsaSecp256r1, SEED2)?;
        key.assert(
            "96cd4465a9644e31528eda3592aa35eb39a9527769ce1855beafc1b81055e75d",
            "eaa31c2e46ca2962227cf21d73a7ef0ce8b31c756897521eb6c7b39796633357",
            "02c9e16154474b3ed5b38218bb0463e008f89ee03e62d22fdcc8014beab25b48fa",
        );
        let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(0))?;
        key.assert(
            "84e9c258bb8557a40e0d041115b376dd55eda99c0042ce29e81ebe4efed9b86a",
            "d7d065f63a62624888500cdb4f88b6d59c2927fee9e6d0cdff9cad555884df6e",
            "039b6df4bece7b6c81e2adfeea4bcf5c8c8a6e40ea7ffa3cf6e8494c61a1fc82cc",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(2147483647))?;
        key.assert(
            "f235b2bc5c04606ca9c30027a84f353acf4e4683edbd11f635d0dcc1cd106ea6",
            "96d2ec9316746a75e7793684ed01e3d51194d81a42a3276858a5b7376d4b94b9",
            "02f89c5deb1cae4fedc9905f98ae6cbf6cbab120d8cb85d5bd9a91a72f4c068c76",
        );
        let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(1))?;
        key.assert(
            "7c0b833106235e452eba79d2bdd58d4086e663bc8cc55e9773d2b5eeda313f3b",
            "974f9096ea6873a915910e82b29d7c338542ccde39d2064d1cc228f371542bbc",
            "03abe0ad54c97c1d654c1852dfdc32d6d3e487e75fa16f0fd6304b9ceae4220c64",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(2147483646))?;
        key.assert(
            "5794e616eadaf33413aa309318a26ee0fd5163b70466de7a4512fd4b1a5c9e6a",
            "da29649bbfaff095cd43819eda9a7be74236539a29094cd8336b07ed8d4eff63",
            "03cb8cb067d248691808cd6b5a5a06b48e34ebac4d965cba33e6dc46fe13d9b933",
        );
        let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(2))?;
        key.assert(
            "3bfb29ee8ac4484f09db09c2079b520ea5616df7820f071a20320366fbe226a7",
            "bb0a77ba01cc31d77205d51d08bd313b979a71ef4de9b062f8958297e746bd67",
            "020ee02e18967237cf62672983b253ee62fa4dd431f8243bfeccdf39dbe181387f",
        );
        Ok(())
    }

    #[test]
    fn bip32_derive_ed25519_2() -> Result<()> {
        // test vectors from SLIP0010
        let key = DerivedKey::bip32_master_key(Algorithm::Ed25519, SEED2)?;
        key.assert(
            "ef70a74db9c3a5af931b5fe73ed8e1a53464133654fd55e7a66f8570b8e33c3b",
            "171cb88b1b3c1db25add599712e36245d75bc65a1a5c9e18d76f9f2b1eab4012",
            "8fe9693f8fa62a4305a140b9764c5ee01e455963744fe18204b4fb948249308a",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(0))?;
        key.assert(
            "0b78a3226f915c082bf118f83618a618ab6dec793752624cbeb622acb562862d",
            "1559eb2bbec5790b0c65d8693e4d0875b1747f4970ae8b650486ed7470845635",
            "86fab68dcb57aa196c77c5f264f215a112c22a912c10d123b0d03c3c28ef1037",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(2147483647))?;
        key.assert(
            "138f0b2551bcafeca6ff2aa88ba8ed0ed8de070841f0c4ef0165df8181eaad7f",
            "ea4f5bfe8694d8bb74b7b59404632fd5968b774ed545e810de9c32a4fb4192f4",
            "5ba3b9ac6e90e83effcd25ac4e58a1365a9e35a3d3ae5eb07b9e4d90bcf7506d",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(1))?;
        key.assert(
            "73bd9fff1cfbde33a1b846c27085f711c0fe2d66fd32e139d3ebc28e5a4a6b90",
            "3757c7577170179c7868353ada796c839135b3d30554bbb74a4b1e4a5a58505c",
            "2e66aa57069c86cc18249aecf5cb5a9cebbfd6fadeab056254763874a9352b45",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(2147483646))?;
        key.assert(
            "0902fe8a29f9140480a00ef244bd183e8a13288e4412d8389d140aac1794825a",
            "5837736c89570de861ebc173b1086da4f505d4adb387c6a1b1342d5e4ac9ec72",
            "e33c0f7d81d843c572275f287498e8d408654fdf0d1e065b84e2e6f157aab09b",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(2))?;
        key.assert(
            "5d70af781f3a37b829f0d060924d5e960bdc02e85423494afc0b1a41bbe196d4",
            "551d333177df541ad876a60ea71f00447931c0a9da16f227c11ea080d7391b8d",
            "47150c75db263559a70d5778bf36abbab30fb061ad69f69ece61a72b0cfa4fc0",
        );
        Ok(())
    }

    #[test]
    fn bip32_retry_derive_secp256r1() -> Result<()> {
        // test vectors from SLIP0010
        let seed = "000102030405060708090a0b0c0d0e0f";
        let key = DerivedKey::bip32_master_key(Algorithm::EcdsaSecp256r1, seed)?;
        key.assert(
            "beeb672fe4621673f722f38529c07392fecaa61015c80c34f29ce8b41b3cb6ea",
            "612091aaa12e22dd2abef664f8a01a82cae99ad7441b7ef8110424915c268bc2",
            "0266874dc6ade47b3ecd096745ca09bcd29638dd52c2c12117b11ed3e458cfa9e8",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(28578))?;
        key.assert(
            "e94c8ebe30c2250a14713212f6449b20f3329105ea15b652ca5bdfc68f6c65c2",
            "06f0db126f023755d0b8d86d4591718a5210dd8d024e3e14b6159d63f53aa669",
            "02519b5554a4872e8c9c1c847115363051ec43e93400e030ba3c36b52a3e70a5b7",
        );
        let key = key.bip32_derive(ChildNumber::non_hardened_from_u32(33941))?;
        key.assert(
            "9e87fe95031f14736774cd82f25fd885065cb7c358c1edf813c72af535e83071",
            "092154eed4af83e078ff9b84322015aefe5769e31270f62c3f66c33888335f3a",
            "0235bfee614c0d5b2cae260000bb1d0d84b270099ad790022c1ae0b2e782efe120",
        );
        Ok(())
    }

    #[test]
    fn bip32_retry_seed_secp256r1() -> Result<()> {
        // test vectors from SLIP0010
        let seed = "a7305bc8df8d0951f0cb224c0e95d7707cbdf2c6ce7e8d481fec69c7ff5e9446";
        let key = DerivedKey::bip32_master_key(Algorithm::EcdsaSecp256r1, seed)?;
        key.assert(
            "7762f9729fed06121fd13f326884c82f59aa95c57ac492ce8c9654e60efd130c",
            "3b8c18469a4634517d6d0b65448f8e6c62091b45540a1743c5846be55d47d88f",
            "0383619fadcde31063d8c5cb00dbfe1713f3e6fa169d8541a798752a1c1ca0cb20",
        );
        Ok(())
    }

    #[test]
    fn bip32_derive_secp256k1_3() -> Result<()> {
        // test vector 3 from BIP32
        let seed = "4b381541583be4423346c643850da4b320e46a87ae3d2a4e6da11eba819cd4acba45d239319ac14f863b8d5ab5a0d0c64d2e8a1e7d1457df2e5a3c51c73235be";
        let key = DerivedKey::bip32_master_key(Algorithm::EcdsaSecp256k1, seed)?;
        key.assert_bip32(
            "xprv9s21ZrQH143K25QhxbucbDDuQ4naNntJRi4KUfWT7xo4EKsHt2QJDu7KXp1A3u7Bi1j8ph3EGsZ9Xvz9dGuVrtHHs7pXeTzjuxBrCmmhgC6",
            "xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(0))?;
        key.assert_bip32(
            "xprv9uPDJpEQgRQfDcW7BkF7eTya6RPxXeJCqCJGHuCJ4GiRVLzkTXBAJMu2qaMWPrS7AANYqdq6vcBcBUdJCVVFceUvJFjaPdGZ2y9WACViL4L",
            "xpub68NZiKmJWnxxS6aaHmn81bvJeTESw724CRDs6HbuccFQN9Ku14VQrADWgqbhhTHBaohPX4CjNLf9fq9MYo6oDaPPLPxSb7gwQN3ih19Zm4Y",
        );
        Ok(())
    }

    #[test]
    fn bip32_derive_secp256k1_4() -> Result<()> {
        // test vector 4 from BIP32
        let seed = "3ddd5602285899a946114506157c7997e5444528f3003f6134712147db19b678";
        let key = DerivedKey::bip32_master_key(Algorithm::EcdsaSecp256k1, seed)?;
        key.assert_bip32(
            "xprv9s21ZrQH143K48vGoLGRPxgo2JNkJ3J3fqkirQC2zVdk5Dgd5w14S7fRDyHH4dWNHUgkvsvNDCkvAwcSHNAQwhwgNMgZhLtQC63zxwhQmRv",
            "xpub661MyMwAqRbcGczjuMoRm6dXaLDEhW1u34gKenbeYqAix21mdUKJyuyu5F1rzYGVxyL6tmgBUAEPrEz92mBXjByMRiJdba9wpnN37RLLAXa",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(0))?;
        key.assert_bip32(
            "xprv9vB7xEWwNp9kh1wQRfCCQMnZUEG21LpbR9NPCNN1dwhiZkjjeGRnaALmPXCX7SgjFTiCTT6bXes17boXtjq3xLpcDjzEuGLQBM5ohqkao9G",
            "xpub69AUMk3qDBi3uW1sXgjCmVjJ2G6WQoYSnNHyzkmdCHEhSZ4tBok37xfFEqHd2AddP56Tqp4o56AePAgCjYdvpW2PU2jbUPFKsav5ut6Ch1m",
        );
        let key = key.bip32_derive(ChildNumber::hardened_from_u32(1))?;
        key.assert_bip32(
            "xprv9xJocDuwtYCMNAo3Zw76WENQeAS6WGXQ55RCy7tDJ8oALr4FWkuVoHJeHVAcAqiZLE7Je3vZJHxspZdFHfnBEjHqU5hG1Jaj32dVoS6XLT1",
            "xpub6BJA1jSqiukeaesWfxe6sNK9CCGaujFFSJLomWHprUL9DePQ4JDkM5d88n49sMGJxrhpjazuXYWdMf17C9T5XnxkopaeS7jGk1GyyVziaMt",
        );
        Ok(())
    }
}
