pub use ss58_registry::{Ss58AddressFormat, Ss58AddressFormatRegistry};

/// Encodes an address bytes into specified SS58 format.
pub fn ss58_encode(address_format: Ss58AddressFormat, public_key: &[u8]) -> String {
    // We mask out the upper two bits of the ident - SS58 Prefix currently only supports 14-bits
    let ident: u16 = u16::from(address_format) & 0b0011_1111_1111_1111;
    let mut v = match ident {
        0..=63 => {
            // The value will not truncate once is between 0 and 63
            #[allow(clippy::cast_possible_truncation)]
            let ident = ident as u8;
            vec![ident]
        }
        64..=16_383 => {
            // upper six bits of the lower byte(!)
            let first = ((ident & 0b0000_0000_1111_1100) as u8) >> 2;
            // lower two bits of the lower byte in the high pos,
            // lower bits of the upper byte in the low pos
            let second = ((ident >> 8) as u8) | ((ident & 0b0000_0000_0000_0011) as u8) << 6;
            vec![first | 0b0100_0000, second]
        }
        _ => unreachable!("masked out the upper two bits; qed"),
    };
    v.extend(public_key);
    let r = ss58hash(&v);
    v.extend(&r.as_bytes()[0..2]);
    bs58::encode(&v).into_string()
}

fn ss58hash(data: &[u8]) -> blake2_rfc::blake2b::Blake2bResult {
    let mut context = blake2_rfc::blake2b::Blake2b::new(64);
    context.update(b"SS58PRE");
    context.update(data);
    context.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ss58_registry::Ss58AddressFormatRegistry;

    #[test]
    fn test_ss58_encode() {
        let public_key = "ec41bdaf7893f2dc6dd853eecfdaa220a7d87b6f05801cae18db11ca7b1ba731";
        let ss58 = "5HQUgoe4VCFp4q42XbnnFhDTaveW9W5LQfqiGMVGfTiKDvqi";
        let address_format = Ss58AddressFormat::from(Ss58AddressFormatRegistry::SubstrateAccount);
        let public_key = hex::decode(public_key).unwrap();
        assert_eq!(ss58_encode(address_format, &public_key), ss58);
    }
}
