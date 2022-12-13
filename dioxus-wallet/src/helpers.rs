use rosetta_client::Chain;

pub fn convert_to_lowest_unit(amount: u128, chain: Chain) -> u128 {
    match chain {
        Chain::Eth => amount * 1000000000000000000, // i.e 1 eth = 1000000000000000000 wei
        Chain::Btc => amount * 100000000,           // 1 btc = 100000000 satoshi
        Chain::Dot => amount * 10000000000,         // 1 dot = 10000000000 Plank
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub fn slice_string(s: &str, from: u8) -> String {
    let slice_index = get_word_index(s.to_string(), from);
    let sliced_string = &s[slice_index..];
    return sliced_string.to_string();
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub fn get_word_index(s: String, from: u8) -> usize {
    let string_bytes = s.as_bytes();
    for (i, &item) in string_bytes.iter().enumerate() {
        if item == from {
            return i + 1;
        }
    }
    return 0;
}
