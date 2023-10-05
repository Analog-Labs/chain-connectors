/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `ConstructionPreprocessRequest` is passed to the `/construction/preprocess` endpoint so that a
/// Rosetta implementation can determine which metadata it needs to request for construction.
/// Metadata provided in this object should NEVER be a product of live data (i.e. the caller must
/// follow some network-specific data fetching strategy outside of the Construction API to populate
/// required Metadata). If live data is required for construction, it MUST be fetched in the call to
/// `/construction/metadata`.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConstructionPreprocessRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: crate::NetworkIdentifier,
    #[serde(rename = "operations")]
    pub operations: Vec<crate::Operation>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl ConstructionPreprocessRequest {
    /// `ConstructionPreprocessRequest` is passed to the `/construction/preprocess` endpoint so that
    /// a Rosetta implementation can determine which metadata it needs to request for construction.
    /// Metadata provided in this object should NEVER be a product of live data (i.e. the caller
    /// must follow some network-specific data fetching strategy outside of the Construction API to
    /// populate required Metadata). If live data is required for construction, it MUST be fetched
    /// in the call to `/construction/metadata`.
    #[must_use]
    pub const fn new(
        network_identifier: crate::NetworkIdentifier,
        operations: Vec<crate::Operation>,
    ) -> Self {
        Self { network_identifier, operations, metadata: None }
    }
}
