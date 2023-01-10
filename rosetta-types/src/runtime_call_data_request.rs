use serde_json::Value;

use crate::NetworkIdentifier;

/// AccountFaucetRequest : AccountFaucetRequest is sent for faucet on an account.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RuntimeCallDataRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: NetworkIdentifier,
    #[serde(rename = "pallet_name")]
    pub pallet_name: String,
    #[serde(rename = "call_name")]
    pub call_name: String,
    #[serde(rename = "params")]
    pub params: Value,
    #[serde(rename = "nonce")]
    pub nonce: u64,
    #[serde(rename = "sender_address")]
    pub sender_address: String,
}

impl RuntimeCallDataRequest {
    /// AccountCoinsRequest is utilized to make a request on the /account/coins endpoint.
    pub fn new(
        network_identifier: NetworkIdentifier,
        pallet_name: String,
        call_name: String,
        params: Value,
        nonce: u64,
        sender_address: String,
    ) -> RuntimeCallDataRequest {
        RuntimeCallDataRequest {
            network_identifier,
            pallet_name,
            call_name,
            params,
            nonce,
            sender_address,
        }
    }
}
