use types::indicator::IndicatorConfig;
use serde_json;

fn main() {
    // 你提供的 JSON 数据
    let json_data = r#"{
        "indicatorConfig": {
            "maType": "SMA",
            "priceSourc": "CLOSE",
            "timePeriod": 14
        },
        "indicatorId": 1,
        "indicatorType": "ma",
        "outputHandleId": "indicator_node_3_output1",
        "value": {
            "ma": 0,
            "timestamp": 0
        }
    }"#;

    // 尝试解析 indicatorConfig 部分
    let parsed: serde_json::Value = serde_json::from_str(json_data).unwrap();
    let indicator_config_json = &parsed["indicatorConfig"];
    let indicator_type = parsed["indicatorType"].as_str().unwrap();
    
    println!("指标类型: {}", indicator_type);
    println!("指标配置: {}", serde_json::to_string_pretty(indicator_config_json).unwrap());
    
    // 尝试创建 IndicatorConfig
    match IndicatorConfig::new(indicator_type, indicator_config_json) {
        Ok(config) => println!("成功创建配置: {:?}", config),
        Err(e) => println!("失败: {}", e),
    }
} 