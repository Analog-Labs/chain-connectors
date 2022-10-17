use crate::types::{
    AccountIdentifier, Amount, Coin, CoinAction, CoinChange, Currency, Operation,
    OperationIdentifier,
};
use anyhow::Result;

pub struct TransactionBuilder {
    ops: Vec<Operation>,
    input_amount: u128,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            ops: Vec::with_capacity(4),
            input_amount: 0,
        }
    }

    pub fn operations(&self) -> &[Operation] {
        &self.ops
    }

    pub fn input_amount(&self) -> u128 {
        self.input_amount
    }

    fn push_op(
        &mut self,
        op: &str,
        relate_previous: bool,
        account: &AccountIdentifier,
        amount: Amount,
        coin_change: Option<CoinChange>,
    ) {
        self.ops.push(Operation {
            operation_identifier: OperationIdentifier {
                index: self.ops.len() as _,
                network_index: None,
            },
            related_operations: if relate_previous {
                Some(vec![OperationIdentifier {
                    index: (self.ops.len() - 1) as _,
                    network_index: None,
                }])
            } else {
                None
            },
            r#type: op.into(),
            status: None,
            account: Some(account.clone()),
            amount: Some(amount),
            coin_change,
            metadata: None,
        });
    }

    fn inner_transfer(
        &mut self,
        op: &str,
        from: &AccountIdentifier,
        to: &AccountIdentifier,
        amount: Amount,
    ) {
        let neg_amount = Amount {
            value: format!("-{}", amount.value),
            currency: amount.currency.clone(),
            metadata: amount.metadata.clone(),
        };
        self.push_op(op, false, from, neg_amount, None);
        self.push_op(op, true, to, amount, None);
    }

    pub fn transfer(
        &mut self,
        from: &AccountIdentifier,
        to: &AccountIdentifier,
        amount: u128,
        currency: &Currency,
    ) {
        let amount = Amount {
            value: amount.to_string(),
            currency: currency.clone(),
            metadata: None,
        };
        self.inner_transfer("CALL", from, to, amount);
    }

    pub fn input(&mut self, account: &AccountIdentifier, coin: &Coin) -> Result<()> {
        let input_amount: u128 = coin.amount.value.parse()?;
        self.input_amount = self
            .input_amount
            .checked_add(input_amount)
            .ok_or_else(|| anyhow::anyhow!("input coins overflowed u128"))?;
        let neg_amount = Amount {
            value: format!("-{}", coin.amount.value),
            currency: coin.amount.currency.clone(),
            metadata: coin.amount.metadata.clone(),
        };
        let coin_change = Some(CoinChange {
            coin_action: CoinAction::Spent,
            coin_identifier: coin.coin_identifier.clone(),
        });
        self.push_op("INPUT", false, account, neg_amount, coin_change);
        Ok(())
    }

    pub fn output(&mut self, account: &AccountIdentifier, amount: u128, currency: &Currency) {
        let amount = Amount {
            value: amount.to_string(),
            currency: currency.clone(),
            metadata: None,
        };
        self.push_op("OUTPUT", false, account, amount, None);
    }

    pub fn pop(&mut self) {
        self.ops.pop();
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
