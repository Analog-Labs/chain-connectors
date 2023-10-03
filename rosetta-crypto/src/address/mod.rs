//! Support for various blockchain address formats.
use crate::bip32::DerivedPublicKey;
use crate::error::AddressError;
use crate::PublicKey;
use ethers::types::H160;
use sp_core::{
    crypto::{AccountId32, Ss58Codec},
    hashing::blake2_256,
};

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
    #[must_use]
    pub const fn new(format: AddressFormat, address: String) -> Self {
        Self { format, address }
    }

    /// Formats the public key as an address.
    #[must_use]
    pub fn from_public_key_bytes(format: AddressFormat, public_key: &[u8]) -> Self {
        let address = match format {
            AddressFormat::Bech32(hrp) => bech32::bech32_encode(hrp, public_key),
            AddressFormat::Eip55 => eip55::eip55_encode(public_key),
            AddressFormat::Ss58(format) => ss58::ss58_encode(format, public_key),
        };
        Self::new(format, address)
    }

    /// Converts an EVM address to its corresponding SS58 address.
    /// reference: [evmToAddress.ts](https://github.com/polkadot-js/common/blob/v12.3.2/packages/util-crypto/src/address/evmToAddress.ts)
    ///
    /// # Errors
    ///
    /// Will return `Err` when `self.address` is not a valid 160bit hex string
    pub fn evm_to_ss58(&self, ss58format: Ss58AddressFormat) -> Result<Self, AddressError> {
        if self.format != AddressFormat::Eip55 {
            return Err(AddressError::InvalidAddressFormat);
        }
        let address: H160 = self
            .address
            .parse()
            .map_err(|_| AddressError::FailedToDecodeAddress)?;
        let mut data = [0u8; 24];
        data[0..4].copy_from_slice(b"evm:");
        data[4..24].copy_from_slice(&address[..]);
        let hash = blake2_256(&data);
        Ok(Self {
            format: AddressFormat::Ss58(ss58format),
            address: ss58::ss58_encode(ss58format, &hash),
        })
    }

    /// Converts an SS58 address to its corresponding EVM address.
    /// reference: [addressToEvm.ts](https://github.com/polkadot-js/common/blob/v12.3.2/packages/util-crypto/src/address/addressToEvm.ts#L13)
    ///
    /// # Errors
    /// Will return `Err` when:
    /// * self.format is not [`AddressFormat::Ss58`]
    /// * self.address is not a valid SS58 address string
    ///
    pub fn ss58_to_evm(&self) -> Result<Self, AddressError> {
        if !matches!(self.format, AddressFormat::Ss58(_)) {
            return Err(AddressError::InvalidAddressFormat);
        }
        let ss58_addr = <AccountId32 as Ss58Codec>::from_string(&self.address)
            .map_err(|_| AddressError::FailedToDecodeAddress)?;
        let bytes: [u8; 32] = ss58_addr.into();
        Ok(Self {
            format: AddressFormat::Eip55,
            address: hex::encode(&bytes[0..20]),
        })
    }

    /// Returns the format of the address.
    #[must_use]
    pub const fn format(&self) -> AddressFormat {
        self.format
    }

    /// Returns the address.
    #[must_use]
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
    #[must_use]
    pub fn to_address(&self, format: AddressFormat) -> Address {
        Address::from_public_key_bytes(format, &self.to_bytes())
    }
}

impl DerivedPublicKey {
    /// Returns the address of a public key.
    #[must_use]
    pub fn to_address(&self, format: AddressFormat) -> Address {
        Address::from_public_key_bytes(format, &self.public_key().to_bytes())
    }
}
