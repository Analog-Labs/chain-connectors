/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `ConstructionHashRequest` : `ConstructionHashRequest` is the input to the `/construction/hash`
/// endpoint.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConstructionHashRequest {
    #[serde(rename = "signed_transaction")]
    pub signed_transaction: String,
}

impl ConstructionHashRequest {
    /// `ConstructionHashRequest` is the input to the `/construction/hash` endpoint.
    #[must_use]
    pub const fn new(signed_transaction: String) -> Self {
        Self { signed_transaction }
    }
}
