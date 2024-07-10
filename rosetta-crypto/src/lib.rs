//! Implements cryptography needed for various chains.
#![deny(missing_docs)]
#![deny(warnings)]

use anyhow::{Context, Result};
use ecdsa::{
    hazmat::SignPrimitive,
    signature::{hazmat::PrehashSigner, Signer as _, Verifier as _},
    RecoveryId,
};
// use ed25519_dalek::{Signer as _, Verifier as _};
use sha2::Digest;

pub mod address;
pub mod bip32;
pub use bip39;
pub mod bip44;
mod error;

/// Signing algorithm.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Algorithm {
    /// ECDSA with secp256k1.
    EcdsaSecp256k1,
    /// ECDSA with secp256k1 in Ethereum compatible format.
    EcdsaRecoverableSecp256k1,
    /// ECDSA with NIST P-256.
    EcdsaSecp256r1,
    /// Ed25519.
    Ed25519,
    /// Schnorrkel used by substrate/polkadot.
    Sr25519,
}

impl Algorithm {
    /// Returns true if the signer's public key is recoverable from the signature.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::EcdsaRecoverableSecp256k1)
    }
}

/// Secret key used for constructing signatures.
pub enum SecretKey {
    /// ECDSA with secp256k1.
    EcdsaSecp256k1(ecdsa::SigningKey<k256::Secp256k1>),
    /// ECDSA with secp256k1 in Ethereum compatible format.
    EcdsaRecoverableSecp256k1(ecdsa::SigningKey<k256::Secp256k1>),
    /// ECDSA with NIST P-256.
    EcdsaSecp256r1(ecdsa::SigningKey<p256::NistP256>),
    /// Ed25519.
    Ed25519(ed25519_dalek::SigningKey),
    /// Schnorrkel used by substrate/polkadot.
    Sr25519(schnorrkel::Keypair, Option<schnorrkel::MiniSecretKey>),
}

impl Clone for SecretKey {
    fn clone(&self) -> Self {
        #[allow(clippy::unwrap_used)]
        Self::from_bytes(self.algorithm(), &self.to_bytes()).unwrap()
    }
}

