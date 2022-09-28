/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// BlockTransaction : BlockTransaction contains a populated Transaction and the BlockIdentifier that contains it.

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BlockTransaction {
    #[serde(rename = "block_identifier")]
    pub block_identifier: crate::BlockIdentifier,
    #[serde(rename = "transaction")]
    pub transaction: crate::Transaction,
}

impl BlockTransaction {
    /// BlockTransaction contains a populated Transaction and the BlockIdentifier that contains it.
    pub fn new(
        block_identifier: crate::BlockIdentifier,
        transaction: crate::Transaction,
    ) -> BlockTransaction {
        BlockTransaction {
            block_identifier,
            transaction,
        }
    }
}
