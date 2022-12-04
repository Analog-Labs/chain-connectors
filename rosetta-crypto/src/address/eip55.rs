use crate::{Algorithm, PublicKey};
use sha3::Digest;

pub fn eip55_encode(public_key: &[u8]) -> String {
    let uncompressed = PublicKey::from_bytes(Algorithm::EcdsaSecp256k1, public_key)
        .unwrap()
        .to_uncompressed_bytes();
    let digest = sha3::Keccak256::digest(&uncompressed[1..]);
    eip55_encode_bytes(&digest[12..])
}

fn eip55_encode_bytes(bytes: &[u8]) -> String {
    let address = hex::encode(bytes);
    let hashed_address = hex::encode(sha3::Keccak256::digest(&address));
    let mut result = String::with_capacity(42);
    result.push_str("0x");
    for (nibble_index, mut character) in address.chars().enumerate() {
        if character.is_alphabetic() {
            let nibble = hashed_address.as_bytes()[nibble_index] as char;
            if nibble.to_digit(16).unwrap() > 7 {
                character = character.to_ascii_uppercase()
            }
        }
        result.push(character);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vectors() {
        let vectors = [
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
            "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
            "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
            "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
        ];
        for address in vectors {
            let bytes = hex::decode(&address[2..]).unwrap();
            let address2 = eip55_encode_bytes(&bytes[..]);
            assert_eq!(address, address2);
        }
    }

    #[test]
    fn test_derive() {
        let pubkey = "0x03f349dec2b5205707c778534a7f134125ea31e82134e5aa987417f1091103e263";
        let address = "0x445CB6cE4047FB4689ec53827eC4457BA8D05F94";
        let pubkey = hex::decode(&pubkey[2..]).unwrap();
        let address2 = eip55_encode(&pubkey);
        assert_eq!(address, address2);
    }
}
