/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// ConstructionPreprocessResponse : ConstructionPreprocessResponse contains `options` that will be sent unmodified to `/construction/metadata`. If it is not necessary to make a request to `/construction/metadata`, `options` should be omitted.   Some blockchains require the PublicKey of particular AccountIdentifiers to construct a valid transaction. To fetch these PublicKeys, populate `required_public_keys` with the AccountIdentifiers associated with the desired PublicKeys. If it is not necessary to retrieve any PublicKeys for construction, `required_public_keys` should be omitted.

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConstructionPreprocessResponse {
    /// The options that will be sent directly to `/construction/metadata` by the caller.
    #[serde(rename = "options", skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
    #[serde(
        rename = "required_public_keys",
        skip_serializing_if = "Option::is_none"
    )]
    pub required_public_keys: Option<Vec<crate::AccountIdentifier>>,
}

impl ConstructionPreprocessResponse {
    /// ConstructionPreprocessResponse contains `options` that will be sent unmodified to `/construction/metadata`. If it is not necessary to make a request to `/construction/metadata`, `options` should be omitted.   Some blockchains require the PublicKey of particular AccountIdentifiers to construct a valid transaction. To fetch these PublicKeys, populate `required_public_keys` with the AccountIdentifiers associated with the desired PublicKeys. If it is not necessary to retrieve any PublicKeys for construction, `required_public_keys` should be omitted.
    pub fn new() -> ConstructionPreprocessResponse {
        ConstructionPreprocessResponse {
            options: None,
            required_public_keys: None,
        }
    }
}
