/// AccountFaucetRequest : AccountFaucetRequest is sent for faucet on an account.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AccountFaucetRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: crate::NetworkIdentifier,
    #[serde(rename = "account_address")]
    pub account_address: String,
    #[serde(rename = "amount")]
    pub amount: u128,
}

impl AccountFaucetRequest {
    /// AccountCoinsRequest is utilized to make a request on the /account/coins endpoint.
    pub fn new(
        network_identifier: crate::NetworkIdentifier,
        account_address: String,
        amount: u128,
    ) -> AccountFaucetRequest {
        AccountFaucetRequest {
            network_identifier,
            account_address,
            amount,
        }
    }
}
