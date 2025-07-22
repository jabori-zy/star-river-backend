// pub mod rsi;
// pub mod ma;
// pub mod macd;
// pub mod bbands;
pub mod talib_macros;
pub mod lookback;

use crate::indicator_engine::talib_bindings::*;
use crate::indicator_engine::talib_error::TalibError;
use crate::talib_fn;
use crate::talib_snake_fn;
use types::indicator::indicator::*;
use types::indicator::Indicator;

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

        Ok(Self)
    }



    talib_fn!(
        ACCBANDS,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(upper: f64), (middle: f64), (lower: f64)],
    );

    talib_fn!(
        AD,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64]), (volume: &[f64])],
        talib_params => [],
        output => [(ad: f64)],
    );

    talib_fn!(
        ADOSC,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64]), (volume: &[f64])],
        talib_params => [(fast_period: i32), (slow_period: i32)],
        output => [(adosc: f64)],
    );

    talib_fn!(
        ADX,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(adx: f64)],
    );

    talib_fn!(
        ADXR,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(adxr: f64)],
    );

    talib_fn!(
        APO,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [(fast_period: i32), (slow_period: i32), (ma_type: i32)],
        output => [(apo: f64)],
    );

    talib_fn!(
        AROON,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(aroon_down: f64), (aroon_up: f64)],
    );

    talib_fn!(
        AROONOSC,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(aroon_osc: f64)],
    );

    talib_fn!(
        ATR,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(atr: f64)],
    );

    talib_fn!(
        BBands,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [
            (time_period: i32),
            (dev_up: f64),
            (dev_down: f64),
            (ma_type: i32),
        ],
        output => [(upper: f64), (middle: f64), (lower: f64)],
    );

    talib_fn!(
        BOP,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(bop: f64)],
    );


    talib_fn!(
        CCI,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(cci: f64)],
    );

    talib_fn!(
        MA,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [
            (time_period: i32),
            (ma_type: i32),
        ],
        output => [(ma: f64)],
    );

    talib_fn!(
        MACD,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
            (signal_period: i32),
        ],
        output => [(macd: f64), (signal: f64), (histogram: f64)],
    );

    

    

    talib_fn!(
        RSI,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(rsi: f64)],
    );

    talib_fn!(
        CMO,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(cmo: f64)],
    );

    talib_fn!(
        DX,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(dx: f64)],
    );

    talib_fn!(
        MACDEXT,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [(fast_period: i32),(fast_ma_type: i32),(slow_period: i32),(slow_ma_type: i32),(signal_period: i32),(signal_ma_type: i32)],
        output => [(macd: f64), (signal: f64), (histogram: f64)],
    );

    talib_fn!(
        MACDFIX,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [(signal_period: i32)],
        output => [(macd: f64), (signal: f64), (histogram: f64)],
    );

    talib_fn!(
        MFI,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64]), (volume: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(mfi: f64)],
    );


    talib_snake_fn!(
        MinusDi,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(minus_di: f64)],
    );

    talib_snake_fn!(
        MinusDm,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(minus_dm: f64)],
    );

    talib_fn!(
        MOM,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(momentum: f64)],
    );

    talib_snake_fn!(
        PlusDi,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(plus_di: f64)],
    );

    talib_snake_fn!(
        PlusDm,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(plus_dm: f64)],
    );

    talib_fn!(
        PPO,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [(fast_period: i32), (slow_period: i32), (ma_type: i32)],
        output => [(ppo: f64)],
    );


    talib_fn!(
        ROC,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(roc: f64)],
    );

    talib_fn!(
        ROCP,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(rocp: f64)],
    );

    talib_fn!(
        ROCR,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(rocr: f64)],
    );

    talib_fn!(
        ROCR100,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32),],
        output => [(rocr100: f64)],
    );

    talib_fn!(
        STOCH,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [
            (fast_k_period: i32), 
            (slow_k_period: i32), 
            (slow_k_ma_type: i32), 
            (slow_d_period: i32), 
            (slow_d_ma_type: i32),
        ],
        output => [(slow_k: f64), (slow_d: f64)],
    );

    talib_fn!(
        STOCHF,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [
            (fast_k_period: i32), 
            (fast_d_period: i32), 
            (fast_d_ma_type: i32),
        ],
        output => [(fast_k: f64), (fast_d: f64)],
    );

    talib_fn!(
        STOCHRSI,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [
            (time_period: i32), 
            (fast_k_period: i32), 
            (fast_d_period: i32), 
            (fast_d_ma_type: i32),
        ],
        output => [(fast_k: f64), (fast_d: f64)],
    );

    talib_fn!(
        ULTOSC,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period1: i32), (time_period2: i32), (time_period3: i32)],
        output => [(ultosc: f64)],
    );


    // // 蜡烛图形态识别函数

    talib_fn!(
        CDL2CROWS,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(two_crows: i32)],
    );

    talib_fn!(
        CDL3BLACKCROWS,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_black_crows: i32)],
    );

    talib_fn!(
        CDL3INSIDE,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_inside: i32)],
    );

    talib_fn!(
        CDL3LINESTRIKE,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_line_strike: i32)],
    );

    talib_fn!(
        CDL3OUTSIDE,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_outside: i32)],
    );

    talib_fn!(
        CDL3STARSINSOUTH,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_stars_in_south: i32)],
    );

    talib_fn!(
        CDL3WHITESOLDIERS,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_white_soldiers: i32)],
    );

    talib_fn!(
        CDLABANDONEDBABY,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(cdl_abandoned_baby: i32)],
    );

    talib_fn!(
        CDLADVANCEBLOCK,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(cdl_advance_block: i32)],
    );

    talib_fn!(
        CDLBELTHOLD,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(belthold: i32)],
    );

    talib_fn!(
        CDLBREAKAWAY,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(breakaway: i32)],
    );

    talib_fn!(
        CDLCLOSINGMARUBOZU,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(closing_marubozu: i32)],
    );

    talib_fn!(
        CDLCONCEALBABYSWALL,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(conceal_baby_wall: i32)],
    );

    talib_fn!(
        CDLCOUNTERATTACK,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(counter_attack: i32)],
    );

    talib_fn!(
        CDLDARKCLOUDCOVER,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(dark_cloud_cover: i32)],
    );

    talib_fn!(
        CDLDOJI,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(doji: i32)],
    );


    pub fn shutdown() {
        unsafe {
            TA_Shutdown();
        }
    }
}
