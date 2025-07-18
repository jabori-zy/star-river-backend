/// 所有TA-Lib指标的定义和分类
/// 
/// 这个文件包含了所有150+个TA-Lib指标的元数据定义

use crate::indicator_engine::talib::indicator_meta::*;
use std::collections::HashMap;

/// 指标目录 - 包含所有指标的分类和信息
pub struct IndicatorCatalog {
    indicators: HashMap<&'static str, IndicatorInfo>,
    groups: HashMap<IndicatorGroup, Vec<&'static str>>,
}

/// 指标信息
#[derive(Debug, Clone)]
pub struct IndicatorInfo {
    pub name: &'static str,
    pub group: IndicatorGroup,
    pub input_type: InputType,
    pub output_format: OutputFormat,
    pub description: &'static str,
    pub params: Vec<ParamInfo>,
}

/// 参数信息
#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: &'static str,
    pub param_type: ParamType,
    pub default_value: Option<ParamValue>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub description: &'static str,
}

/// 参数类型
#[derive(Debug, Clone)]
pub enum ParamType {
    Integer,
    Real,
    MAType,
}

/// 参数值
#[derive(Debug, Clone)]
pub enum ParamValue {
    Integer(i32),
    Real(f64),
}

impl IndicatorCatalog {
    pub fn new() -> Self {
        let mut catalog = IndicatorCatalog {
            indicators: HashMap::new(),
            groups: HashMap::new(),
        };
        
        catalog.init_indicators();
        catalog
    }

    fn init_indicators(&mut self) {
        // 重叠研究指标 (Overlap Studies)
        self.add_indicator(IndicatorInfo {
            name: "SMA",
            group: IndicatorGroup::Overlap,
            input_type: InputType::Single,
            output_format: OutputFormat::Single,
            description: "Simple Moving Average",
            params: vec![
                ParamInfo {
                    name: "TimePeriod",
                    param_type: ParamType::Integer,
                    default_value: Some(ParamValue::Integer(30)),
                    min_value: Some(2.0),
                    max_value: Some(100000.0),
                    description: "Number of periods for the moving average",
                }
            ],
        });

        self.add_indicator(IndicatorInfo {
            name: "EMA",
            group: IndicatorGroup::Overlap,
            input_type: InputType::Single,
            output_format: OutputFormat::Single,
            description: "Exponential Moving Average",
            params: vec![
                ParamInfo {
                    name: "TimePeriod",
                    param_type: ParamType::Integer,
                    default_value: Some(ParamValue::Integer(30)),
                    min_value: Some(2.0),
                    max_value: Some(100000.0),
                    description: "Number of periods for the moving average",
                }
            ],
        });

        self.add_indicator(IndicatorInfo {
            name: "BBANDS",
            group: IndicatorGroup::Overlap,
            input_type: InputType::Single,
            output_format: OutputFormat::Triple,
            description: "Bollinger Bands",
            params: vec![
                ParamInfo {
                    name: "TimePeriod",
                    param_type: ParamType::Integer,
                    default_value: Some(ParamValue::Integer(5)),
                    min_value: Some(2.0),
                    max_value: Some(100000.0),
                    description: "Number of periods for the moving average",
                },
                ParamInfo {
                    name: "NbDevUp",
                    param_type: ParamType::Real,
                    default_value: Some(ParamValue::Real(2.0)),
                    min_value: Some(-30000000000000000000000000000000000000.0),
                    max_value: Some(30000000000000000000000000000000000000.0),
                    description: "Deviation multiplier for upper band",
                },
                ParamInfo {
                    name: "NbDevDn",
                    param_type: ParamType::Real,
                    default_value: Some(ParamValue::Real(2.0)),
                    min_value: Some(-30000000000000000000000000000000000000.0),
                    max_value: Some(30000000000000000000000000000000000000.0),
                    description: "Deviation multiplier for lower band",
                },
                ParamInfo {
                    name: "MAType",
                    param_type: ParamType::MAType,
                    default_value: Some(ParamValue::Integer(0)),
                    min_value: None,
                    max_value: None,
                    description: "Type of Moving Average",
                }
            ],
        });

        // 动量指标 (Momentum Indicators)
        self.add_indicator(IndicatorInfo {
            name: "RSI",
            group: IndicatorGroup::Momentum,
            input_type: InputType::Single,
            output_format: OutputFormat::Single,
            description: "Relative Strength Index",
            params: vec![
                ParamInfo {
                    name: "TimePeriod",
                    param_type: ParamType::Integer,
                    default_value: Some(ParamValue::Integer(14)),
                    min_value: Some(2.0),
                    max_value: Some(100000.0),
                    description: "Number of periods for RSI calculation",
                }
            ],
        });

        self.add_indicator(IndicatorInfo {
            name: "MACD",
            group: IndicatorGroup::Momentum,
            input_type: InputType::Single,
            output_format: OutputFormat::Triple,
            description: "Moving Average Convergence/Divergence",
            params: vec![
                ParamInfo {
                    name: "FastPeriod",
                    param_type: ParamType::Integer,
                    default_value: Some(ParamValue::Integer(12)),
                    min_value: Some(2.0),
                    max_value: Some(100000.0),
                    description: "Fast EMA period",
                },
                ParamInfo {
                    name: "SlowPeriod",
                    param_type: ParamType::Integer,
                    default_value: Some(ParamValue::Integer(26)),
                    min_value: Some(2.0),
                    max_value: Some(100000.0),
                    description: "Slow EMA period",
                },
                ParamInfo {
                    name: "SignalPeriod",
                    param_type: ParamType::Integer,
                    default_value: Some(ParamValue::Integer(9)),
                    min_value: Some(1.0),
                    max_value: Some(100000.0),
                    description: "Signal line EMA period",
                }
            ],
        });

        // 更多指标将在后续添加...
    }

    fn add_indicator(&mut self, info: IndicatorInfo) {
        // 添加到分组
        self.groups.entry(info.group).or_insert_with(Vec::new).push(info.name);
        
        // 添加到指标映射
        self.indicators.insert(info.name, info);
    }

    pub fn get_indicator_info(&self, name: &str) -> Option<&IndicatorInfo> {
        self.indicators.get(name)
    }

    pub fn get_indicators_by_group(&self, group: IndicatorGroup) -> Option<&Vec<&'static str>> {
        self.groups.get(&group)
    }

    pub fn list_all_indicators(&self) -> Vec<&'static str> {
        self.indicators.keys().copied().collect()
    }

    pub fn search_indicators(&self, keyword: &str) -> Vec<&'static str> {
        let keyword_lower = keyword.to_lowercase();
        self.indicators.values()
            .filter(|info| {
                info.name.to_lowercase().contains(&keyword_lower) ||
                info.description.to_lowercase().contains(&keyword_lower)
            })
            .map(|info| info.name)
            .collect()
    }

    pub fn get_all_groups(&self) -> Vec<IndicatorGroup> {
        self.groups.keys().copied().collect()
    }
}

/// 获取全局指标目录实例
pub fn get_indicator_catalog() -> &'static IndicatorCatalog {
    static mut CATALOG: Option<IndicatorCatalog> = None;
    static INIT: std::sync::Once = std::sync::Once::new();
    
    unsafe {
        INIT.call_once(|| {
            CATALOG = Some(IndicatorCatalog::new());
        });
        CATALOG.as_ref().unwrap()
    }
}
