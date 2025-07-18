pub mod rsi;
pub mod ma;
pub mod macd;
pub mod bbands;
pub mod indicator_meta;
pub mod indicator_macros;
pub mod indicator_definitions;
pub mod indicators;
pub mod examples;
pub mod expand_test;

use crate::indicator_engine::talib_bindings::*;
use crate::indicator_engine::talib_error::TalibError;
use types::indicator::IndicatorConfig;
use indicator_meta::*;
use indicator_definitions::*;
use indicators::*;

#[derive(Clone)]
pub struct TALib;


impl TALib {

    pub fn init() -> Result<Self, String> {
        unsafe {
            let ret = TA_Initialize();
            if ret != TA_RetCode_TA_SUCCESS {
                return Err(format!("TA-Lib 初始化失败: {:?}", ret));
            }
        }

        // 初始化指标注册表
        indicators::init_indicators();

        Ok(Self)
    }

    pub fn lookback(indicator_config: &IndicatorConfig) -> u32 {
        match indicator_config {
            IndicatorConfig::SMA(sma_config) => {
                let params = vec![IndicatorParam::Period(sma_config.period)];
                indicators::get_indicator_lookback("SMA", &params).unwrap_or(0) as u32
            },
            IndicatorConfig::MACD(macd_config) => {
                let params = vec![
                    IndicatorParam::FastPeriod(macd_config.fast_period),
                    IndicatorParam::SlowPeriod(macd_config.slow_period),
                    IndicatorParam::SignalPeriod(macd_config.signal_period),
                ];
                indicators::get_indicator_lookback("MACD", &params).unwrap_or(0) as u32
            },
            IndicatorConfig::BBands(bbands_config) => {
                let params = vec![
                    IndicatorParam::Period(bbands_config.period),
                    IndicatorParam::DevUp(bbands_config.dev_up.into_inner()),
                    IndicatorParam::DevDown(bbands_config.dev_down.into_inner()),
                    IndicatorParam::MAType(bbands_config.ma_type.clone() as i32),
                ];
                indicators::get_indicator_lookback("BBANDS", &params).unwrap_or(0) as u32
            },
        }
    }



    


    

    /// 通用的指标计算接口
    pub fn calculate_indicator(name: &str, input: &IndicatorInput, params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
        get_indicator_registry().calculate(name, input, params)
    }

    /// 单一数据输入的便利方法
    pub fn calculate_indicator_single(name: &str, data: &[f64], params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
        get_indicator_registry().calculate_single(name, data, params)
    }

    /// 获取指标的lookback期
    pub fn get_indicator_lookback(name: &str, params: &[IndicatorParam]) -> Result<i32, TalibError> {
        get_indicator_registry().lookback(name, params)
    }

    /// 列出所有可用的指标
    pub fn list_indicators() -> Vec<&'static str> {
        get_indicator_registry().list_indicators()
    }

    /// 按分组列出指标
    pub fn get_indicators_by_group(group: IndicatorGroup) -> Vec<&'static str> {
        get_indicator_registry().get_indicators_by_group(group)
    }

    /// 搜索指标
    pub fn search_indicators(keyword: &str) -> Vec<&'static str> {
        get_indicator_registry().search_indicators(keyword)
    }

    /// 获取指标信息
    pub fn get_indicator_info(name: &str) -> Option<(&IndicatorGroup, &InputType, &OutputFormat, &str)> {
        get_indicator_registry().get_indicator_info(name)
    }

    /// 获取指标目录
    pub fn get_catalog() -> &'static IndicatorCatalog {
        get_indicator_catalog()
    }

    pub fn shutdown() {
        unsafe {
            TA_Shutdown();
        }
    }
}
