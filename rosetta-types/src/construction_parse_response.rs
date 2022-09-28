/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// ConstructionParseResponse : ConstructionParseResponse contains an array of operations that occur in a transaction blob. This should match the array of operations provided to `/construction/preprocess` and `/construction/payloads`.

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConstructionParseResponse {
    #[serde(rename = "operations")]
    pub operations: Vec<crate::Operation>,
    /// [DEPRECATED by `account_identifier_signers` in `v1.4.4`] All signers (addresses) of a particular transaction. If the transaction is unsigned, it should be empty.
    #[serde(rename = "signers", skip_serializing_if = "Option::is_none")]
    pub signers: Option<Vec<String>>,
    #[serde(
        rename = "account_identifier_signers",
        skip_serializing_if = "Option::is_none"
    )]
    pub account_identifier_signers: Option<Vec<crate::AccountIdentifier>>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl ConstructionParseResponse {
    /// ConstructionParseResponse contains an array of operations that occur in a transaction blob. This should match the array of operations provided to `/construction/preprocess` and `/construction/payloads`.
    pub fn new(operations: Vec<crate::Operation>) -> ConstructionParseResponse {
        ConstructionParseResponse {
            operations,
            signers: None,
            account_identifier_signers: None,
            metadata: None,
        }
    }
}
