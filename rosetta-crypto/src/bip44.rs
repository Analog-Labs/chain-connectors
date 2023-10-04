//! BIP44 implementation.
use anyhow::Result;
use std::str::FromStr;

const HARDENED_BIT: u32 = 1 << 31;

/// A child number for a derived key.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ChildNumber(u32);

impl ChildNumber {
    /// Is a hard derivation.
    #[must_use]
    pub const fn is_hardened(&self) -> bool {
        self.0 & HARDENED_BIT == HARDENED_BIT
    }

    /// Is a normal derivation.
    #[must_use]
    pub const fn is_normal(&self) -> bool {
        self.0 & HARDENED_BIT == 0
    }

    /// Creates a new hard derivation.
    #[must_use]
    pub const fn hardened_from_u32(index: u32) -> Self {
        Self(index | HARDENED_BIT)
    }

    /// Creates a new soft derivation.
    #[must_use]
    pub const fn non_hardened_from_u32(index: u32) -> Self {
        Self(index)
    }

    /// Returns the index.
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.0 & (i32::MAX as u32)
    }

    /// Returns BIP32 byte sequence.
    #[must_use]
    pub const fn to_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    /// Returns the substrate compatible chain code.
    #[must_use]
    pub fn to_substrate_chain_code(&self) -> [u8; 32] {
        let mut chain_code = [0; 32];
        let bytes = u64::from(self.index()).to_le_bytes();
        chain_code[..bytes.len()].copy_from_slice(&bytes[..]);
        chain_code
    }
}

impl core::ops::Add<u32> for ChildNumber {
    type Output = Self;

    fn add(self, other: u32) -> Self::Output {
        Self(self.0 + other)
    }
}

impl FromStr for ChildNumber {
    type Err = anyhow::Error;

    fn from_str(child: &str) -> Result<Self> {
        let (child, mask) =
            child.strip_suffix('\'').map_or((child, 0), |child| (child, HARDENED_BIT));

        let index: u32 = child.parse()?;

        if index & HARDENED_BIT != 0 {
            anyhow::bail!("invalid child number");
        }

        Ok(Self(index | mask))
    }
}

/// BIP44 key derivation path.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct DerivationPath {
    path: Vec<ChildNumber>,
}

impl FromStr for DerivationPath {
    type Err = anyhow::Error;

    fn from_str(path: &str) -> Result<Self> {
        let mut path = path.split('/');

        if path.next() != Some("m") {
            anyhow::bail!("invalid derivation path");
        }

        Ok(Self { path: path.map(str::parse).collect::<Result<Vec<ChildNumber>>>()? })
    }
}

impl AsRef<[ChildNumber]> for DerivationPath {
    fn as_ref(&self) -> &[ChildNumber] {
        &self.path
    }
}

impl DerivationPath {
    /// Returns an iterator of child numbers.
    pub fn iter(&self) -> impl Iterator<Item = &ChildNumber> {
        self.path.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_path() {
        let path: DerivationPath = "m/44'/60'/0'/0".parse().unwrap();

        assert_eq!(
            path,
            DerivationPath {
                path: vec![
                    ChildNumber::hardened_from_u32(44),
                    ChildNumber::hardened_from_u32(60),
                    ChildNumber::hardened_from_u32(0),
                    ChildNumber::non_hardened_from_u32(0),
                ],
            }
        );
    }
}
