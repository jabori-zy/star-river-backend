pub mod indicator_macros;
pub mod utils;
pub mod indicator_define;

use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use crate::error::star_river_error::*;
use crate::indicator::indicator_define::cycle::*;
use crate::indicator::indicator_define::momentum::*;
use crate::indicator::indicator_define::overlap::*;
use crate::indicator::indicator_define::pattern_recognition::*;
use crate::indicator::indicator_define::price_transform::*;
use crate::indicator::indicator_define::volatility::*;
use crate::indicator::indicator_define::volume::*;
use crate::{impl_indicator, impl_indicator_config};
use deepsize::DeepSizeOf;
use strum::{Display, EnumString};
use chrono::{DateTime, Utc};

use crate::error::indicator_error::*;
use snafu::ResultExt;

// 价格来源
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display)]
pub enum PriceSource {
    #[strum(serialize = "CLOSE")]
    #[serde(rename = "CLOSE")]
    Close,
    #[strum(serialize = "OPEN")]
    #[serde(rename = "OPEN")]
    Open,
    #[strum(serialize = "HIGH")]
    #[serde(rename = "HIGH")]
    High,
    #[strum(serialize = "LOW")]
    #[serde(rename = "LOW")]
    Low,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, EnumString, Display)]
pub enum MAType {
    #[strum(serialize = "SMA")]
    #[serde(rename = "SMA")]
    SMA, // 简单移动平均线
    #[strum(serialize = "EMA")]
    #[serde(rename = "EMA")]
    EMA, // Exponential Moving Average (EMA) 指数移动平均线
    #[strum(serialize = "WMA")]
    #[serde(rename = "WMA")]
    WMA, // Weighted Moving Average (WMA) 加权移动平均线
    #[strum(serialize = "DEMA")]
    #[serde(rename = "DEMA")]
    DEMA, // Double Exponential Moving Average (DEMA) 双指数移动平均线
    #[strum(serialize = "TEMA")]
    #[serde(rename = "TEMA")]
    TEMA, // Triple Exponential Moving Average (TEMA) 三指数移动平均线
    #[strum(serialize = "TRIMA")]
    #[serde(rename = "TRIMA")]
    TRIMA, // Triangular Moving Average (TRIMA) 三角形移动平均线
    #[strum(serialize = "KAMA")]
    #[serde(rename = "KAMA")]
    KAMA, // Kaufman Adaptive Moving Average (KAMA) 卡夫曼自适应移动平均线
    #[strum(serialize = "MAMA")]
    #[serde(rename = "MAMA")]
    MAMA, // MESA Adaptive Moving Average (MAMA) 梅萨自适应移动平均线
    #[strum(serialize = "T3")]
    #[serde(rename = "T3")]
    T3, // Triple Exponential Moving Average (T3) 三重指数移动平均线
}

pub trait IndicatorConfigTrait {
    fn new(config: &Value) -> Result<Self, serde_json::Error>
    where
        Self: Sized; // 创建指标配置,有可能失败
}

// pub trait IndicatorTrait {
//     fn to_json(&self) -> serde_json::Value;
//     fn to_list(&self) -> Vec<f64>;
//     fn to_json_with_time(&self) -> serde_json::Value;
// }

// 2. 为枚举使用enum_dispatch
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "indicator_type", content = "indicator_config")]
pub enum IndicatorConfig {
    // Overlap
    #[serde(rename = "bbands")]
    BBANDS(BBANDSConfig),

    #[serde(rename = "dema")]
    DEMA(DEMAConfig),

    #[serde(rename = "ema")]
    EMA(EMAConfig),

    #[serde(rename = "ht_trendline")]
    HtTrendline(HtTrendlineConfig),

    #[serde(rename = "kama")]
    KAMA(KAMAConfig),

    #[serde(rename = "ma")]
    MA(MAConfig),

    #[serde(rename = "mama")]
    MAMA(MAMAConfig),

    // #[serde(rename = "mavp")]
    // MAVP(MAVPConfig),
    #[serde(rename = "midpoint")]
    MIDPOINT(MIDPOINTConfig),

