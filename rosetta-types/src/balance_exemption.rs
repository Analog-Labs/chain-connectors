/*
 * Rosetta
 *
 * Build Once. Integrate Your Blockchain Everywhere.
 *
 * The version of the OpenAPI document: 1.4.13
 *
 * Generated by: https://openapi-generator.tech
 */

/// `BalanceExemption` : `BalanceExemption` indicates that the balance for an exempt account could change without a corresponding Operation. This typically occurs with staking rewards, vesting balances, and Currencies with a dynamic supply.  Currently, it is possible to exempt an account from strict reconciliation by SubAccountIdentifier.Address or by Currency. This means that any account with SubAccountIdentifier.Address would be exempt or any balance of a particular Currency would be exempt, respectively.  `BalanceExemptions` should be used sparingly as they may introduce significant complexity for integrators that attempt to reconcile all account balance changes.  If your implementation relies on any `BalanceExemptions`, you MUST implement historical balance lookup (the ability to query an account balance at any `BlockIdentifier`).
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BalanceExemption {
    /// SubAccountAddress is the SubAccountIdentifier.Address that the BalanceExemption applies to (regardless of the value of SubAccountIdentifier.Metadata).
    #[serde(
        rename = "sub_account_address",
        skip_serializing_if = "Option::is_none"
    )]
    pub sub_account_address: Option<String>,
    #[serde(rename = "currency", skip_serializing_if = "Option::is_none")]
    pub currency: Option<crate::Currency>,
    #[serde(rename = "exemption_type", skip_serializing_if = "Option::is_none")]
    pub exemption_type: Option<crate::ExemptionType>,
}

impl BalanceExemption {
    /// `BalanceExemption` indicates that the balance for an exempt account could change without a corresponding Operation. This typically occurs with staking rewards, vesting balances, and Currencies with a dynamic supply.  Currently, it is possible to exempt an account from strict reconciliation by SubAccountIdentifier.Address or by Currency. This means that any account with SubAccountIdentifier.Address would be exempt or any balance of a particular Currency would be exempt, respectively.  `BalanceExemptions` should be used sparingly as they may introduce significant complexity for integrators that attempt to reconcile all account balance changes.  If your implementation relies on any `BalanceExemptions`, you MUST implement historical balance lookup (the ability to query an account balance at any `BlockIdentifier`).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sub_account_address: None,
            currency: None,
            exemption_type: None,
        }
    }
}
