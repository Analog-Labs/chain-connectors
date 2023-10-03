/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `MempoolTransactionRequest` : A `MempoolTransactionRequest` is utilized to retrieve a transaction from the mempool.

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MempoolTransactionRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: crate::NetworkIdentifier,
    #[serde(rename = "transaction_identifier")]
    pub transaction_identifier: crate::TransactionIdentifier,
}

impl MempoolTransactionRequest {
    /// A `MempoolTransactionRequest` is utilized to retrieve a transaction from the mempool.
    #[must_use] pub fn new(
        network_identifier: crate::NetworkIdentifier,
        transaction_identifier: crate::TransactionIdentifier,
    ) -> Self {
        Self {
            network_identifier,
            transaction_identifier,
        }
    }
}
