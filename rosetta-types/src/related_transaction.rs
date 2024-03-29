/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `RelatedTransaction` : The `related_transaction` allows implementations to link together
/// multiple transactions. An unpopulated network identifier indicates that the related transaction
/// is on the same network.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RelatedTransaction {
    #[serde(rename = "transaction_identifier")]
    pub transaction_identifier: crate::TransactionIdentifier,
    #[serde(rename = "direction")]
    pub direction: crate::Direction,
}

impl RelatedTransaction {
    /// The `related_transaction` allows implementations to link together multiple transactions. An
    /// unpopulated network identifier indicates that the related transaction is on the same
    /// network.
    #[must_use]
    pub const fn new(
        transaction_identifier: crate::TransactionIdentifier,
        direction: crate::Direction,
    ) -> Self {
        Self { transaction_identifier, direction }
    }
}
