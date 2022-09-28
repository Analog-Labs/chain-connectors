/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// AccountBalanceRequest : An AccountBalanceRequest is utilized to make a balance request on the /account/balance endpoint. If the block_identifier is populated, a historical balance query should be performed.

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AccountBalanceRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: crate::NetworkIdentifier,
    #[serde(rename = "account_identifier")]
    pub account_identifier: crate::AccountIdentifier,
    #[serde(rename = "block_identifier", skip_serializing_if = "Option::is_none")]
    pub block_identifier: Option<crate::PartialBlockIdentifier>,
    /// In some cases, the caller may not want to retrieve all available balances for an AccountIdentifier. If the currencies field is populated, only balances for the specified currencies will be returned. If not populated, all available balances will be returned.
    #[serde(rename = "currencies", skip_serializing_if = "Option::is_none")]
    pub currencies: Option<Vec<crate::Currency>>,
}

impl AccountBalanceRequest {
    /// An AccountBalanceRequest is utilized to make a balance request on the /account/balance endpoint. If the block_identifier is populated, a historical balance query should be performed.
    pub fn new(
        network_identifier: crate::NetworkIdentifier,
        account_identifier: crate::AccountIdentifier,
    ) -> AccountBalanceRequest {
        AccountBalanceRequest {
            network_identifier,
            account_identifier,
            block_identifier: None,
            currencies: None,
        }
    }
}
