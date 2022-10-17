use crate::crypto::{
    bip32::{DerivedPublicKey, DerivedSecretKey},
    bip39::Mnemonic,
    bip44::ChildNumber,
    Algorithm,
};
use crate::types::{CurveType, PublicKey, Signature, SignatureType, SigningPayload};
use anyhow::Result;

pub struct Signer {
    secp256k1: DerivedSecretKey,
    secp256k1_recoverable: DerivedSecretKey,
    secp256r1: DerivedSecretKey,
    ed25519: DerivedSecretKey,
    sr25519: DerivedSecretKey,
}

impl Signer {
    pub fn new(mnemonic: &Mnemonic, password: &str) -> Result<Self> {
        let secp256k1 = DerivedSecretKey::new(mnemonic, password, Algorithm::EcdsaSecp256k1)?;
        let secp256k1_recoverable =
            DerivedSecretKey::new(mnemonic, password, Algorithm::EcdsaRecoverableSecp256k1)?;
        let secp256r1 = DerivedSecretKey::new(mnemonic, password, Algorithm::EcdsaSecp256r1)?;
        let ed25519 = DerivedSecretKey::new(mnemonic, password, Algorithm::Ed25519)?;
        let sr25519 = DerivedSecretKey::new(mnemonic, password, Algorithm::Sr25519)?;
        Ok(Self {
            secp256k1,
            secp256k1_recoverable,
            secp256r1,
            ed25519,
            sr25519,
        })
    }

    pub fn master_key(&self, algorithm: Algorithm) -> Result<&DerivedSecretKey> {
        Ok(match algorithm {
            Algorithm::EcdsaSecp256k1 => &self.secp256k1,
            Algorithm::EcdsaRecoverableSecp256k1 => &self.secp256k1_recoverable,
            Algorithm::EcdsaSecp256r1 => &self.secp256r1,
            Algorithm::Ed25519 => &self.ed25519,
            Algorithm::Sr25519 => &self.sr25519,
        })
    }

    pub fn bip44_account(
        &self,
        algorithm: Algorithm,
        coin: u32,
        account: u32,
    ) -> Result<DerivedSecretKey> {
        self.master_key(algorithm)?
            .derive(ChildNumber::hardened_from_u32(44))?
            .derive(ChildNumber::hardened_from_u32(coin))?
            .derive(ChildNumber::hardened_from_u32(account))
    }
}

pub trait RosettaSecretKey {
    fn sign(&self, payload: SigningPayload) -> Result<Signature>;
}

pub trait RosettaPublicKey {
    fn to_rosetta(&self) -> PublicKey;
}

impl RosettaSecretKey for DerivedSecretKey {
    fn sign(&self, payload: SigningPayload) -> Result<Signature> {
        let payload_bytes = hex::decode(&payload.hex_bytes)?;
        let secret_key = self.secret_key();
        let (signature, signature_type) = match secret_key.algorithm() {
            Algorithm::EcdsaSecp256k1 => (
                secret_key.sign_prehashed(&payload_bytes)?,
                SignatureType::Ecdsa,
            ),
            Algorithm::EcdsaRecoverableSecp256k1 => (
                secret_key.sign_prehashed(&payload_bytes)?,
                SignatureType::EcdsaRecovery,
            ),
            Algorithm::EcdsaSecp256r1 => (
                secret_key.sign_prehashed(&payload_bytes)?,
                SignatureType::Ecdsa,
            ),
            Algorithm::Ed25519 => (
                secret_key.sign_prehashed(&payload_bytes)?,
                SignatureType::Ed25519,
            ),
            Algorithm::Sr25519 => (
                secret_key.sign_prehashed(&payload_bytes)?,
                SignatureType::Sr25519,
            ),
        };
        Ok(Signature {
            signing_payload: payload,
            public_key: self.public_key().to_rosetta(),
            signature_type,
            hex_bytes: hex::encode(&signature.to_bytes()),
        })
    }
}

impl RosettaPublicKey for DerivedPublicKey {
    fn to_rosetta(&self) -> PublicKey {
        PublicKey {
            curve_type: match self.public_key().algorithm() {
                Algorithm::EcdsaSecp256k1 => CurveType::Secp256k1,
                Algorithm::EcdsaRecoverableSecp256k1 => CurveType::Secp256k1,
                Algorithm::EcdsaSecp256r1 => CurveType::Secp256r1,
                Algorithm::Ed25519 => CurveType::Edwards25519,
                Algorithm::Sr25519 => CurveType::Schnorrkel,
            },
            hex_bytes: hex::encode(&self.public_key().to_bytes()),
        }
    }
}
