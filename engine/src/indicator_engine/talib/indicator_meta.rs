use crate::indicator_engine::talib_error::TalibError;
use std::collections::HashMap;

/// 指标参数类型 - 支持所有TA-Lib参数
#[derive(Debug, Clone)]
pub enum IndicatorParam {
    // 时间周期参数
    TimePeriod(i32),
    FastPeriod(i32),
    SlowPeriod(i32),
    SignalPeriod(i32),
    TimePeriod1(i32),
    TimePeriod2(i32),
    TimePeriod3(i32),

    // 数值参数
    DevUp(f64),
    DevDown(f64),
    NbDev(f64),
    Deviation(f64),
    Acceleration(f64),
    Maximum(f64),
    Minimum(f64),

    // MA类型参数
    MAType(i32),
    FastMAType(i32),
    SlowMAType(i32),

    // 特殊参数
    Penetration(f64),
    VFactor(f64),
    FastLimit(f64),
    SlowLimit(f64),

    // K和D参数（用于随机指标）
    FastKPeriod(i32),
    SlowKPeriod(i32),
    SlowKMAType(i32),
    SlowDPeriod(i32),
    SlowDMAType(i32),

    // 其他常用参数
    Multiplier(f64),
    Offset(f64),
    StartValue(f64),
    EndValue(f64),

    // 兼容旧版本
    Period(i32),
}

impl IndicatorParam {
    // 通用的参数提取方法
    pub fn as_i32(&self) -> i32 {
        match self {
            IndicatorParam::TimePeriod(v) | IndicatorParam::Period(v) |
            IndicatorParam::FastPeriod(v) | IndicatorParam::SlowPeriod(v) |
            IndicatorParam::SignalPeriod(v) | IndicatorParam::TimePeriod1(v) |
            IndicatorParam::TimePeriod2(v) | IndicatorParam::TimePeriod3(v) |
            IndicatorParam::MAType(v) | IndicatorParam::FastMAType(v) |
            IndicatorParam::SlowMAType(v) | IndicatorParam::FastKPeriod(v) |
            IndicatorParam::SlowKPeriod(v) | IndicatorParam::SlowKMAType(v) |
            IndicatorParam::SlowDPeriod(v) | IndicatorParam::SlowDMAType(v) => *v,
            _ => panic!("Parameter is not an integer type: {:?}", self),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            IndicatorParam::DevUp(v) | IndicatorParam::DevDown(v) |
            IndicatorParam::NbDev(v) | IndicatorParam::Deviation(v) |
            IndicatorParam::Acceleration(v) | IndicatorParam::Maximum(v) |
            IndicatorParam::Minimum(v) | IndicatorParam::Penetration(v) |
            IndicatorParam::VFactor(v) | IndicatorParam::FastLimit(v) |
            IndicatorParam::SlowLimit(v) | IndicatorParam::Multiplier(v) |
            IndicatorParam::Offset(v) | IndicatorParam::StartValue(v) |
            IndicatorParam::EndValue(v) => *v,
            _ => panic!("Parameter is not a float type: {:?}", self),
        }
    }

    // 兼容旧版本的方法
    pub fn as_period(&self) -> i32 {
        match self {
            IndicatorParam::Period(p) | IndicatorParam::TimePeriod(p) => *p,
            _ => panic!("Expected Period parameter"),
        }
    }

    pub fn as_dev_up(&self) -> f64 {
        match self {
            IndicatorParam::DevUp(d) => *d,
            _ => panic!("Expected DevUp parameter"),
        }
    }

    pub fn as_dev_down(&self) -> f64 {
        match self {
            IndicatorParam::DevDown(d) => *d,
            _ => panic!("Expected DevDown parameter"),
        }
    }

    pub fn as_ma_type(&self) -> i32 {
        match self {
            IndicatorParam::MAType(t) => *t,
            _ => panic!("Expected MAType parameter"),
        }
    }

    pub fn as_fast_period(&self) -> i32 {
        match self {
            IndicatorParam::FastPeriod(p) => *p,
            _ => panic!("Expected FastPeriod parameter"),
        }
    }

    pub fn as_slow_period(&self) -> i32 {
        match self {
            IndicatorParam::SlowPeriod(p) => *p,
            _ => panic!("Expected SlowPeriod parameter"),
        }
    }

    pub fn as_signal_period(&self) -> i32 {
        match self {
            IndicatorParam::SignalPeriod(p) => *p,
            _ => panic!("Expected SignalPeriod parameter"),
        }
    }
}

/// 指标输入数据类型
#[derive(Debug, Clone)]
pub enum IndicatorInput {
    Single(Vec<f64>),                                           // inReal
    Dual(Vec<f64>, Vec<f64>),                                  // inReal0, inReal1
    OHLC(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>),             // Open, High, Low, Close
    OHLCV(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>),  // + Volume
    HLC(Vec<f64>, Vec<f64>, Vec<f64>),                        // High, Low, Close
    HL(Vec<f64>, Vec<f64>),                                   // High, Low
}

/// 指标输出格式
#[derive(Debug, Clone)]
pub enum IndicatorOutput {
    Single(Vec<f64>),           // 单一输出，如SMA
    Dual(Vec<f64>, Vec<f64>),   // 双输出，如AROON
    Triple(Vec<f64>, Vec<f64>, Vec<f64>), // 三重输出，如MACD, BBands
    Quad(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), // 四重输出，如STOCH
}

