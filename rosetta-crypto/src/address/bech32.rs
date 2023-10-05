use bech32::{u5, ToBase32, Variant};
use sha2::Digest;

#[allow(clippy::unwrap_used)]
pub fn bech32_encode(hrp: &str, public_key: &[u8]) -> String {
    let sha2 = sha2::Sha256::digest(public_key);
    let ripemd = ripemd::Ripemd160::digest(sha2);
    let mut bytes = Vec::with_capacity(33);
    bytes.push(u5::try_from_u8(0x00).unwrap());
    ripemd.write_base32(&mut bytes).unwrap();
    bech32::encode(hrp, bytes, Variant::Bech32).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive() {
        let pubkey = "0x03f349dec2b5205707c778534a7f134125ea31e82134e5aa987417f1091103e263";
        let address = "bcrt1qsqxddufe9qz0phxnntsgytg3wr8sl9z4czyj5k";
        let pubkey = hex::decode(&pubkey[2..]).unwrap();
        let address2 = bech32_encode("bcrt", &pubkey);
        assert_eq!(address, address2);
    }
}