    #[serde(rename = "midprice")]
    MIDPRICE(MIDPRICEConfig),

    #[serde(rename = "sar")]
    SAR(SARConfig),

    #[serde(rename = "sarext")]
    SAREXT(SAREXTConfig),

    #[serde(rename = "sma")]
    SMA(SMAConfig),

    #[serde(rename = "t3")]
    T3(T3Config),

    #[serde(rename = "tema")]
    TEMA(TEMAConfig),

    #[serde(rename = "trima")]
    TRIMA(TRIMAConfig),

    #[serde(rename = "wma")]
    WMA(WMAConfig),

    // Momentum
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
    #[serde(rename = "bop")]
    BOP(BOPConfig),
    #[serde(rename = "cci")]
    CCI(CCIConfig),
    #[serde(rename = "cmo")]
    CMO(CMOConfig),
    #[serde(rename = "dx")]
    DX(DXConfig),
    #[serde(rename = "macd")]
    MACD(MACDConfig),
    #[serde(rename = "macdxt")]
    MACDEXT(MACDEXTConfig),
    #[serde(rename = "macdfix")]
    MACDFIX(MACDFIXConfig),
    #[serde(rename = "mfi")]
    MFI(MFIConfig),
    #[serde(rename = "minusdi")]
    MinusDi(MinusDiConfig),
    #[serde(rename = "minusdm")]
    MinusDm(MinusDmConfig),
    #[serde(rename = "mom")]
    MOM(MOMConfig),
    #[serde(rename = "plusdi")]
    PlusDi(PlusDiConfig),
    #[serde(rename = "plusdm")]
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
    #[serde(rename = "rsi")]
    RSI(RSIConfig),
    #[serde(rename = "stoch")]
    STOCH(STOCHConfig),
    #[serde(rename = "stochf")]
    STOCHF(STOCHFConfig),
    #[serde(rename = "stochrsi")]
    STOCHRSI(STOCHRSIConfig),
    #[serde(rename = "trix")]
    TRIX(TRIXConfig),
    #[serde(rename = "ultosc")]
    ULTOSC(ULTOSCConfig),
    #[serde(rename = "willr")]
    WILLR(WILLRConfig),

    // Volume
    #[serde(rename = "ad")]
    AD(ADConfig),
    #[serde(rename = "adosc")]
    ADOSC(ADOSCConfig),
    #[serde(rename = "obv")]
    OBV(OBVConfig),

    // Cycle
    #[serde(rename = "ht_dcperiod")]
    HtDcperiod(HtDcperiodConfig),
    #[serde(rename = "ht_dcphase")]
    HtDcphase(HtDcphaseConfig),
    #[serde(rename = "ht_phasor")]
    HtPhasor(HtPhasorConfig),
    #[serde(rename = "ht_sine")]
    HtSine(HtSineConfig),
    #[serde(rename = "ht_trendmode")]
    HtTrendmode(HtTrendmodeConfig),

    // Price Transform
    #[serde(rename = "avg_price")]
    AVGPRICE(AVGPRICEConfig),
    #[serde(rename = "med_price")]
    MEDPRICE(MEDPRICEConfig),
    #[serde(rename = "typ_price")]
    TYPPRICE(TYPPRICEConfig),
    #[serde(rename = "wcl_price")]
    WCLPRICE(WCLPRICEConfig),

    // Volatility
    #[serde(rename = "atr")]
    ATR(ATRConfig),
    #[serde(rename = "natr")]
    NATR(NATRConfig),
    #[serde(rename = "trange")]
    TRANGE(TRANGEConfig),

