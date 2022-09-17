/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// TransactionIdentifier : The transaction_identifier uniquely identifies a transaction in a particular network and block or in the mempool.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct TransactionIdentifier {
    /// Any transactions that are attributable only to a block (ex: a block event) should use the hash of the block as the identifier.  This should be normalized according to the case specified in the transaction_hash_case in network options.
    #[serde(rename = "hash")]
    pub hash: String,
}

impl TransactionIdentifier {
    /// The transaction_identifier uniquely identifies a transaction in a particular network and block or in the mempool.
    pub fn new(hash: String) -> TransactionIdentifier {
        TransactionIdentifier { hash }
    }
}
