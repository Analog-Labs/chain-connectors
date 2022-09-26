use anyhow::Result;
use std::str::FromStr;

const HARDENED_BIT: u32 = 1 << 31;

/// A child number for a derived key
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ChildNumber(u32);

impl ChildNumber {
    pub fn is_hardened(&self) -> bool {
        self.0 & HARDENED_BIT == HARDENED_BIT
    }

    pub fn is_normal(&self) -> bool {
        self.0 & HARDENED_BIT == 0
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    pub fn hardened_from_u32(index: u32) -> Self {
        ChildNumber(index | HARDENED_BIT)
    }

    pub fn non_hardened_from_u32(index: u32) -> Self {
        ChildNumber(index)
    }

    pub fn index(&self) -> u32 {
        self.0 & (i32::MAX as u32)
    }

    pub fn to_substrate_chain_code(&self) -> [u8; 32] {
        let mut chain_code = [0; 32];
        let bytes = (self.index() as u64).to_le_bytes();
        chain_code[..bytes.len()].copy_from_slice(&bytes[..]);
        chain_code
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
