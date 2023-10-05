/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `BlockTransactionResponse` : A `BlockTransactionResponse` contains information about a block
/// transaction.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BlockTransactionResponse {
    #[serde(rename = "transaction")]
    pub transaction: crate::Transaction,
}

impl BlockTransactionResponse {
    /// A `BlockTransactionResponse` contains information about a block transaction.
    #[must_use]
    pub const fn new(transaction: crate::Transaction) -> Self {
        Self { transaction }
    }
}
