use std::str::FromStr;
use types::cache::key::IndicatorKey;

fn main() {
    let indicator_key_str = "indicator|metatrader5(Exness-MT5Trial5)|BTCUSDm|1m|ma(time_period=14 ma_type=SMA price_source=CLOSE)|2025-07-25|2025-07-26";

    let indicator_key = IndicatorKey::from_str(indicator_key_str).unwrap();
    println!("{:?}", indicator_key);
}
