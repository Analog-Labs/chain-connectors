/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// Coin : Coin contains its unique identifier and the amount it represents.

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Coin {
    #[serde(rename = "coin_identifier")]
    pub coin_identifier: crate::CoinIdentifier,
    #[serde(rename = "amount")]
    pub amount: crate::Amount,
}

impl Coin {
    /// Coin contains its unique identifier and the amount it represents.
    #[must_use] pub fn new(coin_identifier: crate::CoinIdentifier, amount: crate::Amount) -> Self {
        Self {
            coin_identifier,
            amount,
        }
    }
}
