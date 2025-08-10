use types::indicator::IndicatorConfig;
use serde_json;

fn main() {
    let config = IndicatorConfig::new("ht_dcperiod", &serde_json::json!({
        "priceSource": "CLOSE",
    })).unwrap();
    
    println!("配置对象: {:?}", config);
    println!("ToString输出: {}", config.to_string());
}