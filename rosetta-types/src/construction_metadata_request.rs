/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `ConstructionMetadataRequest` : A `ConstructionMetadataRequest` is utilized to get information required to construct a transaction.  The Options object used to specify which metadata to return is left purposely unstructured to allow flexibility for implementers. Options is not required in the case that there is network-wide metadata of interest.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConstructionMetadataRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: crate::NetworkIdentifier,
    /// Some blockchains require different metadata for different types of transaction construction (ex: delegation versus a transfer). Instead of requiring a blockchain node to return all possible types of metadata for construction (which may require multiple node fetches), the client can populate an options object to limit the metadata returned to only the subset required.
    #[serde(rename = "options", skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,

    #[serde(rename = "public_keys", skip_serializing_if = "Vec::is_empty")]
    pub public_keys: Vec<crate::PublicKey>,
}

impl ConstructionMetadataRequest {
    /// A `ConstructionMetadataRequest` is utilized to get information required to construct a transaction.  The Options object used to specify which metadata to return is left purposely unstructured to allow flexibility for implementers. Options is not required in the case that there is network-wide metadata of interest.
    #[must_use]
    pub const fn new(network_identifier: crate::NetworkIdentifier) -> Self {
        Self {
            network_identifier,
            options: None,
            public_keys: vec![],
        }
    }
}
