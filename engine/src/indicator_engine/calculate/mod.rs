
pub mod calculate_macros;

use tokio::sync::Mutex;
use crate::cache_engine::CacheEngine;
use types::indicator::Indicator;
use std::sync::Arc;
use types::cache::Key;
use types::indicator::IndicatorConfig;
use types::cache::CacheValue;
use crate::indicator_engine::talib::TALib;
use types::indicator::indicator::*;
use types::indicator::PriceSource;
use crate::calculate_fn;
use crate::calculate_fn_snake;



pub struct CalculateIndicatorFunction;



impl CalculateIndicatorFunction {

    pub async fn calculate_indicator(
        cache_engine: Arc<Mutex<CacheEngine>>, 
        kline_key: Key,
        indicator_config: IndicatorConfig,
        ignore_config: bool // 是否忽略指标计算配置中所需要的长度，而是使用缓存中的所有数据
    ) -> Result<Vec<Indicator>, String> {
        tracing::info!("indicator_config: {:?}", indicator_config);

        let lookback = TALib::lookback(&indicator_config);
        let kline_series: Vec<Arc<CacheValue>>;
        
        if ignore_config  {
            kline_series = cache_engine.lock().await.get_cache_value(&kline_key, None, None).await;
        } else {
            kline_series = cache_engine.lock().await.get_cache_value(&kline_key, None,Some(lookback as u32 +1)).await;
            if kline_series.len() < (lookback+1) as usize {
                return Err(format!("kline_series length is less than lookback: {:?}", lookback));
            }
        }

        match &indicator_config {
            IndicatorConfig::ACCBANDS(accbands_config) => {
                CalculateIndicatorFunction::calculate_accbands(kline_series, accbands_config)
            }
            IndicatorConfig::AD(_) => {
                CalculateIndicatorFunction::calculate_ad(kline_series)
            }
            IndicatorConfig::ADOSC(adosc_config) => {
                CalculateIndicatorFunction::calculate_adosc(kline_series, adosc_config)
            }
            IndicatorConfig::ADX(adx_config) => {
                CalculateIndicatorFunction::calculate_adx(kline_series, adx_config)
            }
            IndicatorConfig::ADXR(adxr_config) => {
                CalculateIndicatorFunction::calculate_adxr(kline_series, adxr_config)
            }
            IndicatorConfig::APO(apo_config) => {
                CalculateIndicatorFunction::calculate_apo(kline_series, apo_config)
            }
            IndicatorConfig::AROON(aroon_config) => {
                CalculateIndicatorFunction::calculate_aroon(kline_series, aroon_config)
            }
            IndicatorConfig::AROONOSC(aroonosc_config) => {
                CalculateIndicatorFunction::calculate_aroonosc(kline_series, aroonosc_config)
            }
            IndicatorConfig::ATR(atr_config) => {
                CalculateIndicatorFunction::calculate_atr(kline_series, atr_config)
            }
            IndicatorConfig::BBands(bbands_config) => {
                CalculateIndicatorFunction::calculate_bbands(kline_series, bbands_config)
            }
            IndicatorConfig::BOP(_) => {
                CalculateIndicatorFunction::calculate_bop(kline_series)
            }
            IndicatorConfig::CCI(cci_config) => {
                CalculateIndicatorFunction::calculate_cci(kline_series, cci_config)
            }
            IndicatorConfig::CMO(cmo_config) => {
                CalculateIndicatorFunction::calculate_cmo(kline_series, cmo_config)
            }
            IndicatorConfig::DX(dx_config) => {
                CalculateIndicatorFunction::calculate_dx(kline_series, dx_config)
            }
            IndicatorConfig::MACDEXT(macdext_config) => {
                CalculateIndicatorFunction::calculate_macdext(kline_series, macdext_config)
            }
            IndicatorConfig::MACDFIX(macdfix_config) => {
                CalculateIndicatorFunction::calculate_macdfix(kline_series, macdfix_config)
            }
            IndicatorConfig::MFI(mfi_config) => {
                CalculateIndicatorFunction::calculate_mfi(kline_series, mfi_config)
            }
            IndicatorConfig::MinusDi(minusdi_config) => {
                CalculateIndicatorFunction::calculate_minus_di(kline_series, minusdi_config)
            }
            IndicatorConfig::MinusDm(minusdm_config) => {
                CalculateIndicatorFunction::calculate_minus_dm(kline_series, minusdm_config)
            }
            IndicatorConfig::MOM(mom_config) => {
                CalculateIndicatorFunction::calculate_mom(kline_series, mom_config)
            }
            IndicatorConfig::PlusDi(plusdi_config) => {
                CalculateIndicatorFunction::calculate_plus_di(kline_series, plusdi_config)
            }
            IndicatorConfig::PlusDm(plusdm_config) => {
                CalculateIndicatorFunction::calculate_plus_dm(kline_series, plusdm_config)
            }
            IndicatorConfig::PPO(ppo_config) => {
                CalculateIndicatorFunction::calculate_ppo(kline_series, ppo_config)
            }
            IndicatorConfig::ROC(roc_config) => {
                CalculateIndicatorFunction::calculate_roc(kline_series, roc_config)
            }
            IndicatorConfig::ROCP(rocp_config) => {
                CalculateIndicatorFunction::calculate_rocp(kline_series, rocp_config)
            }
            IndicatorConfig::ROCR(rocr_config) => {
                CalculateIndicatorFunction::calculate_rocr(kline_series, rocr_config)
            }
            IndicatorConfig::ROCR100(rocr100_config) => {
                CalculateIndicatorFunction::calculate_rocr100(kline_series, rocr100_config)
            }
            IndicatorConfig::RSI(rsi_config) => {
                CalculateIndicatorFunction::calculate_rsi(kline_series, rsi_config)
            }
            IndicatorConfig::STOCH(stoch_config) => {
                CalculateIndicatorFunction::calculate_stoch(kline_series, stoch_config)
            }
            IndicatorConfig::STOCHF(stochf_config) => {
                CalculateIndicatorFunction::calculate_stochf(kline_series, stochf_config)
            }
            IndicatorConfig::STOCHRSI(stochrsi_config) => {
                CalculateIndicatorFunction::calculate_stochrsi(kline_series, stochrsi_config)
            }
            IndicatorConfig::ULTOSC(ultosc_config) => {
                CalculateIndicatorFunction::calculate_ultosc(kline_series, ultosc_config)
            }
            IndicatorConfig::MA(ma_config) => {
                CalculateIndicatorFunction::calculate_ma(kline_series, ma_config)
            }
            IndicatorConfig::MACD(macd_config) => {
                CalculateIndicatorFunction::calculate_macd(kline_series, macd_config)
            }
            IndicatorConfig::CDL2CROWS(_) => {
                CalculateIndicatorFunction::calculate_cdl2crows(kline_series)
            }
            IndicatorConfig::CDL3BLACKCROWS(_) => {
                CalculateIndicatorFunction::calculate_cdl3blackcrows(kline_series)
            }
            IndicatorConfig::CDL3INSIDE(_) => {
                CalculateIndicatorFunction::calculate_cdl3inside(kline_series)
            }
            IndicatorConfig::CDL3LINESTRIKE(_) => {
                CalculateIndicatorFunction::calculate_cdl3linestrike(kline_series)
            }
            IndicatorConfig::CDL3OUTSIDE(_) => {
                CalculateIndicatorFunction::calculate_cdl3outside(kline_series)
            }
            IndicatorConfig::CDL3STARSINSOUTH(_) => {
                CalculateIndicatorFunction::calculate_cdl3starsinsouth(kline_series)
            }
            IndicatorConfig::CDLADVANCEBLOCK(_) => {
                CalculateIndicatorFunction::calculate_cdladvanceblock(kline_series)
            }
            IndicatorConfig::CDLBELTHOLD(_) => {
                CalculateIndicatorFunction::calculate_cdlbelthold(kline_series)
            }
            IndicatorConfig::CDLBREAKAWAY(_) => {
                CalculateIndicatorFunction::calculate_cdlbreakaway(kline_series)
            }
            IndicatorConfig::CDLCLOSINGMARUBOZU(_) => {
                CalculateIndicatorFunction::calculate_cdlclosingmarubozu(kline_series)
            }
            IndicatorConfig::CDLCONCEALBABYSWALL(_) => {
                CalculateIndicatorFunction::calculate_cdlconcealbabyswall(kline_series)
            }
            IndicatorConfig::CDLCOUNTERATTACK(_) => {
                CalculateIndicatorFunction::calculate_cdlcounterattack(kline_series)
            }
            IndicatorConfig::CDLDARKCLOUDCOVER(cdldarkcloudcover_config) => {
                CalculateIndicatorFunction::calculate_cdldarkcloudcover(kline_series, cdldarkcloudcover_config)
            }
            IndicatorConfig::CDLDOJI(_) => {
                CalculateIndicatorFunction::calculate_cdldoji(kline_series)
            }
            IndicatorConfig::CDL3WHITESOLDIERS(_) => {
                CalculateIndicatorFunction::calculate_cdl3whitesoldiers(kline_series)
            }
            IndicatorConfig::CDLABANDONEDBABY(cdlabandonedbaby_config) => {
                CalculateIndicatorFunction::calculate_cdlabandonedbaby(kline_series, cdlabandonedbaby_config)
            }
        }
    }


