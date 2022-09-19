/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// ConstructionSubmitRequest : The transaction submission request includes a signed transaction.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ConstructionSubmitRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: crate::NetworkIdentifier,
    #[serde(rename = "signed_transaction")]
    pub signed_transaction: String,
}

impl ConstructionSubmitRequest {
    /// The transaction submission request includes a signed transaction.
    pub fn new(
        network_identifier: crate::NetworkIdentifier,
        signed_transaction: String,
    ) -> ConstructionSubmitRequest {
        ConstructionSubmitRequest {
            network_identifier,
            signed_transaction,
        }
    }
}
