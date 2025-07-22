
// pub mod sma; // 简单移动平均线
// pub mod bbands; // 布林带
// pub mod macd; // 指数平滑异同移动平均线
// pub mod rsi; // 相对强弱指数
pub mod utils; // 指标配置解析工具
// pub mod talib_types; // TA-Lib 类型定义
// pub mod registry; // 指标注册表
pub mod indicator_macros;
pub mod indicator;


use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::any::Any;
use std::collections::HashMap;
use crate::cache::{CacheItem, CacheValue};
use crate::indicator::indicator::*;
use deepsize::DeepSizeOf;
use strum::{EnumString, Display};
use crate::{impl_indicator, impl_indicator_config};

// 价格来源
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display)]
pub enum PriceSource {
    #[strum(serialize = "close")]
    Close,
    #[strum(serialize = "open")]
    Open,
    #[strum(serialize = "high")]
    High,
    #[strum(serialize = "low")]
    Low,
}

#[derive(Debug, Clone, Hash, Eq,PartialEq, Serialize, Deserialize, EnumString, Display)]
pub enum MAType {
    #[strum(serialize = "sma")]
    SMA, // 简单移动平均线
    #[strum(serialize = "ema")]
    EMA, // Exponential Moving Average (EMA) 指数移动平均线
    #[strum(serialize = "wma")]
    WMA, // Weighted Moving Average (WMA) 加权移动平均线
    #[strum(serialize = "dema")]
    DEMA, // Double Exponential Moving Average (DEMA) 双指数移动平均线
    #[strum(serialize = "tema")]
    TEMA, // Triple Exponential Moving Average (TEMA) 三指数移动平均线
    #[strum(serialize = "trima")]
    TRIMA, // Triangular Moving Average (TRIMA) 三角形移动平均线
    #[strum(serialize = "kama")]
    KAMA, // Kaufman Adaptive Moving Average (KAMA) 卡夫曼自适应移动平均线
    #[strum(serialize = "mama")]
    MAMA, // MESA Adaptive Moving Average (MAMA) 梅萨自适应移动平均线
    #[strum(serialize = "t3")]
    T3, // Triple Exponential Moving Average (T3) 三重指数移动平均线
}


pub trait IndicatorConfigTrait {
    fn new(config: &Value) -> Result<Self, String> where Self: Sized; // 创建指标配置,有可能失败
}

pub trait IndicatorTrait {
    fn to_json(&self) -> serde_json::Value;
    fn to_list(&self) -> Vec<f64>;
    fn to_json_with_time(&self) -> serde_json::Value;
}

// 2. 为枚举使用enum_dispatch
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "indicator_type", content = "indicator_config")]
pub enum IndicatorConfig {

    #[serde(rename = "accbands")]
    ACCBANDS(ACCBANDSConfig),

    #[serde(rename = "ad")]
    AD(ADConfig),

    #[serde(rename = "adx")]
    ADX(ADXConfig),

    #[serde(rename = "adxr")]
    ADXR(ADXRConfig),

    #[serde(rename = "apo")]
    APO(APOConfig),

    #[serde(rename = "aroon")]
    AROON(AROONConfig),

    #[serde(rename = "aroonosc")]
    AROONOSC(AROONOSCConfig),

    #[serde(rename = "atr")]
    ATR(ATRConfig),

    #[serde(rename = "bbands")]
    BBands(BBandsConfig),

    #[serde(rename = "adosc")]
    ADOSC(ADOSCConfig),

    #[serde(rename = "bop")]
    BOP(BOPConfig),

    #[serde(rename = "cci")]
    CCI(CCIConfig),

    #[serde(rename = "rsi")]
    RSI(RSIConfig),

    
    
    
    


    // 简单移动平均线
    #[serde(rename = "sma")]
    MA(MAConfig),

    #[serde(rename = "macd")]
    MACD(MACDConfig),

    #[serde(rename = "cdl2crows")]
    CDL2CROWS(CDL2CROWSConfig),

    #[serde(rename = "cdl3blackcrows")]
    CDL3BLACKCROWS(CDL3BLACKCROWSConfig),

    #[serde(rename = "cdl3inside")]
    CDL3INSIDE(CDL3INSIDEConfig),

    #[serde(rename = "cdl3linestrike")]
    CDL3LINESTRIKE(CDL3LINESTRIKEConfig),

    #[serde(rename = "cdl3outside")]
    CDL3OUTSIDE(CDL3OUTSIDEConfig),

    #[serde(rename = "cdl3starsinsouth")]
    CDL3STARSINSOUTH(CDL3STARSINSOUTHConfig),

    #[serde(rename = "cdladvanceblock")]
    CDLADVANCEBLOCK(CDLADVANCEBLOCKConfig),