    fn get_price_source_and_timestamp(price_source: &PriceSource, kline_series: Vec<Arc<CacheValue>>) -> Result<(Vec<i64>, Vec<f64>), String> {
        let (timestamp_list, price_list) = match price_source {
            PriceSource::Close => {
                let (timestamp_list, close_list): (Vec<i64>, Vec<f64>) = kline_series
                .iter()
                .enumerate()
                .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {}", i)).map(|kline| (kline.timestamp, kline.close)))
                .collect::<Result<Vec<(i64, f64)>, _>>()?
                .into_iter()
                .unzip();
                (timestamp_list, close_list)
            },
            PriceSource::Open => {
                let (timestamp_list, open_list): (Vec<i64>, Vec<f64>) = kline_series
                .iter()
                .enumerate()
                .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {}", i)).map(|kline| (kline.timestamp, kline.open)))
                .collect::<Result<Vec<(i64, f64)>, _>>()?
                .into_iter()
                .unzip();
                (timestamp_list, open_list)
            },
            PriceSource::High => {
                let (timestamp_list, high_list): (Vec<i64>, Vec<f64>) = kline_series
                .iter()
                .enumerate()
                .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {}", i)).map(|kline| (kline.timestamp, kline.high)))
                .collect::<Result<Vec<(i64, f64)>, _>>()?
                .into_iter()
                .unzip();
                (timestamp_list, high_list)
            },
            PriceSource::Low => {
                let (timestamp_list, low_list): (Vec<i64>, Vec<f64>) = kline_series
                .iter()
                .enumerate()
                .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {}", i)).map(|kline| (kline.timestamp, kline.low)))
                .collect::<Result<Vec<(i64, f64)>, _>>()?
                .into_iter()
                .unzip();
                (timestamp_list, low_list)
            },
        };

        Ok((timestamp_list, price_list))
    }


