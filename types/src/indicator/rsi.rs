use crate::indicator::{PriceSource, IndicatorConfigTrait, Indicator, IndicatorTrait, MAType};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use serde_json::Value;



#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RSIConfig {
    pub period: i32,
    pub price_source: PriceSource,
}

impl ToString for RSIConfig {
    fn to_string(&self) -> String {
        format!("rsi(period={} source={})", self.period, self.price_source)
    }
}


impl FromStr for RSIConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::indicator::utils::*;
        
        let (_name, params) = parse_indicator_config_from_str(s)?;
        let period = get_required_i32_param(&params, "period")?;
        let price_source = get_required_parsed_param::<PriceSource>(&params, "source")?;
        Ok(RSIConfig { period, price_source })
    }
}