impl SecretKey {
    /// Returns the signing algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> Algorithm {
        match self {
            Self::EcdsaSecp256k1(_) => Algorithm::EcdsaSecp256k1,
            Self::EcdsaRecoverableSecp256k1(_) => Algorithm::EcdsaRecoverableSecp256k1,
            Self::EcdsaSecp256r1(_) => Algorithm::EcdsaSecp256r1,
            Self::Ed25519(_) => Algorithm::Ed25519,
            Self::Sr25519(_, _) => Algorithm::Sr25519,
        }
    }

    /// Creates a secret key from a byte sequence for a given signing algorithm.
    ///
    /// # Errors
    /// Will return `Err` if `bytes` has the wrong length
    pub fn from_bytes(algorithm: Algorithm, bytes: &[u8]) -> Result<Self> {
        Ok(match algorithm {
            Algorithm::EcdsaSecp256k1 => {
                Self::EcdsaSecp256k1(ecdsa::SigningKey::from_bytes(bytes.into())?)
            },
            Algorithm::EcdsaRecoverableSecp256k1 => {
                Self::EcdsaRecoverableSecp256k1(ecdsa::SigningKey::from_bytes(bytes.into())?)
            },
            Algorithm::EcdsaSecp256r1 => {
                Self::EcdsaSecp256r1(ecdsa::SigningKey::from_bytes(bytes.into())?)
            },
            Algorithm::Ed25519 => {
                let signing_key = match bytes.len() {
                    ed25519_dalek::KEYPAIR_LENGTH => {
                        let mut keypair = [0u8; ed25519_dalek::KEYPAIR_LENGTH];
                        keypair.copy_from_slice(bytes);
                        ed25519_dalek::SigningKey::from_keypair_bytes(&keypair)?
                    },
                    ed25519_dalek::SECRET_KEY_LENGTH => {
                        let mut secret = ed25519_dalek::SecretKey::default();
                        secret.copy_from_slice(bytes);
                        ed25519_dalek::SigningKey::from_bytes(&secret)
                    },
                    len => {
                        anyhow::bail!(
                            "invalid Ed25519 keypair, expected {} bytes, got {len} bytes",
                            ed25519_dalek::KEYPAIR_LENGTH
                        )
                    },
                };
                // let public = ed25519_dalek::PublicKey::from(&secret);
                // let keypair = ed25519_dalek::Keypair { secret, public };
                Self::Ed25519(signing_key)
            },
            Algorithm::Sr25519 => {
                if bytes.len() == 32 {
                    let minisecret = schnorrkel::MiniSecretKey::from_bytes(bytes)
                        .map_err(|err| anyhow::anyhow!("{}", err))?;
                    let secret =
                        minisecret.expand_to_keypair(schnorrkel::MiniSecretKey::ED25519_MODE);
                    Self::Sr25519(secret, Some(minisecret))
                } else {
                    let secret = schnorrkel::SecretKey::from_bytes(bytes)
                        .map_err(|err| anyhow::anyhow!("{}", err))?;
                    Self::Sr25519(secret.to_keypair(), None)
                }
            },
        })
    }

    /// Returns a byte sequence representing the secret key.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::EcdsaRecoverableSecp256k1(secret) | Self::EcdsaSecp256k1(secret) => {
                secret.to_bytes().to_vec()
            },
            Self::EcdsaSecp256r1(secret) => secret.to_bytes().to_vec(),
            Self::Ed25519(secret) => secret.as_bytes().to_vec(),
            Self::Sr25519(_, Some(minisecret)) => minisecret.as_bytes().to_vec(),
            Self::Sr25519(secret, None) => secret.secret.to_bytes().to_vec(),
        }
    }

    /// Returns the public key used for verifying signatures.
    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        match self {
            Self::EcdsaSecp256k1(secret) => PublicKey::EcdsaSecp256k1(*secret.verifying_key()),
            Self::EcdsaRecoverableSecp256k1(secret) => {
                PublicKey::EcdsaRecoverableSecp256k1(*secret.verifying_key())
            },
            Self::EcdsaSecp256r1(secret) => PublicKey::EcdsaSecp256r1(*secret.verifying_key()),
            Self::Ed25519(secret) => PublicKey::Ed25519(secret.verifying_key()),
            Self::Sr25519(secret, _) => PublicKey::Sr25519(secret.public),
        }
    }

    /// Signs a message and returns it's signature.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn sign(&self, msg: &[u8], context_param: &str) -> Signature {
        match self {
            Self::EcdsaSecp256k1(secret) => Signature::EcdsaSecp256k1(secret.sign(msg)),
            Self::EcdsaRecoverableSecp256k1(_) => {
                let digest = sha2::Sha256::digest(msg);
                #[allow(clippy::expect_used)]
                self.sign_prehashed(&digest).expect("supports prehash; qed")
            },
            Self::EcdsaSecp256r1(secret) => Signature::EcdsaSecp256r1(secret.sign(msg)),
            Self::Ed25519(secret) => Signature::Ed25519(secret.sign(msg)),
            Self::Sr25519(secret, _) => {
                // need a signing context here for substrate
                let context = schnorrkel::signing_context(context_param.as_bytes());
                Signature::Sr25519(secret.sign(context.bytes(msg)))
            },
        }
    }

    /// Signs a prehashed message and returns it's signature.
    ///
    /// # Errors
    ///
    /// Not supported by [`SecretKey::Ed25519`] and [`SecretKey::Sr25519`]
    pub fn sign_prehashed(&self, hash: &[u8]) -> Result<Signature> {
        Ok(match self {
            Self::EcdsaSecp256k1(secret) => Signature::EcdsaSecp256k1(secret.sign_prehash(hash)?),
            Self::EcdsaRecoverableSecp256k1(secret) => {
                let (sig, recid) = secret
                    .as_nonzero_scalar()
                    .try_sign_prehashed_rfc6979::<sha2::Sha256>(hash.into(), b"")?;
                Signature::EcdsaRecoverableSecp256k1(sig, recid.context("no recovery id")?)
            },
            Self::EcdsaSecp256r1(secret) => Signature::EcdsaSecp256r1(secret.sign_prehash(hash)?),
            Self::Ed25519(_) => anyhow::bail!("unimplemented"),
            Self::Sr25519(_, _) => {
                anyhow::bail!("unsupported")
            },
        })
    }
}