    #[serde(rename = "cdlbellwether")]
    CDLBELTHOLD(CDLBELTHOLDConfig),

    #[serde(rename = "cdlbreakaway")]
    CDLBREAKAWAY(CDLBREAKAWAYConfig),

    #[serde(rename = "cdlclosingmarubozu")]
    CDLCLOSINGMARUBOZU(CDLCLOSINGMARUBOZUConfig),

    #[serde(rename = "cdlconcealbabyswall")]
    CDLCONCEALBABYSWALL(CDLCONCEALBABYSWALLConfig),

    #[serde(rename = "cdlcounterattack")]
    CDLCOUNTERATTACK(CDLCOUNTERATTACKConfig),

    #[serde(rename = "cdldarkcloudcover")]
    CDLDARKCLOUDCOVER(CDLDARKCLOUDCOVERConfig),

    #[serde(rename = "cdldoji")]
    CDLDOJI(CDLDOJIConfig),

    #[serde(rename = "cdl3whitesoldiers")]
    CDL3WHITESOLDIERS(CDL3WHITESOLDIERSConfig),

    #[serde(rename = "cdlabandonedbaby")]
    CDLABANDONEDBABY(CDLABANDONEDBABYConfig),

    #[serde(rename = "cmo")]
    CMO(CMOConfig),

    #[serde(rename = "dx")]
    DX(DXConfig),   

    #[serde(rename = "macdext")]
    MACDEXT(MACDEXTConfig),

    #[serde(rename = "macdfix")]
    MACDFIX(MACDFIXConfig),

    #[serde(rename = "mfi")]
    MFI(MFIConfig),

    #[serde(rename = "minus_di")]
    MinusDi(MinusDiConfig),

    #[serde(rename = "minus_dm")]
    MinusDm(MinusDmConfig),

    #[serde(rename = "mom")]
    MOM(MOMConfig),

    #[serde(rename = "plus_di")]
    PlusDi(PlusDiConfig),

    #[serde(rename = "plus_dm")]
    PlusDm(PlusDmConfig),

    #[serde(rename = "ppo")]
    PPO(PPOConfig),

    #[serde(rename = "roc")]
    ROC(ROCConfig),

    #[serde(rename = "rocp")]
    ROCP(ROCPConfig),

    #[serde(rename = "rocr")]
    ROCR(ROCRConfig),

    #[serde(rename = "rocr100")]
    ROCR100(ROCR100Config),

    #[serde(rename = "stoch")]
    STOCH(STOCHConfig),

    #[serde(rename = "stochf")]
    STOCHF(STOCHFConfig),

    #[serde(rename = "stochrsi")]
    STOCHRSI(STOCHRSIConfig),

    #[serde(rename = "ultosc")]
    ULTOSC(ULTOSCConfig),



}

impl_indicator_config!(IndicatorConfig,
    (
        ACCBANDS,
        AD,
        ADOSC,
        ADX,
        ADXR,
        APO,
        AROON,
        AROONOSC,
        ATR,
        BBands,
        BOP,
        CCI,
        CDL2CROWS,
        CDL3BLACKCROWS,
        CDL3INSIDE,
        CDL3LINESTRIKE,
        CDL3OUTSIDE,
        CDL3STARSINSOUTH,
        CDLADVANCEBLOCK,
        CDLBELTHOLD,
        CDLBREAKAWAY,
        CDLCLOSINGMARUBOZU,
        CDLCONCEALBABYSWALL,
        CDLCOUNTERATTACK,
        CDLDARKCLOUDCOVER,
        CDLDOJI,
        CDL3WHITESOLDIERS,
        CDLABANDONEDBABY,
        CMO,
        DX,
        MACDEXT,
        MACDFIX,
        MFI,
        MinusDi,
        MinusDm,
        MOM,
        PlusDi,
        PlusDm,
        PPO,
        ROC,
        ROCP,
        ROCR,
        ROCR100,
        STOCH,
        RSI,
        MA,
        STOCHRSI,
        MACD,
        STOCHF,
        ULTOSC
    )
);







#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, DeepSizeOf)]
pub enum Indicator {
    ACCBANDS(ACCBANDS), // 加速布林带
    AD(AD), // 蔡金累积/派发线
    ADOSC(ADOSC), // 蔡金振荡器
    ADX(ADX), // 平均方向性指数
    ADXR(ADXR), // 平均方向性指数评级
    APO(APO), // 绝对价格振荡器
    AROON(AROON), // 阿隆指标
    AROONOSC(AROONOSC), // 阿隆振荡器
    ATR(ATR), // 平均真实波幅
    BBands(BBands), // 布林带
    BOP(BOP), // Balance of power
    CCI(CCI), // 商品通道指数

