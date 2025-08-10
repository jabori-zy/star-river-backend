use types::indicator::IndicatorConfig;
use serde_json;


fn main() {
    let indicator_type = "ht_dcperiod";
    let config = IndicatorConfig::new(indicator_type, &serde_json::json!({
        "priceSource": "CLOSE",
    })).unwrap();
    println!("config: {:?}", config);
}