/// Public key used for verifying signatures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicKey {
    /// ECDSA with secp256k1.
    EcdsaSecp256k1(ecdsa::VerifyingKey<k256::Secp256k1>),
    /// ECDSA with secp256k1 in Ethereum compatible format.
    EcdsaRecoverableSecp256k1(ecdsa::VerifyingKey<k256::Secp256k1>),
    /// ECDSA with NIST P-256.
    EcdsaSecp256r1(ecdsa::VerifyingKey<p256::NistP256>),
    /// Ed25519.
    Ed25519(ed25519_dalek::VerifyingKey),
    /// Schnorrkel used by substrate/polkadot.
    Sr25519(schnorrkel::PublicKey),
}

impl PublicKey {
    /// Returns the signing algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> Algorithm {
        match self {
            Self::EcdsaSecp256k1(_) => Algorithm::EcdsaSecp256k1,
            Self::EcdsaRecoverableSecp256k1(_) => Algorithm::EcdsaRecoverableSecp256k1,
            Self::EcdsaSecp256r1(_) => Algorithm::EcdsaSecp256r1,
            Self::Ed25519(_) => Algorithm::Ed25519,
            Self::Sr25519(_) => Algorithm::Sr25519,
        }
    }

    /// Creates a public key from a byte sequence for a given signing algorithm.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `bytes` is not a valid public key for `algoritm`.
    pub fn from_bytes(algorithm: Algorithm, bytes: &[u8]) -> Result<Self> {
        Ok(match algorithm {
            Algorithm::EcdsaSecp256k1 => {
                Self::EcdsaSecp256k1(ecdsa::VerifyingKey::from_sec1_bytes(bytes)?)
            },
            Algorithm::EcdsaRecoverableSecp256k1 => {
                Self::EcdsaRecoverableSecp256k1(ecdsa::VerifyingKey::from_sec1_bytes(bytes)?)
            },
            Algorithm::EcdsaSecp256r1 => {
                Self::EcdsaSecp256r1(ecdsa::VerifyingKey::from_sec1_bytes(bytes)?)
            },
            Algorithm::Ed25519 => Self::Ed25519(ed25519_dalek::VerifyingKey::try_from(bytes)?),
            Algorithm::Sr25519 => {
                let public = schnorrkel::PublicKey::from_bytes(bytes)
                    .map_err(|err| anyhow::anyhow!("{}", err))?;
                Self::Sr25519(public)
            },
        })
    }

    /// Returns a byte sequence representing the public key.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::EcdsaSecp256k1(public) => public.to_encoded_point(true).as_bytes().to_vec(),
            Self::EcdsaRecoverableSecp256k1(public) => {
                public.to_encoded_point(true).as_bytes().to_vec()
            },
            Self::EcdsaSecp256r1(public) => public.to_encoded_point(true).as_bytes().to_vec(),
            Self::Ed25519(public) => public.to_bytes().to_vec(),
            Self::Sr25519(public) => public.to_bytes().to_vec(),
        }
    }

    /// Returns an uncompressed byte sequence representing the public key.
    #[must_use]
    pub fn to_uncompressed_bytes(&self) -> Vec<u8> {
        match self {
            Self::EcdsaSecp256k1(public) => public.to_encoded_point(false).as_bytes().to_vec(),
            Self::EcdsaRecoverableSecp256k1(public) => {
                public.to_encoded_point(false).as_bytes().to_vec()
            },
            Self::EcdsaSecp256r1(public) => public.to_encoded_point(false).as_bytes().to_vec(),
            Self::Ed25519(public) => public.to_bytes().to_vec(),
            Self::Sr25519(public) => public.to_bytes().to_vec(),
        }
    }

    /// Verifies a signature.
    ///
    /// # Errors
    ///
    /// Will return `Err` when:
    /// - Signature is invalid
    /// - The `sig` type doesn't match `self` type.
    pub fn verify(&self, msg: &[u8], sig: &Signature) -> Result<()> {
        match (self, &sig) {
            (Self::EcdsaSecp256k1(public), Signature::EcdsaSecp256k1(sig)) => {
                public.verify(msg, sig)?;
            },
            (
                Self::EcdsaRecoverableSecp256k1(public),
                Signature::EcdsaRecoverableSecp256k1(sig, _),
            ) => public.verify(msg, sig)?,
            (Self::EcdsaSecp256r1(public), Signature::EcdsaSecp256r1(sig)) => {
                public.verify(msg, sig)?;
            },
            (Self::Ed25519(public), Signature::Ed25519(sig)) => public.verify(msg, sig)?,
            (Self::Sr25519(public), Signature::Sr25519(sig)) => {
                public.verify_simple(&[], msg, sig).map_err(|err| anyhow::anyhow!("{}", err))?;
            },
            (_, _) => anyhow::bail!("unsupported signature scheme"),
        };
        Ok(())
    }
}

