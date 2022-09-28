/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// AccountCoinsResponse : AccountCoinsResponse is returned on the /account/coins endpoint and includes all unspent Coins owned by an AccountIdentifier.

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AccountCoinsResponse {
    #[serde(rename = "block_identifier")]
    pub block_identifier: crate::BlockIdentifier,
    /// If a blockchain is UTXO-based, all unspent Coins owned by an account_identifier should be returned alongside the balance. It is highly recommended to populate this field so that users of the Rosetta API implementation don't need to maintain their own indexer to track their UTXOs.
    #[serde(rename = "coins")]
    pub coins: Vec<crate::Coin>,
    /// Account-based blockchains that utilize a nonce or sequence number should include that number in the metadata. This number could be unique to the identifier or global across the account address.
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl AccountCoinsResponse {
    /// AccountCoinsResponse is returned on the /account/coins endpoint and includes all unspent Coins owned by an AccountIdentifier.
    pub fn new(
        block_identifier: crate::BlockIdentifier,
        coins: Vec<crate::Coin>,
    ) -> AccountCoinsResponse {
        AccountCoinsResponse {
            block_identifier,
            coins,
            metadata: None,
        }
    }
}
