use serde_json;
use types::indicator::IndicatorConfig;

fn main() {
    let indicator_type = "ht_dcperiod";
    let config = IndicatorConfig::new(
        indicator_type,
        &serde_json::json!({
            "priceSource": "CLOSE",
        }),
    )
    .unwrap();
    println!("config: {:?}", config);
}