    // Pattern Recognition
    #[serde(rename = "2_crows")]
    CDL2CROWS(CDL2CROWSConfig),
    #[serde(rename = "3_black_crows")]
    CDL3BLACKCROWS(CDL3BLACKCROWSConfig),
    #[serde(rename = "3_inside")]
    CDL3INSIDE(CDL3INSIDEConfig),
    #[serde(rename = "3_line_strike")]
    CDL3LINESTRIKE(CDL3LINESTRIKEConfig),
    #[serde(rename = "3_outside")]
    CDL3OUTSIDE(CDL3OUTSIDEConfig),
    #[serde(rename = "3_stars_in_south")]
    CDL3STARSINSOUTH(CDL3STARSINSOUTHConfig),
    #[serde(rename = "3_white_soldiers")]
    CDL3WHITESOLDIERS(CDL3WHITESOLDIERSConfig),
    #[serde(rename = "abandoned_baby")]
    CDLABANDONEDBABY(CDLABANDONEDBABYConfig),
    #[serde(rename = "advance_block")]
    CDLADVANCEBLOCK(CDLADVANCEBLOCKConfig),
    #[serde(rename = "belthold")]
    CDLBELTHOLD(CDLBELTHOLDConfig),
    #[serde(rename = "breakaway")]
    CDLBREAKAWAY(CDLBREAKAWAYConfig),
    #[serde(rename = "closing_marubozu")]
    CDLCLOSINGMARUBOZU(CDLCLOSINGMARUBOZUConfig),
    #[serde(rename = "conceal_baby_wall")]
    CDLCONCEALBABYSWALL(CDLCONCEALBABYSWALLConfig),
    #[serde(rename = "counter_attack")]
    CDLCOUNTERATTACK(CDLCOUNTERATTACKConfig),
    #[serde(rename = "dark_cloud_cover")]
    CDLDARKCLOUDCOVER(CDLDARKCLOUDCOVERConfig),
    #[serde(rename = "doji")]
    CDLDOJI(CDLDOJIConfig),
    #[serde(rename = "doji_star")]
    CDLDOJISTAR(CDLDOJISTARConfig),
    #[serde(rename = "dragonfly_doji")]
    CDLDRAGONFLYDOJI(CDLDRAGONFLYDOJIConfig),
    #[serde(rename = "engulfing")]
    CDLENGULFING(CDLENGULFINGConfig),
    #[serde(rename = "evening_doji_star")]
    CDLEVENINGDOJISTAR(CDLEVENINGDOJISTARConfig),
    #[serde(rename = "evening_star")]
    CDLEVENINGSTAR(CDLEVENINGSTARConfig),
    #[serde(rename = "gap_side_side_white")]
    CDLGAPSIDESIDEWHITE(CDLGAPSIDESIDEWHITEConfig),
    #[serde(rename = "gravestone_doji")]
    CDLGRAVESTONEDOJI(CDLGRAVESTONEDOJIConfig),
    #[serde(rename = "hammer")]
    CDLHAMMER(CDLHAMMERConfig),
    #[serde(rename = "hanging_man")]
    CDLHANGINGMAN(CDLHANGINGMANConfig),
    #[serde(rename = "harami")]
    CDLHARAMI(CDLHARAMIConfig),
    #[serde(rename = "harami_cross")]
    CDLHARAMICROSS(CDLHARAMICROSSConfig),
    #[serde(rename = "high_wave")]
    CDLHIGHWAVE(CDLHIGHWAVEConfig),
    #[serde(rename = "hikkake")]
    CDLHIKKAKE(CDLHIKKAKEConfig),
    #[serde(rename = "hikkake_mod")]
    CDLHIKKAKEMOD(CDLHIKKAKEMODConfig),
    #[serde(rename = "homing_pigeon")]
    CDLHOMINGPIGEON(CDLHOMINGPIGEONConfig),
    #[serde(rename = "identical_3_crows")]
    CDLIDENTICAL3CROWS(CDLIDENTICAL3CROWSConfig),
    #[serde(rename = "in_neck")]
    CDLINNECK(CDLINNECKConfig),
    #[serde(rename = "inverted_hammer")]
    CDLINVERTEDHAMMER(CDLINVERTEDHAMMERConfig),
    #[serde(rename = "kicking")]
    CDLKICKING(CDLKICKINGConfig),
    #[serde(rename = "kicking_by_length")]
    CDLKICKINGBYLENGTH(CDLKICKINGBYLENGTHConfig),
    #[serde(rename = "ladder_bottom")]
    CDLLADDERBOTTOM(CDLLADDERBOTTOMConfig),
    #[serde(rename = "long_legged_doji")]
    CDLLONGLEGGEDDOJI(CDLLONGLEGGEDDOJIConfig),
    #[serde(rename = "long_line")]
    CDLLONGLINE(CDLLONGLINEConfig),
    #[serde(rename = "marubozu")]
    CDLMARUBOZU(CDLMARUBOZUConfig),
    #[serde(rename = "matching_low")]
    CDLMATCHINGLOW(CDLMATCHINGLOWConfig),
    #[serde(rename = "mat_hold")]
    CDLMATHOLD(CDLMATHOLDConfig),
    #[serde(rename = "morning_doji_star")]
    CDLMORNINGDOJISTAR(CDLMORNINGDOJISTARConfig),
    #[serde(rename = "morning_star")]
    CDLMORNINGSTAR(CDLMORNINGSTARConfig),
    #[serde(rename = "on_neck")]
    CDLONNECK(CDLONNECKConfig),
    #[serde(rename = "piercing")]
    CDLPIERCING(CDLPIERCINGConfig),
    #[serde(rename = "rickshaw_man")]
    CDLRICKSHAWMAN(CDLRICKSHAWMANConfig),
    #[serde(rename = "rise_fall_3_methods")]
    CDLRISEFALL3METHODS(CDLRISEFALL3METHODSConfig),
    #[serde(rename = "separating_lines")]
    CDLSEPARATINGLINES(CDLSEPARATINGLINESConfig),
    #[serde(rename = "shooting_star")]
    CDLSHOOTINGSTAR(CDLSHOOTINGSTARConfig),
    #[serde(rename = "short_line")]
    CDLSHORTLINE(CDLSHORTLINEConfig),
    #[serde(rename = "spinning_top")]
    CDLSPINNINGTOP(CDLSPINNINGTOPConfig),
    #[serde(rename = "stalled_pattern")]
    CDLSTALLEDPATTERN(CDLSTALLEDPATTERNConfig),
    #[serde(rename = "stick_sandwich")]
    CDLSTICKSANDWICH(CDLSTICKSANDWICHConfig),
    #[serde(rename = "takuri")]
    CDLTAKURI(CDLTAKURIConfig),
    #[serde(rename = "tasuki_gap")]
    CDLTASUKIGAP(CDLTASUKIGAPConfig),
    #[serde(rename = "thrusting")]
    CDLTHRUSTING(CDLTHRUSTINGConfig),
    #[serde(rename = "tristar")]
    CDLTRISTAR(CDLTRISTARConfig),
    #[serde(rename = "unique_3_river")]
    CDLUNIQUE3RIVER(CDLUNIQUE3RIVERConfig),
    #[serde(rename = "upside_gap_2_crows")]
    CDLUPSIDEGAP2CROWS(CDLUPSIDEGAP2CROWSConfig),
    #[serde(rename = "xside_gap_3_methods")]
    CDLXSIDEGAP3METHODS(CDLXSIDEGAP3METHODSConfig),
}

