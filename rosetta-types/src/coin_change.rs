/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `CoinChange` : `CoinChange` is used to represent a change in state of a some coin identified by a `coin_identifier`. This object is part of the Operation model and must be populated for UTXO-based blockchains.  Coincidentally, this abstraction of UTXOs allows for supporting both account-based transfers and UTXO-based transfers on the same blockchain (when a transfer is account-based, don't populate this model).
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CoinChange {
    #[serde(rename = "coin_identifier")]
    pub coin_identifier: crate::CoinIdentifier,
    #[serde(rename = "coin_action")]
    pub coin_action: crate::CoinAction,
}

impl CoinChange {
    /// `CoinChange` is used to represent a change in state of a some coin identified by a `coin_identifier`. This object is part of the Operation model and must be populated for UTXO-based blockchains.  Coincidentally, this abstraction of UTXOs allows for supporting both account-based transfers and UTXO-based transfers on the same blockchain (when a transfer is account-based, don't populate this model).
    #[must_use]
    pub const fn new(
        coin_identifier: crate::CoinIdentifier,
        coin_action: crate::CoinAction,
    ) -> Self {
        Self {
            coin_identifier,
            coin_action,
        }
    }
}
