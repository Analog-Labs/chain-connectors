pub mod unit_converter {
    use rosetta_client::Chain;

    pub fn convert_to_lowest(amount: u128, chain: Chain) -> u128 {
        match chain {
            Chain::Eth => amount * 1000000000000000000, // i.e 1 eth = 1000000000000000000 wei
            Chain::Btc => amount * 100000000,           // 1 btc = 100000000 satoshi
            Chain::Dot => amount * 10000000000,         // 1 dot = 10000000000 Plank
        }
    }
}
