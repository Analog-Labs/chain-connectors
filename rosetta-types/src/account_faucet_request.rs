use crate::{AccountIdentifier, NetworkIdentifier};

/// `AccountFaucetRequest` : `AccountFaucetRequest` is sent for faucet on an account.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AccountFaucetRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: NetworkIdentifier,
    #[serde(rename = "account_identifier")]
    pub account_identifier: AccountIdentifier,
    #[serde(rename = "faucet_parameter")]
    pub faucet_parameter: u128,
}

impl AccountFaucetRequest {
    /// `AccountCoinsRequest` is utilized to make a request on the /account/coins endpoint.
    #[must_use]
    pub const fn new(
        network_identifier: NetworkIdentifier,
        account_identifier: AccountIdentifier,
        faucet_parameter: u128,
    ) -> Self {
        Self { network_identifier, account_identifier, faucet_parameter }
    }
}