impl_indicator_config!(
    IndicatorConfig,
    (
        // Overlap
        BBANDS,
        DEMA,
        EMA,
        HtTrendline,
        KAMA,
        MA,
        MAMA,
        // MAVP,
        MIDPOINT,
        MIDPRICE,
        SAR,
        SAREXT,
        SMA,
        T3,
        TEMA,
        TRIMA,
        WMA,
        // Momentum
        ADX,
        ADXR,
        APO,
        AROON,
        AROONOSC,
        BOP,
        CCI,
        CMO,
        DX,
        MACD,
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
        RSI,
        STOCH,
        STOCHF,
        STOCHRSI,
        TRIX,
        ULTOSC,
        WILLR,
        // Volume
        AD,
        ADOSC,
        OBV,
        // Cycle
        HtDcperiod,
        HtDcphase,
        HtPhasor,
        HtSine,
        HtTrendmode,
        // Price Transform
        AVGPRICE,
        MEDPRICE,
        TYPPRICE,
        WCLPRICE,
        // Volatility
        ATR,
        NATR,
        TRANGE,
        // Pattern Recognition
        CDL2CROWS,
        CDL3BLACKCROWS,
        CDL3INSIDE,
        CDL3LINESTRIKE,
        CDL3OUTSIDE,
        CDL3STARSINSOUTH,
        CDL3WHITESOLDIERS,
        CDLABANDONEDBABY,
        CDLADVANCEBLOCK,
        CDLBELTHOLD,
        CDLBREAKAWAY,
        CDLCLOSINGMARUBOZU,
        CDLCONCEALBABYSWALL,
        CDLCOUNTERATTACK,
        CDLDARKCLOUDCOVER,
        CDLDOJI,
        CDLDOJISTAR,
        CDLDRAGONFLYDOJI,
        CDLENGULFING,
        CDLEVENINGDOJISTAR,
        CDLEVENINGSTAR,
        CDLGAPSIDESIDEWHITE,
        CDLGRAVESTONEDOJI,
        CDLHAMMER,
        CDLHANGINGMAN,
        CDLHARAMI,
        CDLHARAMICROSS,
        CDLHIGHWAVE,
        CDLHIKKAKE,
        CDLHIKKAKEMOD,
        CDLHOMINGPIGEON,
        CDLIDENTICAL3CROWS,
        CDLINNECK,
        CDLINVERTEDHAMMER,
        CDLKICKING,
        CDLKICKINGBYLENGTH,
        CDLLADDERBOTTOM,
        CDLLONGLEGGEDDOJI,
        CDLLONGLINE,
        CDLMARUBOZU,
        CDLMATCHINGLOW,
        CDLMATHOLD,
        CDLMORNINGDOJISTAR,
        CDLMORNINGSTAR,
        CDLONNECK,
        CDLPIERCING,
        CDLRICKSHAWMAN,
        CDLRISEFALL3METHODS,
        CDLSEPARATINGLINES,
        CDLSHOOTINGSTAR,
        CDLSHORTLINE,
        CDLSPINNINGTOP,
        CDLSTALLEDPATTERN,
        CDLSTICKSANDWICH,
        CDLTAKURI,
        CDLTASUKIGAP,
        CDLTHRUSTING,
        CDLTRISTAR,
        CDLUNIQUE3RIVER,
        CDLUPSIDEGAP2CROWS,
        CDLXSIDEGAP3METHODS
    )
);

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, DeepSizeOf)]
pub enum Indicator {
    // Overlap
    BBANDS(BBANDS),
    DEMA(DEMA),
    EMA(EMA),
    HtTrendline(HtTrendline),
    KAMA(KAMA),
    MA(MA),
    MAMA(MAMA),
    // MAVP(MAVP),
    MIDPOINT(MIDPOINT),
    MIDPRICE(MIDPRICE),
    SAR(SAR),
    SAREXT(SAREXT),
    SMA(SMA),
    T3(T3),
    TEMA(TEMA),
    TRIMA(TRIMA),
    WMA(WMA),

