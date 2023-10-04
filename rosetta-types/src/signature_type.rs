/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `SignatureType` is the type of a cryptographic signature.
/// * `ecdsa`: `r (32-bytes) || s (32-bytes)` - `64 bytes`
/// * `ecdsa_recovery`: `r (32-bytes) || s (32-bytes) || v (1-byte)` - `65 bytes`
/// * `ed25519`: `R (32-byte) || s (32-bytes)` - `64 bytes`
/// * `schnorr_1`: `r (32-bytes) || s (32-bytes)` - `64 bytes`  (schnorr signature implemented by Zilliqa where both `r` and `s` are scalars encoded as `32-bytes` values, most significant byte first.)
/// * `schnorr_poseidon`: `r (32-bytes) || s (32-bytes)` where s = Hash(1st pk || 2nd pk || r) - `64 bytes`  (schnorr signature w/ Poseidon hash function implemented by O(1) Labs where both `r` and `s` are scalars encoded as `32-bytes` values, least significant byte first. [reference](https://github.com/CodaProtocol/signer-reference/blob/master/schnorr.ml) )
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum SignatureType {
    #[serde(rename = "ecdsa")]
    Ecdsa,
    #[serde(rename = "ecdsa_recovery")]
    EcdsaRecovery,
    #[serde(rename = "ed25519")]
    Ed25519,
    #[serde(rename = "schnorr_1")]
    Schnorr1,
    #[serde(rename = "schnorr_poseidon")]
    SchnorrPoseidon,
    #[serde(rename = "sr25519")]
    Sr25519,
}

impl ToString for SignatureType {
    fn to_string(&self) -> String {
        match self {
            Self::Ecdsa => String::from("ecdsa"),
            Self::EcdsaRecovery => String::from("ecdsa_recovery"),
            Self::Ed25519 => String::from("ed25519"),
            Self::Schnorr1 => String::from("schnorr_1"),
            Self::SchnorrPoseidon => String::from("schnorr_poseidon"),
            Self::Sr25519 => String::from("sr25519"),
        }
    }
}

impl Default for SignatureType {
    fn default() -> Self {
        Self::Ecdsa
    }
}
