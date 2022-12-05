//! Support for various blockchain address formats.
use crate::bip32::DerivedPublicKey;
use crate::PublicKey;

mod bech32;
mod eip55;
mod ss58;

pub use ss58::{Ss58AddressFormat, Ss58AddressFormatRegistry};

/// Address format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddressFormat {
    /// bech32
    Bech32(&'static str),
    /// eip55
    Eip55,
    /// ss58
    Ss58(Ss58AddressFormat),
}

impl From<Ss58AddressFormat> for AddressFormat {
    fn from(format: Ss58AddressFormat) -> Self {
        Self::Ss58(format)
    }
}

impl From<Ss58AddressFormatRegistry> for AddressFormat {
    fn from(format: Ss58AddressFormatRegistry) -> Self {
        Self::Ss58(format.into())
    }
}

/// Address.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Address {
    format: AddressFormat,
    address: String,
}

impl Address {
    /// Creates a new address.
    pub fn new(format: AddressFormat, address: String) -> Self {
        Self { format, address }
    }

    /// Formats the public key as an address.
    pub fn from_public_key_bytes(format: AddressFormat, public_key: &[u8]) -> Self {
        let address = match format {
            AddressFormat::Bech32(hrp) => bech32::bech32_encode(hrp, public_key),
            AddressFormat::Eip55 => eip55::eip55_encode(public_key),
            AddressFormat::Ss58(format) => ss58::ss58_encode(format, public_key),
        };
        Self::new(format, address)
    }

    /// Returns the format of the address.
    pub fn format(&self) -> AddressFormat {
        self.format
    }

    /// Returns the address.
    pub fn address(&self) -> &str {
        &self.address
    }
}

impl From<Address> for String {
    fn from(address: Address) -> Self {
        address.address
    }
}

impl PublicKey {
    /// Returns the address of a public key.
    pub fn to_address(&self, format: AddressFormat) -> Address {
        Address::from_public_key_bytes(format, &self.to_bytes())
    }
}

impl DerivedPublicKey {
    /// Returns the address of a public key.
    pub fn to_address(&self, format: AddressFormat) -> Address {
        Address::from_public_key_bytes(format, &self.public_key().to_bytes())
    }
}