    // Momentum
    ADX(ADX),
    ADXR(ADXR),
    APO(APO),
    AROON(AROON),
    AROONOSC(AROONOSC),
    BOP(BOP),
    CCI(CCI),
    CMO(CMO),
    DX(DX),
    MACD(MACD),
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
    RSI(RSI),
    STOCH(STOCH),
    STOCHF(STOCHF),
    STOCHRSI(STOCHRSI),
    TRIX(TRIX),
    ULTOSC(ULTOSC),
    WILLR(WILLR),

    // Volume
    AD(AD),
    ADOSC(ADOSC),
    OBV(OBV),

    // Cycle
    HtDcperiod(HtDcperiod),
    HtDcphase(HtDcphase),
    HtPhasor(HtPhasor),
    HtSine(HtSine),
    HtTrendmode(HtTrendmode),

    // Price Transform
    AVGPRICE(AVGPRICE),
    MEDPRICE(MEDPRICE),
    TYPPRICE(TYPPRICE),
    WCLPRICE(WCLPRICE),

    // Volatility
    ATR(ATR),
    NATR(NATR),
    TRANGE(TRANGE),

    // Pattern Recognition
    CDL2CROWS(CDL2CROWS),
    CDL3BLACKCROWS(CDL3BLACKCROWS),
    CDL3INSIDE(CDL3INSIDE),
    CDL3LINESTRIKE(CDL3LINESTRIKE),
    CDL3OUTSIDE(CDL3OUTSIDE),
    CDL3STARSINSOUTH(CDL3STARSINSOUTH),
    CDL3WHITESOLDIERS(CDL3WHITESOLDIERS),
    CDLABANDONEDBABY(CDLABANDONEDBABY),
    CDLADVANCEBLOCK(CDLADVANCEBLOCK),
    CDLBELTHOLD(CDLBELTHOLD),
    CDLBREAKAWAY(CDLBREAKAWAY),
    CDLCLOSINGMARUBOZU(CDLCLOSINGMARUBOZU),
    CDLCONCEALBABYSWALL(CDLCONCEALBABYSWALL),
    CDLCOUNTERATTACK(CDLCOUNTERATTACK),
    CDLDARKCLOUDCOVER(CDLDARKCLOUDCOVER),
    CDLDOJI(CDLDOJI),
    CDLDOJISTAR(CDLDOJISTAR),
    CDLDRAGONFLYDOJI(CDLDRAGONFLYDOJI),
    CDLENGULFING(CDLENGULFING),
    CDLEVENINGDOJISTAR(CDLEVENINGDOJISTAR),
    CDLEVENINGSTAR(CDLEVENINGSTAR),
    CDLGAPSIDESIDEWHITE(CDLGAPSIDESIDEWHITE),
    CDLGRAVESTONEDOJI(CDLGRAVESTONEDOJI),
    CDLHAMMER(CDLHAMMER),
    CDLHANGINGMAN(CDLHANGINGMAN),
    CDLHARAMI(CDLHARAMI),
    CDLHARAMICROSS(CDLHARAMICROSS),
    CDLHIGHWAVE(CDLHIGHWAVE),
    CDLHIKKAKE(CDLHIKKAKE),
    CDLHIKKAKEMOD(CDLHIKKAKEMOD),
    CDLHOMINGPIGEON(CDLHOMINGPIGEON),
    CDLIDENTICAL3CROWS(CDLIDENTICAL3CROWS),
    CDLINNECK(CDLINNECK),
    CDLINVERTEDHAMMER(CDLINVERTEDHAMMER),
    CDLKICKING(CDLKICKING),
    CDLKICKINGBYLENGTH(CDLKICKINGBYLENGTH),
    CDLLADDERBOTTOM(CDLLADDERBOTTOM),
    CDLLONGLEGGEDDOJI(CDLLONGLEGGEDDOJI),
    CDLLONGLINE(CDLLONGLINE),
    CDLMARUBOZU(CDLMARUBOZU),
    CDLMATCHINGLOW(CDLMATCHINGLOW),
    CDLMATHOLD(CDLMATHOLD),
    CDLMORNINGDOJISTAR(CDLMORNINGDOJISTAR),
    CDLMORNINGSTAR(CDLMORNINGSTAR),
    CDLONNECK(CDLONNECK),
    CDLPIERCING(CDLPIERCING),
    CDLRICKSHAWMAN(CDLRICKSHAWMAN),
    CDLRISEFALL3METHODS(CDLRISEFALL3METHODS),
    CDLSEPARATINGLINES(CDLSEPARATINGLINES),
    CDLSHOOTINGSTAR(CDLSHOOTINGSTAR),
    CDLSHORTLINE(CDLSHORTLINE),
    CDLSPINNINGTOP(CDLSPINNINGTOP),
    CDLSTALLEDPATTERN(CDLSTALLEDPATTERN),
    CDLSTICKSANDWICH(CDLSTICKSANDWICH),
    CDLTAKURI(CDLTAKURI),
    CDLTASUKIGAP(CDLTASUKIGAP),
    CDLTHRUSTING(CDLTHRUSTING),
    CDLTRISTAR(CDLTRISTAR),
    CDLUNIQUE3RIVER(CDLUNIQUE3RIVER),
    CDLUPSIDEGAP2CROWS(CDLUPSIDEGAP2CROWS),
    CDLXSIDEGAP3METHODS(CDLXSIDEGAP3METHODS),
}

