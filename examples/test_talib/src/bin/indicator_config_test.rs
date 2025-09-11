use serde_json;
use star_river_core::indicator::IndicatorConfig;

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