    // 获取高开低收+时间戳
    fn get_tohlcv(kline_series: Vec<Arc<CacheValue>>) -> Result<(Vec<i64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), String> {
        let mut timestamp_list = Vec::new();
        let mut open_list = Vec::new();
        let mut high_list = Vec::new();
        let mut low_list = Vec::new();
        let mut close_list = Vec::new();
        let mut volume_list = Vec::new();

        for (i, v) in kline_series.iter().enumerate() {
            let kline = v.as_kline().ok_or_else(|| format!("Invalid kline data at index {}", i))?;
            timestamp_list.push(kline.timestamp);
            open_list.push(kline.open);
            high_list.push(kline.high);
            low_list.push(kline.low);
            close_list.push(kline.close);
            volume_list.push(kline.volume);
        }

        Ok((timestamp_list, open_list, high_list, low_list, close_list, volume_list))
    }


    calculate_fn!(ACCBANDS,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(AD,
        input => [high,low,close,volume]
    );

    calculate_fn!(ADOSC,
        input => [high,low,close,volume],
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
        ]
    );

    calculate_fn!(ADX,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(ADXR,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(APO,
        input => [close],
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
            (ma_type: MAType),
        ]
    );

    calculate_fn!(AROON,
        input => [high,low],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(AROONOSC,
        input => [high,low],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(ATR,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(BBands,
        talib_params => [
            (time_period: i32),
            (dev_up: f64),
            (dev_down: f64),
            (ma_type: MAType),
        ]
    );

    calculate_fn!(MA,
        talib_params => [
            (time_period: i32),
            (ma_type: MAType),
        ]
    );

    calculate_fn!(MACD,
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
            (signal_period: i32),
        ]
    );

    calculate_fn!(RSI,
        talib_params => [
            (time_period: i32),
        ]
    );

    

    calculate_fn!(BOP,
        input => [open,high,low,close]
    );

    calculate_fn!(CCI,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(CMO,
        input => [close],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(DX,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(MACDEXT,
        input => [close],
        talib_params => [
            (fast_period: i32),
            (fast_ma_type: MAType),
            (slow_period: i32),
            (slow_ma_type: MAType),
            (signal_period: i32),
            (signal_ma_type: MAType),
        ]
    );

    calculate_fn!(MACDFIX,
        input => [close],
        talib_params => [
            (signal_period: i32),
        ]
    );

    calculate_fn!(MFI,
        input => [high,low,close,volume],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn_snake!(MinusDi,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn_snake!(MinusDm,
        input => [high,low],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(MOM,
        input => [close],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn_snake!(PlusDi,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn_snake!(PlusDm,
        input => [high,low],
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(PPO,
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
            (ma_type: MAType),
        ]
    );

    calculate_fn!(ROC,
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(ROCP,
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(ROCR,
        talib_params => [
            (time_period: i32),
        ]
    );

    calculate_fn!(ROCR100,
        talib_params => [
            (time_period: i32),
        ]
    );
    
    
    calculate_fn!(STOCH,
        input => [high,low,close],
        talib_params => [
            (fast_k_period: i32), 
            (slow_k_period: i32), 
            (slow_k_ma_type: MAType), 
            (slow_d_period: i32), 
            (slow_d_ma_type: MAType),
        ]
    );

    calculate_fn!(STOCHF,
        input => [high,low,close],
        talib_params => [
            (fast_k_period: i32), 
            (fast_d_period: i32), 
            (fast_d_ma_type: MAType),
        ]
    );

    calculate_fn!(STOCHRSI,
        input => [close],
        talib_params => [
            (time_period: i32), 
            (fast_k_period: i32), 
            (fast_d_period: i32), 
            (fast_d_ma_type: MAType),
        ]
    );

    calculate_fn!(ULTOSC,
        input => [high,low,close],
        talib_params => [
            (time_period1: i32), 
            (time_period2: i32),
            (time_period3: i32),
        ]
    );

    // // 蜡烛形态识别

    calculate_fn!(CDL2CROWS,
        input => [open,high,low,close]
    );

    calculate_fn!(CDL3BLACKCROWS,
        input => [open,high,low,close]
    );

    calculate_fn!(CDL3INSIDE,
        input => [open,high,low,close]
    );

    calculate_fn!(CDL3LINESTRIKE,
        input => [open,high,low,close]
    );

    calculate_fn!(CDL3OUTSIDE,
        input => [open,high,low,close]
    );

    calculate_fn!(CDL3STARSINSOUTH,
        input => [open,high,low,close]
    );

    calculate_fn!(CDL3WHITESOLDIERS,
        input => [open,high,low,close]
    );

    calculate_fn!(CDLABANDONEDBABY,
        input => [open,high,low,close],
        talib_params => [(penetration: f64)]
    );

    calculate_fn!(CDLADVANCEBLOCK,
        input => [open,high,low,close]
    );

    calculate_fn!(CDLBELTHOLD,
        input => [open,high,low,close]
    );

    calculate_fn!(CDLBREAKAWAY,
        input => [open,high,low,close]
    );

    calculate_fn!(CDLCLOSINGMARUBOZU,
        input => [open,high,low,close]
    );

    calculate_fn!(CDLCONCEALBABYSWALL,
        input => [open,high,low,close]
    );

    calculate_fn!(CDLCOUNTERATTACK,
        input => [open,high,low,close]
    );

    calculate_fn!(CDLDARKCLOUDCOVER,
        input => [open,high,low,close],
        talib_params => [(penetration: f64)]
    );

    calculate_fn!(CDLDOJI,
        input => [open,high,low,close]
    );

}