/// 输出格式类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Single,
    Dual,
    Triple,
    Quad,
}

/// 输入数据类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputType {
    Single,     // 单一数据输入
    Dual,       // 双数据输入
    OHLC,       // OHLC数据输入
    OHLCV,      // OHLCV数据输入
    HLC,        // HLC数据输入
    HL,         // HL数据输入
}

/// 指标元数据
pub struct IndicatorMeta {
    pub name: &'static str,
    pub group: IndicatorGroup,
    pub input_type: InputType,
    pub output_format: OutputFormat,
    pub lookback_fn: fn(&[IndicatorParam]) -> i32,
    pub calculate_fn: fn(&IndicatorInput, &[IndicatorParam]) -> Result<IndicatorOutput, TalibError>,
    pub description: &'static str,
}

/// 指标分组
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndicatorGroup {
    Overlap,        // 重叠研究 (SMA, EMA, BBANDS等)
    Momentum,       // 动量指标 (RSI, MACD, STOCH等)
    Volume,         // 成交量指标 (AD, OBV, MFI等)
    Volatility,     // 波动率指标 (ATR, NATR等)
    Price,          // 价格变换 (AVGPRICE, MEDPRICE等)
    Cycle,          // 周期指标 (HT系列)
    Pattern,        // 形态识别 (CDL系列)
    Statistic,      // 统计函数 (CORREL, BETA等)
    Math,           // 数学运算 (ADD, SUB, SIN等)
}

/// 指标注册表
pub struct IndicatorRegistry {
    indicators: HashMap<&'static str, IndicatorMeta>,
}

impl IndicatorRegistry {
    pub fn new() -> Self {
        Self {
            indicators: HashMap::new(),
        }
    }

    pub fn register(&mut self, meta: IndicatorMeta) {
        self.indicators.insert(meta.name, meta);
    }

    pub fn calculate(&self, name: &str, input: &IndicatorInput, params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
        let meta = self.indicators.get(name)
            .ok_or_else(|| TalibError::UnsupportedIndicator { name: name.to_string() })?;
        (meta.calculate_fn)(input, params)
    }

    pub fn calculate_single(&self, name: &str, data: &[f64], params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
        let input = IndicatorInput::Single(data.to_vec());
        self.calculate(name, &input, params)
    }

    pub fn lookback(&self, name: &str, params: &[IndicatorParam]) -> Result<i32, TalibError> {
        let meta = self.indicators.get(name)
            .ok_or_else(|| TalibError::UnsupportedIndicator { name: name.to_string() })?;
        Ok((meta.lookback_fn)(params))
    }

    pub fn get_output_format(&self, name: &str) -> Result<OutputFormat, TalibError> {
        let meta = self.indicators.get(name)
            .ok_or_else(|| TalibError::UnsupportedIndicator { name: name.to_string() })?;
        Ok(meta.output_format)
    }

    pub fn list_indicators(&self) -> Vec<&'static str> {
        self.indicators.keys().copied().collect()
    }

    pub fn get_indicators_by_group(&self, group: IndicatorGroup) -> Vec<&'static str> {
        self.indicators.values()
            .filter(|meta| meta.group == group)
            .map(|meta| meta.name)
            .collect()
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

    pub fn get_indicator_info(&self, name: &str) -> Option<(&IndicatorGroup, &InputType, &OutputFormat, &str)> {
        self.indicators.get(name)
            .map(|meta| (&meta.group, &meta.input_type, &meta.output_format, meta.description))
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

// 为IndicatorOutput添加便利方法
impl IndicatorOutput {
    pub fn as_single(&self) -> Result<&Vec<f64>, TalibError> {
        match self {
            IndicatorOutput::Single(data) => Ok(data),
            _ => Err(TalibError::GenericCalculationError {
                error: "Expected single output".to_string()
            }),
        }
    }

    pub fn as_dual(&self) -> Result<(&Vec<f64>, &Vec<f64>), TalibError> {
        match self {
            IndicatorOutput::Dual(data1, data2) => Ok((data1, data2)),
            _ => Err(TalibError::GenericCalculationError {
                error: "Expected dual output".to_string()
            }),
        }
    }

    pub fn as_triple(&self) -> Result<(&Vec<f64>, &Vec<f64>, &Vec<f64>), TalibError> {
        match self {
            IndicatorOutput::Triple(data1, data2, data3) => Ok((data1, data2, data3)),
            _ => Err(TalibError::GenericCalculationError {
                error: "Expected triple output".to_string()
            }),
        }
    }

    pub fn as_quad(&self) -> Result<(&Vec<f64>, &Vec<f64>, &Vec<f64>, &Vec<f64>), TalibError> {
        match self {
            IndicatorOutput::Quad(data1, data2, data3, data4) => Ok((data1, data2, data3, data4)),
            _ => Err(TalibError::GenericCalculationError {
                error: "Expected quad output".to_string()
            }),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            IndicatorOutput::Single(data) => data.len(),
            IndicatorOutput::Dual(data1, _) => data1.len(),
            IndicatorOutput::Triple(data1, _, _) => data1.len(),
            IndicatorOutput::Quad(data1, _, _, _) => data1.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
