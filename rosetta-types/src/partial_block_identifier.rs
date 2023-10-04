/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `PartialBlockIdentifier` : When fetching data by `BlockIdentifier`, it may be possible to only
/// specify the index or hash. If neither property is specified, it is assumed that the client is
/// making a request at the current block.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PartialBlockIdentifier {
    #[serde(rename = "index", skip_serializing_if = "Option::is_none")]
    pub index: Option<u64>,
    #[serde(rename = "hash", skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl PartialBlockIdentifier {
    /// When fetching data by `BlockIdentifier`, it may be possible to only specify the index or
    /// hash. If neither property is specified, it is assumed that the client is making a request at
    /// the current block.
    #[must_use]
    pub const fn new() -> Self {
        Self { index: None, hash: None }
    }
}
