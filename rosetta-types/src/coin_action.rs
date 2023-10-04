/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `CoinAction` : `CoinActions` are different state changes that a Coin can undergo. When a Coin is created, it is `coin_created`. When a Coin is spent, it is `coin_spent`. It is assumed that a single Coin cannot be created or spent more than once.
/// `CoinActions` are different state changes that a Coin can undergo. When a Coin is created, it is `coin_created`. When a Coin is spent, it is `coin_spent`. It is assumed that a single Coin cannot be created or spent more than once.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum CoinAction {
    #[serde(rename = "coin_created")]
    Created,
    #[serde(rename = "coin_spent")]
    Spent,
}

impl ToString for CoinAction {
    fn to_string(&self) -> String {
        match self {
            Self::Created => String::from("coin_created"),
            Self::Spent => String::from("coin_spent"),
        }
    }
}

impl Default for CoinAction {
    fn default() -> Self {
        Self::Created
    }
}