// 使用宏自动生成所有重复的match实现
impl_indicator!(
    Indicator,
    // Overlap
    BBANDS,
    DEMA,
    EMA,
    HtTrendline,
    KAMA,
    MA,
    MAMA,
    // MAVP,
    MIDPOINT,
    MIDPRICE,
    SAR,
    SAREXT,
    SMA,
    T3,
    TEMA,
    TRIMA,
    WMA,
    // Momentum
    ADX,
    ADXR,
    APO,
    AROON,
    AROONOSC,
    BOP,
    CCI,
    CMO,
    DX,
    MACD,
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
    RSI,
    STOCH,
    STOCHF,
    STOCHRSI,
    TRIX,
    ULTOSC,
    WILLR,
    // Volume
    AD,
    ADOSC,
    OBV,
    // Cycle
    HtDcperiod,
    HtDcphase,
    HtPhasor,
    HtSine,
    HtTrendmode,
    // Price Transform
    AVGPRICE,
    MEDPRICE,
    TYPPRICE,
    WCLPRICE,
    // Volatility
    ATR,
    NATR,
    TRANGE,
    // Pattern Recognition
    CDL2CROWS,
    CDL3BLACKCROWS,
    CDL3INSIDE,
    CDL3LINESTRIKE,
    CDL3OUTSIDE,
    CDL3STARSINSOUTH,
    CDL3WHITESOLDIERS,
    CDLABANDONEDBABY,
    CDLADVANCEBLOCK,
    CDLBELTHOLD,
    CDLBREAKAWAY,
    CDLCLOSINGMARUBOZU,
    CDLCONCEALBABYSWALL,
    CDLCOUNTERATTACK,
    CDLDARKCLOUDCOVER,
    CDLDOJI,
    CDLDOJISTAR,
    CDLDRAGONFLYDOJI,
    CDLENGULFING,
    CDLEVENINGDOJISTAR,
    CDLEVENINGSTAR,
    CDLGAPSIDESIDEWHITE,
    CDLGRAVESTONEDOJI,
    CDLHAMMER,
    CDLHANGINGMAN,
    CDLHARAMI,
    CDLHARAMICROSS,
    CDLHIGHWAVE,
    CDLHIKKAKE,
    CDLHIKKAKEMOD,
    CDLHOMINGPIGEON,
    CDLIDENTICAL3CROWS,
    CDLINNECK,
    CDLINVERTEDHAMMER,
    CDLKICKING,
    CDLKICKINGBYLENGTH,
    CDLLADDERBOTTOM,
    CDLLONGLEGGEDDOJI,
    CDLLONGLINE,
    CDLMARUBOZU,
    CDLMATCHINGLOW,
    CDLMATHOLD,
    CDLMORNINGDOJISTAR,
    CDLMORNINGSTAR,
    CDLONNECK,
    CDLPIERCING,
    CDLRICKSHAWMAN,
    CDLRISEFALL3METHODS,
    CDLSEPARATINGLINES,
    CDLSHOOTINGSTAR,
    CDLSHORTLINE,
    CDLSPINNINGTOP,
    CDLSTALLEDPATTERN,
    CDLSTICKSANDWICH,
    CDLTAKURI,
    CDLTASUKIGAP,
    CDLTHRUSTING,
    CDLTRISTAR,
    CDLUNIQUE3RIVER,
    CDLUPSIDEGAP2CROWS,
    CDLXSIDEGAP3METHODS
);

// impl From<Indicator> for CacheValue {
//     fn from(indicator: Indicator) -> Self {
//         CacheValue::Indicator(indicator)
//     }
// }

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
        self.indicator_value
            .iter()
            .map(|(key, value)| {
                let latest_value = value.last().unwrap();
                (key.clone(), latest_value.clone())
            })
            .collect()
    }
}
