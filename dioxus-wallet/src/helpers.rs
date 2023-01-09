use anyhow::Result;
use fraction::BigDecimal;
use fraction::BigUint;
use fraction::ToPrimitive;
use futures::Future;
use rosetta_client::Chain;
use std::rc::Rc;

pub fn convert_to_lowest_unit(amount: BigDecimal, chain: Chain) -> u128 {
    let base: u128 = 10;
    BigDecimal::to_u128(&(amount * base.pow(chain.config().currency.decimals).into())).unwrap()
}
pub fn convert_to_highest_unit(amount: String, chain: Chain) -> Result<String> {
    let value = BigUint::parse_bytes(amount.as_bytes(), 10)
        .ok_or_else(|| anyhow::anyhow!("invalid amount {:?}", amount))?;
    let decimals = BigUint::pow(&10u32.into(), chain.config().currency.decimals);
    let value = BigDecimal::from(value) / BigDecimal::from(decimals);
    Ok(format!("{:.256} {}", value, chain.config().currency.symbol))
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub fn slice_string(s: &str, from: &str) -> String {
    let sliced_string = if let Some((_, s)) = s.split_once(from) {
        s
    } else {
        s
    };
    return sliced_string.to_string();
}

pub async fn display_loader(loader: Rc<dyn Fn(bool)>, future: impl Future<Output = ()>) {
    loader(true);
    future.await;
    loader(false);
}
