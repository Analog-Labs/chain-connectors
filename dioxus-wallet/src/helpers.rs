use rosetta_client::Chain;

pub fn convert_to_lowest_unit(amount: u128, chain: Chain) -> u128 {
    let base: u128 = 10;
    amount * base.pow(chain.config().currency.decimals)
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub fn slice_string(s: &str, from: &str) -> String {
    let splitted_string = s.split_once(from);
    let sliced_string = match splitted_string {
        Some(s) => s.1,
        None => s,
    };
    return sliced_string.to_string();
}