    CDL2CROWS(CDL2CROWS), // 两乌鸦

    MA(MA),
    
    MACD(MACD),
    RSI(RSI),
    CMO(CMO),
    DX(DX),
    MACDEXT(MACDEXT),
    MACDFIX(MACDFIX),
    MFI(MFI),
    MinusDi(MinusDi),
    MinusDm(MinusDm),
    MOM(MOM),
    PlusDi(PlusDi),
    PlusDm(PlusDm),    
    PPO(PPO),
    ROC(ROC),
    ROCP(ROCP),
    ROCR(ROCR),
    ROCR100(ROCR100),
    STOCH(STOCH),
    STOCHF(STOCHF),
    STOCHRSI(STOCHRSI),
    ULTOSC(ULTOSC),
    CDL3BLACKCROWS(CDL3BLACKCROWS),
    CDL3INSIDE(CDL3INSIDE),
    CDL3LINESTRIKE(CDL3LINESTRIKE),
    CDL3OUTSIDE(CDL3OUTSIDE),
    CDL3STARSINSOUTH(CDL3STARSINSOUTH),
    CDLADVANCEBLOCK(CDLADVANCEBLOCK),
    CDLBELTHOLD(CDLBELTHOLD),
    CDLBREAKAWAY(CDLBREAKAWAY),
    CDLCLOSINGMARUBOZU(CDLCLOSINGMARUBOZU),
    CDLCONCEALBABYSWALL(CDLCONCEALBABYSWALL),
    CDLCOUNTERATTACK(CDLCOUNTERATTACK),
    CDLDARKCLOUDCOVER(CDLDARKCLOUDCOVER),
    CDLDOJI(CDLDOJI),
    CDL3WHITESOLDIERS(CDL3WHITESOLDIERS),
    CDLABANDONEDBABY(CDLABANDONEDBABY),
}

// 使用宏自动生成所有重复的match实现
impl_indicator!(
    Indicator, 
    ACCBANDS,
    AD,
    ADOSC,
    ADX,
    ADXR,
    APO,
    AROON,
    AROONOSC,
    ATR,
    BBands,
    BOP,
    CCI,
    CDL2CROWS,
    MA,
    MACD, 
    RSI, 
    CMO, 
    DX, 
    MACDEXT, 
    MACDFIX, 
    MFI, 
    MinusDi, 
    MinusDm, 
    MOM, 
    PlusDi, 
    PlusDm, 
    PPO, 
    ROC, 
    ROCP, 
    ROCR, 
    ROCR100, 
    STOCH, 
    STOCHF, 
    STOCHRSI, 
    ULTOSC,
    CDL3BLACKCROWS,
    CDL3INSIDE,
    CDL3LINESTRIKE,
    CDL3OUTSIDE,
    CDL3STARSINSOUTH,
    CDLADVANCEBLOCK,
    CDLBELTHOLD,
    CDLBREAKAWAY,
    CDLCLOSINGMARUBOZU,
    CDLCONCEALBABYSWALL,
    CDLCOUNTERATTACK,
    CDLDARKCLOUDCOVER,
    CDLDOJI,
    CDL3WHITESOLDIERS,
    CDLABANDONEDBABY
);


impl From<Indicator> for CacheValue {   
    fn from(indicator: Indicator) -> Self {
        CacheValue::Indicator(indicator)
    }
}

#[typetag::serde(tag = "type")]
pub trait IndicatorData: Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn IndicatorData>;
    fn get_indicator_value(&self) -> HashMap<String, Vec<IndicatorValue>>;
    fn get_latest_indicator_value(&self) -> HashMap<String, IndicatorValue>; // 获取最新指标值
}

impl Clone for Box<dyn IndicatorData> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndicatorValue {
    pub timestamp: i64,
    pub value: f64,
}




#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SMAIndicator {
    pub exchange: Exchange,
    pub symbol: String,
    pub kline_interval: KlineInterval,
    pub indicator_config: MAConfig,
    pub indicator_value: HashMap<String, Vec<IndicatorValue>>,
}

#[typetag::serde]
impl IndicatorData for SMAIndicator {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn IndicatorData> {
        Box::new(self.clone())
    }
    fn get_indicator_value(&self) -> HashMap<String, Vec<IndicatorValue>> {
        self.indicator_value.clone()
    }
    fn get_latest_indicator_value(&self) -> HashMap<String, IndicatorValue> {
        self.indicator_value.iter().map(|(key, value)| {
            let latest_value = value.last().unwrap();
            (key.clone(), latest_value.clone())
        }).collect()
    }
}