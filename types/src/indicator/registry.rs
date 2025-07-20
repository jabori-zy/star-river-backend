use std::collections::HashMap;
use super::talib_types::*;

/// 指标元数据（包含函数指针，用于运行时）
pub struct IndicatorMeta {
    pub name: &'static str,
    pub group: IndicatorGroup,
    pub input_type: InputType,
    pub output_format: OutputFormat,
    pub lookback_fn: fn(&[IndicatorParam]) -> i32,
    pub calculate_fn: fn(&IndicatorInput, &[IndicatorParam]) -> Result<IndicatorOutput, String>,
    pub description: &'static str,
    pub params: Vec<ParamMeta>,
}

/// 指标注册表
pub struct IndicatorRegistry {
    indicators: HashMap<&'static str, IndicatorMeta>,
    groups: HashMap<IndicatorGroup, Vec<&'static str>>,
}

impl IndicatorRegistry {
    pub fn new() -> Self {
        Self {
            indicators: HashMap::new(),
            groups: HashMap::new(),
        }
    }

    pub fn register(&mut self, meta: IndicatorMeta) {
        // 添加到分组索引
        self.groups
            .entry(meta.group)
            .or_insert_with(Vec::new)
            .push(meta.name);
        
        // 添加到主索引
        self.indicators.insert(meta.name, meta);
    }

    pub fn calculate(&self, name: &str, input: &IndicatorInput, params: &[IndicatorParam]) -> Result<IndicatorOutput, String> {
        let meta = self.indicators.get(name)
            .ok_or_else(|| format!("Unsupported indicator: {}", name))?;
        (meta.calculate_fn)(input, params)
    }

    pub fn calculate_single(&self, name: &str, data: &[f64], params: &[IndicatorParam]) -> Result<IndicatorOutput, String> {
        let input = IndicatorInput::Single(data.to_vec());
        self.calculate(name, &input, params)
    }

    pub fn lookback(&self, name: &str, params: &[IndicatorParam]) -> Result<i32, String> {
        let meta = self.indicators.get(name)
            .ok_or_else(|| format!("Unsupported indicator: {}", name))?;
        Ok((meta.lookback_fn)(params))
    }

    pub fn get_output_format(&self, name: &str) -> Result<OutputFormat, String> {
        let meta = self.indicators.get(name)
            .ok_or_else(|| format!("Unsupported indicator: {}", name))?;
        Ok(meta.output_format)
    }

    pub fn list_indicators(&self) -> Vec<&'static str> {
        self.indicators.keys().copied().collect()
    }

    pub fn get_indicators_by_group(&self, group: IndicatorGroup) -> Vec<&'static str> {
        self.groups.get(&group).cloned().unwrap_or_default()
    }

    pub fn search_indicators(&self, keyword: &str) -> Vec<&'static str> {
        let keyword_lower = keyword.to_lowercase();
        self.indicators.values()
            .filter(|meta| {
                meta.name.to_lowercase().contains(&keyword_lower) ||
                meta.description.to_lowercase().contains(&keyword_lower)
            })
            .map(|meta| meta.name)
            .collect()
    }

    pub fn get_indicator_info(&self, name: &str) -> Option<IndicatorInfo> {
        self.indicators.get(name).map(|meta| IndicatorInfo {
            name: meta.name.to_string(),
            description: meta.description.to_string(),
            group: meta.group,
            input_type: meta.input_type,
            output_format: meta.output_format,
            params: meta.params.clone(),
        })
    }

    pub fn get_indicator_metadata(&self, name: &str) -> Option<IndicatorMetaData> {
        self.indicators.get(name).map(|meta| IndicatorMetaData {
            name: meta.name.to_string(),
            group: meta.group,
            input_type: meta.input_type,
            output_format: meta.output_format,
            description: meta.description.to_string(),
            params: meta.params.clone(),
        })
    }
}

/// 全局指标注册表
static mut INDICATOR_REGISTRY: Option<IndicatorRegistry> = None;
static INIT: std::sync::Once = std::sync::Once::new();

pub fn get_indicator_registry() -> &'static IndicatorRegistry {
    unsafe {
        INIT.call_once(|| {
            INDICATOR_REGISTRY = Some(IndicatorRegistry::new());
        });
        INDICATOR_REGISTRY.as_ref().unwrap()
    }
}

pub fn get_indicator_registry_mut() -> &'static mut IndicatorRegistry {
    unsafe {
        INIT.call_once(|| {
            INDICATOR_REGISTRY = Some(IndicatorRegistry::new());
        });
        INDICATOR_REGISTRY.as_mut().unwrap()
    }
}

/// 初始化指标注册表的trait
pub trait IndicatorRegistryInit {
    fn init_indicators();
}

/// 指标计算trait
pub trait IndicatorCalculator {
    fn calculate(&self, input: &IndicatorInput, params: &[IndicatorParam]) -> Result<IndicatorOutput, String>;
    fn lookback(&self, params: &[IndicatorParam]) -> i32;
    fn get_name(&self) -> &'static str;
    fn get_description(&self) -> &'static str;
    fn get_group(&self) -> IndicatorGroup;
    fn get_input_type(&self) -> InputType;
    fn get_output_format(&self) -> OutputFormat;
    fn get_params(&self) -> Vec<ParamMeta>;
}

/// 便利宏，用于快速创建指标元数据
#[macro_export]
macro_rules! create_indicator_meta {
    ($name:expr, $group:expr, $input:expr, $output:expr, $desc:expr, $lookback:expr, $calculate:expr, $params:expr) => {
        IndicatorMeta {
            name: $name,
            group: $group,
            input_type: $input,
            output_format: $output,
            description: $desc,
            lookback_fn: $lookback,
            calculate_fn: $calculate,
            params: $params,
        }
    };
}

/// 便利宏，用于创建参数元数据
#[macro_export]
macro_rules! create_param_meta {
    ($name:expr, $type:expr, $desc:expr) => {
        ParamMeta {
            name: $name.to_string(),
            param_type: $type,
            default_value: None,
            min_value: None,
            max_value: None,
            description: $desc.to_string(),
        }
    };
    ($name:expr, $type:expr, $desc:expr, $default:expr) => {
        ParamMeta {
            name: $name.to_string(),
            param_type: $type,
            default_value: Some($default),
            min_value: None,
            max_value: None,
            description: $desc.to_string(),
        }
    };
    ($name:expr, $type:expr, $desc:expr, $default:expr, $min:expr, $max:expr) => {
        ParamMeta {
            name: $name.to_string(),
            param_type: $type,
            default_value: Some($default),
            min_value: Some($min),
            max_value: Some($max),
            description: $desc.to_string(),
        }
    };
}
