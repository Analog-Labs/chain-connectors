/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// ConstructionCombineResponse : ConstructionCombineResponse is returned by `/construction/combine`. The network payload will be sent directly to the `construction/submit` endpoint.

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConstructionCombineResponse {
    #[serde(rename = "signed_transaction")]
    pub signed_transaction: String,
}

impl ConstructionCombineResponse {
    /// ConstructionCombineResponse is returned by `/construction/combine`. The network payload will be sent directly to the `construction/submit` endpoint.
    pub fn new(signed_transaction: String) -> ConstructionCombineResponse {
        ConstructionCombineResponse { signed_transaction }
    }
}
