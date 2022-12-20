use fraction::BigDecimal;
use fraction::ToPrimitive;
use futures::Future;
use rosetta_client::Chain;
use std::rc::Rc;

pub fn convert_to_lowest_unit(amount: BigDecimal, chain: Chain) -> u128 {
    let base: u128 = 10;
    BigDecimal::to_u128(&(amount * base.pow(chain.config().currency.decimals).into())).unwrap()
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