/// Signature.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Signature {
    /// ECDSA with secp256k1.
    EcdsaSecp256k1(ecdsa::Signature<k256::Secp256k1>),
    /// ECDSA with secp256k1 in Ethereum compatible format.
    EcdsaRecoverableSecp256k1(ecdsa::Signature<k256::Secp256k1>, RecoveryId),
    /// ECDSA with NIST P-256.
    EcdsaSecp256r1(ecdsa::Signature<p256::NistP256>),
    /// Ed25519.
    Ed25519(ed25519_dalek::Signature),
    /// Schnorrkel used by substrate/polkadot.
    Sr25519(schnorrkel::Signature),
}

impl Signature {
    /// Returns the signing algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> Algorithm {
        match self {
            Self::EcdsaSecp256k1(_) => Algorithm::EcdsaSecp256k1,
            Self::EcdsaRecoverableSecp256k1(_, _) => Algorithm::EcdsaRecoverableSecp256k1,
            Self::EcdsaSecp256r1(_) => Algorithm::EcdsaSecp256r1,
            Self::Ed25519(_) => Algorithm::Ed25519,
            Self::Sr25519(_) => Algorithm::Sr25519,
        }
    }

    /// Creates a signature from a byte sequence for a given signing algorithm.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `bytes` doesn't represent a valid signature for `algorithm`
    pub fn from_bytes(algorithm: Algorithm, bytes: &[u8]) -> Result<Self> {
        Ok(match algorithm {
            Algorithm::EcdsaSecp256k1 => Self::EcdsaSecp256k1(ecdsa::Signature::try_from(bytes)?),
            Algorithm::EcdsaRecoverableSecp256k1 => Self::EcdsaRecoverableSecp256k1(
                ecdsa::Signature::try_from(&bytes[..64])?,
                RecoveryId::from_byte(bytes[64]).context("invalid signature")?,
            ),
            Algorithm::EcdsaSecp256r1 => Self::EcdsaSecp256r1(ecdsa::Signature::try_from(bytes)?),
            Algorithm::Ed25519 => Self::Ed25519(ed25519_dalek::Signature::try_from(bytes)?),
            Algorithm::Sr25519 => {
                let sig = schnorrkel::Signature::from_bytes(bytes)
                    .map_err(|err| anyhow::anyhow!("{}", err))?;
                Self::Sr25519(sig)
            },
        })
    }

    /// Returns a byte sequence representing the signature.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::EcdsaSecp256k1(sig) => sig.to_vec(),
            Self::EcdsaRecoverableSecp256k1(sig, recovery_id) => {
                let mut bytes = Vec::with_capacity(65);
                bytes.extend(sig.to_bytes());
                bytes.push(recovery_id.to_byte());
                bytes
            },
            Self::EcdsaSecp256r1(sig) => sig.to_vec(),
            Self::Ed25519(sig) => sig.to_bytes().to_vec(),
            Self::Sr25519(sig) => sig.to_bytes().to_vec(),
        }
    }

    /// Returns the recovered public key if supported.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `msg` is the public key cannot be recovered
    pub fn recover(&self, msg: &[u8]) -> Result<Option<PublicKey>> {
        if let Self::EcdsaRecoverableSecp256k1(signature, recovery_id) = self {
            let recovered_key =
                ecdsa::VerifyingKey::recover_from_msg(msg, signature, *recovery_id)?;
            Ok(Some(PublicKey::EcdsaRecoverableSecp256k1(recovered_key)))
        } else {
            Ok(None)
        }
    }

    /// Returns the recovered public key if supported.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the public key cannot be recovered
    pub fn recover_prehashed(&self, hash: &[u8]) -> Result<Option<PublicKey>> {
        if let Self::EcdsaRecoverableSecp256k1(signature, recovery_id) = self {
            let recovered_key =
                ecdsa::VerifyingKey::recover_from_prehash(hash, signature, *recovery_id)?;
            Ok(Some(PublicKey::EcdsaRecoverableSecp256k1(recovered_key)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, RngCore};

    const ALGORITHMS: &[Algorithm] = &[
        Algorithm::EcdsaSecp256k1,
        Algorithm::EcdsaRecoverableSecp256k1,
        Algorithm::EcdsaSecp256r1,
        Algorithm::Ed25519,
        Algorithm::Sr25519,
    ];

    #[test]
    fn secret_key_bytes() -> Result<()> {
        let mut rng = thread_rng();
        let mut secret = [0; 32];
        rng.fill_bytes(&mut secret);
        for curve in ALGORITHMS {
            let secret_key = SecretKey::from_bytes(*curve, &secret[..])?;
            let secret2 = secret_key.to_bytes();
            assert_eq!(&secret[..], secret2);
        }
        Ok(())
    }

    #[test]
    fn public_key_bytes() -> Result<()> {
        let mut rng = thread_rng();
        let mut secret = [0; 32];
        rng.fill_bytes(&mut secret);
        for algorithm in ALGORITHMS {
            let secret_key = SecretKey::from_bytes(*algorithm, &secret[..])?;
            let public_key = secret_key.public_key();
            let public = public_key.to_bytes();
            let public_key2 = PublicKey::from_bytes(*algorithm, &public)?;
            assert_eq!(public_key, public_key2);
        }
        Ok(())
    }

    #[test]
    fn signature_bytes() -> Result<()> {
        let mut rng = thread_rng();
        let mut secret = [0; 32];
        rng.fill_bytes(&mut secret);
        let mut msg = [0; 32];
        rng.fill_bytes(&mut msg);
        for algorithm in ALGORITHMS {
            let secret_key = SecretKey::from_bytes(*algorithm, &secret[..])?;
            let signature = secret_key.sign(&msg, "");
            let sig = signature.to_bytes();
            let signature2 = Signature::from_bytes(*algorithm, &sig[..])?;
            assert_eq!(signature, signature2);
        }
        Ok(())
    }

    #[test]
    fn sign_verify() -> Result<()> {
        let mut rng = thread_rng();
        let mut secret = [0; 32];
        rng.fill_bytes(&mut secret);
        let mut msg = [0; 32];
        rng.fill_bytes(&mut msg);
        for algorithm in ALGORITHMS {
            let secret_key = SecretKey::from_bytes(*algorithm, &secret[..])?;
            let public_key = secret_key.public_key();
            let signature = secret_key.sign(&msg, "");
            public_key.verify(&msg, &signature)?;
        }
        Ok(())
    }

    #[test]
    fn sign_recover_pubkey() -> Result<()> {
        let mut rng = thread_rng();
        let mut secret = [0; 32];
        rng.fill_bytes(&mut secret);
        let mut msg = [0; 32];
        rng.fill_bytes(&mut msg);
        let secret_key = SecretKey::from_bytes(Algorithm::EcdsaRecoverableSecp256k1, &secret[..])?;
        let public_key = secret_key.public_key();
        let signature = secret_key.sign(&msg, "");
        let recovered_key = signature.recover(&msg)?.unwrap();
        assert_eq!(public_key, recovered_key);
        Ok(())
    }
}
