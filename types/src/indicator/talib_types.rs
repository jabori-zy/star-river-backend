use serde::{Deserialize, Serialize};
use crate::indicator::{PriceSource, MAType};

/// 指标参数类型 - 支持所有TA-Lib参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    MAType(MAType),
    FastMAType(MAType),
    SlowMAType(MAType),

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

    // 特殊类型参数
    PriceSource(PriceSource),
}

impl IndicatorParam {
    // 通用的参数提取方法
    pub fn as_i32(&self) -> Result<i32, String> {
        Ok(match self {
            IndicatorParam::TimePeriod(v) | IndicatorParam::FastPeriod(v) | 
            IndicatorParam::SlowPeriod(v) |
            IndicatorParam::SignalPeriod(v) | IndicatorParam::TimePeriod1(v) |
            IndicatorParam::TimePeriod2(v) | IndicatorParam::TimePeriod3(v) |
            IndicatorParam::FastKPeriod(v) |
            IndicatorParam::SlowKPeriod(v) | IndicatorParam::SlowKMAType(v) |
            IndicatorParam::SlowDPeriod(v) | IndicatorParam::SlowDMAType(v) => *v,
            _ => return Err(format!("Parameter is not an integer type: {:?}", self)),
        })
    }

    pub fn as_f64(&self) -> Result<f64, String> {
        Ok(match self {
            IndicatorParam::DevUp(v) | IndicatorParam::DevDown(v) |
            IndicatorParam::NbDev(v) | IndicatorParam::Deviation(v) |
            IndicatorParam::Acceleration(v) | IndicatorParam::Maximum(v) |
            IndicatorParam::Minimum(v) | IndicatorParam::Penetration(v) |
            IndicatorParam::VFactor(v) | IndicatorParam::FastLimit(v) |
            IndicatorParam::SlowLimit(v) | IndicatorParam::Multiplier(v) |
            IndicatorParam::Offset(v) | IndicatorParam::StartValue(v) |
            IndicatorParam::EndValue(v) => *v,
            _ => return Err(format!("Parameter is not a float type: {:?}", self)),
            }
        )
    }

    // 返回Result版本的提取方法（更安全的版本）
    pub fn try_as_time_period(&self) -> Result<i32, String> {
        match self {
            IndicatorParam::TimePeriod(p) => Ok(*p),
            _ => Err(format!("Expected TimePeriod parameter, got {:?}", self)),
        }
    }

    pub fn try_as_fast_period(&self) -> Result<i32, String> {
        match self {
            IndicatorParam::FastPeriod(p) => Ok(*p),
            _ => Err(format!("Expected FastPeriod parameter, got {:?}", self)),
        }
    }

    pub fn try_as_slow_period(&self) -> Result<i32, String> {
        match self {
            IndicatorParam::SlowPeriod(p) => Ok(*p),
            _ => Err(format!("Expected SlowPeriod parameter, got {:?}", self)),
        }
    }

    pub fn try_as_signal_period(&self) -> Result<i32, String> {
        match self {
            IndicatorParam::SignalPeriod(p) => Ok(*p),
            _ => Err(format!("Expected SignalPeriod parameter, got {:?}", self)),
        }
    }

    pub fn try_as_dev_up(&self) -> Result<f64, String> {
        match self {
            IndicatorParam::DevUp(d) => Ok(*d),
            _ => Err(format!("Expected DevUp parameter, got {:?}", self)),
        }
    }

    pub fn try_as_dev_down(&self) -> Result<f64, String> {
        match self {
            IndicatorParam::DevDown(d) => Ok(*d),
            _ => Err(format!("Expected DevDown parameter, got {:?}", self)),
        }
    }

    pub fn try_as_ma_type(&self) -> Result<MAType, String> {
        match self {
            IndicatorParam::MAType(t) => Ok(t.clone()),
            _ => Err(format!("Expected MAType parameter, got {:?}", self)),
        }
    }

    // 检查参数类型的方法
    pub fn is_integer_param(&self) -> bool {
        matches!(self,
            IndicatorParam::TimePeriod(_) |
            IndicatorParam::FastPeriod(_) | IndicatorParam::SlowPeriod(_) |
            IndicatorParam::SignalPeriod(_) | IndicatorParam::TimePeriod1(_) |
            IndicatorParam::TimePeriod2(_) | IndicatorParam::TimePeriod3(_) |
            IndicatorParam::MAType(_) | IndicatorParam::FastMAType(_) |
            IndicatorParam::SlowMAType(_) | IndicatorParam::FastKPeriod(_) |
            IndicatorParam::SlowKPeriod(_) | IndicatorParam::SlowKMAType(_) |
            IndicatorParam::SlowDPeriod(_) | IndicatorParam::SlowDMAType(_)
        )
    }

    pub fn is_float_param(&self) -> bool {
        matches!(self,
            IndicatorParam::DevUp(_) | IndicatorParam::DevDown(_) |
            IndicatorParam::NbDev(_) | IndicatorParam::Deviation(_) |
            IndicatorParam::Acceleration(_) | IndicatorParam::Maximum(_) |
            IndicatorParam::Minimum(_) | IndicatorParam::Penetration(_) |
            IndicatorParam::VFactor(_) | IndicatorParam::FastLimit(_) |
            IndicatorParam::SlowLimit(_) | IndicatorParam::Multiplier(_) |
            IndicatorParam::Offset(_) | IndicatorParam::StartValue(_) |
            IndicatorParam::EndValue(_)
        )
    }

