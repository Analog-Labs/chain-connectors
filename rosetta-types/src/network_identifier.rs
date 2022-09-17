/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// NetworkIdentifier : The network_identifier specifies which network a particular object is associated with.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct NetworkIdentifier {
    #[serde(rename = "blockchain")]
    pub blockchain: String,
    /// If a blockchain has a specific chain-id or network identifier, it should go in this field. It is up to the client to determine which network-specific identifier is mainnet or testnet.
    #[serde(rename = "network")]
    pub network: String,
    #[serde(
        rename = "sub_network_identifier",
        skip_serializing_if = "Option::is_none"
    )]
    pub sub_network_identifier: Option<crate::SubNetworkIdentifier>,
}

impl NetworkIdentifier {
    /// The network_identifier specifies which network a particular object is associated with.
    pub fn new(blockchain: String, network: String) -> NetworkIdentifier {
        NetworkIdentifier {
            blockchain,
            network,
            sub_network_identifier: None,
        }
    }
}
