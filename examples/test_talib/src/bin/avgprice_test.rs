use types::indicator::IndicatorConfig;
use serde_json;

fn main() {
    // 测试全大写指标 AVGPRICE
    let config = IndicatorConfig::new("avgprice", &serde_json::json!({})).unwrap();
    println!("AVGPRICE配置对象: {:?}", config);
    println!("AVGPRICE ToString输出: {}", config.to_string());
    
    // 测试驼峰指标 HtDcperiod
    let config2 = IndicatorConfig::new("ht_dcperiod", &serde_json::json!({
        "priceSource": "CLOSE",
    })).unwrap();
    println!("HtDcperiod配置对象: {:?}", config2);
    println!("HtDcperiod ToString输出: {}", config2.to_string());
}