    // 获取参数名称
    pub fn param_name(&self) -> &'static str {
        match self {
            IndicatorParam::TimePeriod(_) => "TimePeriod",
            IndicatorParam::FastPeriod(_) => "FastPeriod",
            IndicatorParam::SlowPeriod(_) => "SlowPeriod",
            IndicatorParam::SignalPeriod(_) => "SignalPeriod",
            IndicatorParam::TimePeriod1(_) => "TimePeriod1",
            IndicatorParam::TimePeriod2(_) => "TimePeriod2",
            IndicatorParam::TimePeriod3(_) => "TimePeriod3",
            IndicatorParam::DevUp(_) => "DevUp",
            IndicatorParam::DevDown(_) => "DevDown",
            IndicatorParam::NbDev(_) => "NbDev",
            IndicatorParam::Deviation(_) => "Deviation",
            IndicatorParam::Acceleration(_) => "Acceleration",
            IndicatorParam::Maximum(_) => "Maximum",
            IndicatorParam::Minimum(_) => "Minimum",
            IndicatorParam::MAType(_) => "MAType",
            IndicatorParam::FastMAType(_) => "FastMAType",
            IndicatorParam::SlowMAType(_) => "SlowMAType",
            IndicatorParam::Penetration(_) => "Penetration",
            IndicatorParam::VFactor(_) => "VFactor",
            IndicatorParam::FastLimit(_) => "FastLimit",
            IndicatorParam::SlowLimit(_) => "SlowLimit",
            IndicatorParam::FastKPeriod(_) => "FastKPeriod",
            IndicatorParam::SlowKPeriod(_) => "SlowKPeriod",
            IndicatorParam::SlowKMAType(_) => "SlowKMAType",
            IndicatorParam::SlowDPeriod(_) => "SlowDPeriod",
            IndicatorParam::SlowDMAType(_) => "SlowDMAType",
            IndicatorParam::Multiplier(_) => "Multiplier",
            IndicatorParam::Offset(_) => "Offset",
            IndicatorParam::StartValue(_) => "StartValue",
            IndicatorParam::EndValue(_) => "EndValue",
            IndicatorParam::PriceSource(_) => "PriceSource",
        }
    }
}

/// 指标输入数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorInput {
    Single(Vec<f64>),                                           // inReal
    Dual(Vec<f64>, Vec<f64>),                                  // inReal0, inReal1
    OHLC(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>),             // Open, High, Low, Close
    OHLCV(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>),  // + Volume
    HLC(Vec<f64>, Vec<f64>, Vec<f64>),                        // High, Low, Close
    HL(Vec<f64>, Vec<f64>),                                   // High, Low
}

/// 指标输出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorOutput {
    Single(Vec<f64>),           // 单一输出，如SMA
    Dual(Vec<f64>, Vec<f64>),   // 双输出，如AROON
    Triple(Vec<f64>, Vec<f64>, Vec<f64>), // 三重输出，如MACD, BBands
    Quad(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), // 四重输出，如STOCH
}

/// 输出格式类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    Single, // 单一输出，如SMA
    Dual, // 双输出，如AROON
    Triple, // 三输出，如MACD
    Quad, // 四输出，如BBands
}

/// 输入数据类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InputType {
    Single,     // 单一数据输入
    Dual,       // 双数据输入
    OHLC,       // OHLC数据输入
    OHLCV,      // OHLCV数据输入
    HLC,        // HLC数据输入
    HL,         // HL数据输入
}

/// 指标分组
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// 参数元数据类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ParamType {
    Integer,
    Float,
    MAType,
}

/// 参数值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParamValue {
    Integer(i32),
    Float(f64),
    PriceSource(PriceSource),
    MAType(MAType),
}

/// 参数元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamMeta {
    pub name: String,
    pub param_type: ParamType,
    pub default_value: Option<ParamValue>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub description: String,
}

/// 指标元数据（不包含函数指针，用于序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorMetaData {
    pub name: String,
    pub group: IndicatorGroup,
    pub input_type: InputType,
    pub output_format: OutputFormat,
    pub description: String,
    pub params: Vec<ParamMeta>,
}

/// 指标信息（用于查询和展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorInfo {
    pub name: String,
    pub description: String,
    pub group: IndicatorGroup,
    pub input_type: InputType,
    pub output_format: OutputFormat,
    pub params: Vec<ParamMeta>,
}

// 为IndicatorOutput添加便利方法
impl IndicatorOutput {
    pub fn as_single(&self) -> Result<&Vec<f64>, String> {
        match self {
            IndicatorOutput::Single(data) => Ok(data),
            _ => Err("Expected single output".to_string()),
        }
    }

    pub fn as_dual(&self) -> Result<(&Vec<f64>, &Vec<f64>), String> {
        match self {
            IndicatorOutput::Dual(data1, data2) => Ok((data1, data2)),
            _ => Err("Expected dual output".to_string()),
        }
    }

    pub fn as_triple(&self) -> Result<(&Vec<f64>, &Vec<f64>, &Vec<f64>), String> {
        match self {
            IndicatorOutput::Triple(data1, data2, data3) => Ok((data1, data2, data3)),
            _ => Err("Expected triple output".to_string()),
        }
    }

    pub fn as_quad(&self) -> Result<(&Vec<f64>, &Vec<f64>, &Vec<f64>, &Vec<f64>), String> {
        match self {
            IndicatorOutput::Quad(data1, data2, data3, data4) => Ok((data1, data2, data3, data4)),
            _ => Err("Expected quad output".to_string()),
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
