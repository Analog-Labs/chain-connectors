//! BIP44 implementation.
use anyhow::Result;
use std::str::FromStr;

const HARDENED_BIT: u32 = 1 << 31;

/// A child number for a derived key.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ChildNumber(u32);

impl ChildNumber {
    /// Is a hard derivation.
    pub fn is_hardened(&self) -> bool {
        self.0 & HARDENED_BIT == HARDENED_BIT
    }

    /// Is a normal derivation.
    pub fn is_normal(&self) -> bool {
        self.0 & HARDENED_BIT == 0
    }

    /// Creates a new hard derivation.
    pub fn hardened_from_u32(index: u32) -> Self {
        ChildNumber(index | HARDENED_BIT)
    }

    /// Creates a new soft derivation.
    pub fn non_hardened_from_u32(index: u32) -> Self {
        ChildNumber(index)
    }

    /// Returns the index.
    pub fn index(&self) -> u32 {
        self.0 & (i32::MAX as u32)
    }

    /// Returns BIP32 byte sequence.
    pub fn to_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    /// Returns the substrate compatible chain code.
    pub fn to_substrate_chain_code(&self) -> [u8; 32] {
        let mut chain_code = [0; 32];
        let bytes = (self.index() as u64).to_le_bytes();
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

    fn from_str(child: &str) -> Result<ChildNumber> {
        let (child, mask) = if let Some(child) = child.strip_suffix('\'') {
            (child, HARDENED_BIT)
        } else {
            (child, 0)
        };

        let index: u32 = child.parse()?;

        if index & HARDENED_BIT != 0 {
            anyhow::bail!("invalid child number");
        }

        Ok(ChildNumber(index | mask))
    }
}

/// BIP44 key derivation path.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct DerivationPath {
    path: Vec<ChildNumber>,
}

impl FromStr for DerivationPath {
    type Err = anyhow::Error;

    fn from_str(path: &str) -> Result<DerivationPath> {
        let mut path = path.split('/');

        if path.next() != Some("m") {
            anyhow::bail!("invalid derivation path");
        }

        Ok(DerivationPath {
            path: path.map(str::parse).collect::<Result<Vec<ChildNumber>>>()?,
        })
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
                    ChildNumber(44 | HARDENED_BIT),
                    ChildNumber(60 | HARDENED_BIT),
                    ChildNumber(0 | HARDENED_BIT),
                    ChildNumber(0),
                ],
            }
        );
    }
}
