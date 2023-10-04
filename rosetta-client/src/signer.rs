use crate::{
    crypto::{
        address::Address,
        bip32::{DerivedPublicKey, DerivedSecretKey},
        bip39::Mnemonic,
        bip44::ChildNumber,
        Algorithm,
    },
    types::{AccountIdentifier, CurveType, PublicKey},
};
use anyhow::Result;

/// Signer derives keys from a mnemonic.
pub struct Signer {
    secp256k1: DerivedSecretKey,
    secp256k1_recoverable: DerivedSecretKey,
    secp256r1: DerivedSecretKey,
    ed25519: DerivedSecretKey,
    sr25519: DerivedSecretKey,
}

impl Signer {
    /// Creates a new signer from a mnemonic and password.
    #[allow(clippy::similar_names, clippy::missing_errors_doc)]
    pub fn new(mnemonic: &Mnemonic, password: &str) -> Result<Self> {
        let secp256k1 = DerivedSecretKey::new(mnemonic, password, Algorithm::EcdsaSecp256k1)?;
        let secp256k1_recoverable =
            DerivedSecretKey::new(mnemonic, password, Algorithm::EcdsaRecoverableSecp256k1)?;
        let secp256r1 = DerivedSecretKey::new(mnemonic, password, Algorithm::EcdsaSecp256r1)?;
        let ed25519 = DerivedSecretKey::new(mnemonic, password, Algorithm::Ed25519)?;
        let sr25519 = DerivedSecretKey::new(mnemonic, password, Algorithm::Sr25519)?;
        Ok(Self { secp256k1, secp256k1_recoverable, secp256r1, ed25519, sr25519 })
    }

    /// Creates a new ephemeral signer.
    #[allow(unused, clippy::missing_errors_doc)]
    pub fn generate() -> Result<Self> {
        let mnemonic = crate::mnemonic::generate_mnemonic()?;
        Self::new(&mnemonic, "")
    }

    /// Derives a master key from a mnemonic.
    #[must_use]
    pub const fn master_key(&self, algorithm: Algorithm) -> &DerivedSecretKey {
        match algorithm {
            Algorithm::EcdsaSecp256k1 => &self.secp256k1,
            Algorithm::EcdsaRecoverableSecp256k1 => &self.secp256k1_recoverable,
            Algorithm::EcdsaSecp256r1 => &self.secp256r1,
            Algorithm::Ed25519 => &self.ed25519,
            Algorithm::Sr25519 => &self.sr25519,
        }
    }

    /// Derives a bip44 key from a mnemonic.
    #[allow(clippy::missing_errors_doc)]
    pub fn bip44_account(
        &self,
        algorithm: Algorithm,
        coin: u32,
        account: u32,
    ) -> Result<DerivedSecretKey> {
        self.master_key(algorithm)
            .derive(ChildNumber::hardened_from_u32(44))?
            .derive(ChildNumber::hardened_from_u32(coin))?
            .derive(ChildNumber::hardened_from_u32(account))?
            .derive(ChildNumber::non_hardened_from_u32(0))
    }
}

/// Conversion trait for public keys.
pub trait RosettaPublicKey {
    /// Returns a rosetta public key.
    fn to_rosetta(&self) -> PublicKey;
}

impl RosettaPublicKey for DerivedPublicKey {
    fn to_rosetta(&self) -> PublicKey {
        PublicKey {
            curve_type: match self.public_key().algorithm() {
                Algorithm::EcdsaSecp256k1 | Algorithm::EcdsaRecoverableSecp256k1 =>
                    CurveType::Secp256k1,
                Algorithm::EcdsaSecp256r1 => CurveType::Secp256r1,
                Algorithm::Ed25519 => CurveType::Edwards25519,
                Algorithm::Sr25519 => CurveType::Schnorrkel,
            },
            hex_bytes: hex::encode(self.public_key().to_bytes()),
        }
    }
}

/// Conversion trait for account identifiers.
pub trait RosettaAccount {
    /// Returns a rosetta account identifier.
    fn to_rosetta(&self) -> AccountIdentifier;
}

impl RosettaAccount for Address {
    fn to_rosetta(&self) -> AccountIdentifier {
        AccountIdentifier { address: self.address().into(), sub_account: None, metadata: None }
    }
}
