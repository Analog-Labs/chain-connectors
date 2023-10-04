/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `ConstructionTransactionResponse` is returned by `/construction/payloads`. It contains an unsigned transaction blob (that is usually needed to construct the a network transaction from a collection of signatures) and an array of payloads that must be signed by the caller.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConstructionPayloadsResponse {
    #[serde(rename = "unsigned_transaction")]
    pub unsigned_transaction: String,
    #[serde(rename = "payloads")]
    pub payloads: Vec<crate::SigningPayload>,
}

impl ConstructionPayloadsResponse {
    /// `ConstructionTransactionResponse` is returned by `/construction/payloads`. It contains an unsigned transaction blob (that is usually needed to construct the a network transaction from a collection of signatures) and an array of payloads that must be signed by the caller.
    #[must_use]
    pub const fn new(unsigned_transaction: String, payloads: Vec<crate::SigningPayload>) -> Self {
        Self {
            unsigned_transaction,
            payloads,
        }
    }
}
