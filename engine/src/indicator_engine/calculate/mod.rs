
pub mod calculate_macros;
pub mod define_calculate_fun;

use tokio::sync::Mutex;
use crate::cache_engine::CacheEngine;
use types::indicator::Indicator;
use std::sync::Arc;
use types::cache::Key;
use types::indicator::IndicatorConfig;
use types::cache::CacheValue;
use crate::indicator_engine::talib::TALib;
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
            // Overlap
            IndicatorConfig::BBands(bbands_config) => {
                CalculateIndicatorFunction::calculate_bbands(kline_series, bbands_config)
            }
            IndicatorConfig::DEMA(dema_config) => {
                CalculateIndicatorFunction::calculate_dema(kline_series, dema_config)
            }
            IndicatorConfig::EMA(ema_config) => {
                CalculateIndicatorFunction::calculate_ema(kline_series, ema_config)
            }
            IndicatorConfig::HtTrendline(ht_trendline_config) => {
                CalculateIndicatorFunction::calculate_ht_trendline(kline_series, &ht_trendline_config)
            }
            IndicatorConfig::KAMA(kama_config) => {
                CalculateIndicatorFunction::calculate_kama(kline_series, kama_config)
            }
            IndicatorConfig::MA(ma_config) => {
                CalculateIndicatorFunction::calculate_ma(kline_series, ma_config)
            }
            IndicatorConfig::MAMA(mama_config) => {
                CalculateIndicatorFunction::calculate_mama(kline_series, mama_config)
            }
            IndicatorConfig::MIDPOINT(midpoint_config) => {
                CalculateIndicatorFunction::calculate_midpoint(kline_series, midpoint_config)
            }
            IndicatorConfig::MIDPRICE(midprice_config) => {
                CalculateIndicatorFunction::calculate_midprice(kline_series, midprice_config)
            }
            IndicatorConfig::SAR(sar_config) => {
                CalculateIndicatorFunction::calculate_sar(kline_series, sar_config)
            }
            IndicatorConfig::SAREXT(sarext_config) => {
                CalculateIndicatorFunction::calculate_sarext(kline_series, sarext_config)
            }
            IndicatorConfig::SMA(sma_config) => {
                CalculateIndicatorFunction::calculate_sma(kline_series, sma_config)
            }
            IndicatorConfig::T3(t3_config) => {
                CalculateIndicatorFunction::calculate_t3(kline_series, t3_config)
            }
            IndicatorConfig::TEMA(tema_config) => {
                CalculateIndicatorFunction::calculate_tema(kline_series, tema_config)
            }
            IndicatorConfig::TRIMA(trima_config) => {
                CalculateIndicatorFunction::calculate_trima(kline_series, trima_config)
            }
            IndicatorConfig::WMA(wma_config) => {
                CalculateIndicatorFunction::calculate_wma(kline_series, wma_config)
            }

            // Momentum
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
            IndicatorConfig::MACD(macd_config) => {
                CalculateIndicatorFunction::calculate_macd(kline_series, macd_config)
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
            IndicatorConfig::TRIX(trix_config) => {
                CalculateIndicatorFunction::calculate_trix(kline_series, trix_config)
            }
            IndicatorConfig::ULTOSC(ultosc_config) => {
                CalculateIndicatorFunction::calculate_ultosc(kline_series, ultosc_config)
            }
            IndicatorConfig::WILLR(willr_config) => {
                CalculateIndicatorFunction::calculate_willr(kline_series, willr_config)
            }

            // Volume
            IndicatorConfig::AD(_) => {
                CalculateIndicatorFunction::calculate_ad(kline_series)
            }
            IndicatorConfig::ADOSC(adosc_config) => {
                CalculateIndicatorFunction::calculate_adosc(kline_series, adosc_config)
            }
            IndicatorConfig::OBV(_) => {
                CalculateIndicatorFunction::calculate_obv(kline_series)
            }

            // Cycle
            IndicatorConfig::HtDcperiod(ht_dcperiod_config) => {
                CalculateIndicatorFunction::calculate_ht_dcperiod(kline_series, &ht_dcperiod_config)
            }
            IndicatorConfig::HtDcphase(ht_dcphase_config) => {
                CalculateIndicatorFunction::calculate_ht_dcphase(kline_series, &ht_dcphase_config)
            }
            IndicatorConfig::HtPhasor(ht_phasor_config) => {
                CalculateIndicatorFunction::calculate_ht_phasor(kline_series, &ht_phasor_config)
            }
            IndicatorConfig::HtSine(ht_sine_config) => {
                CalculateIndicatorFunction::calculate_ht_sine(kline_series, &ht_sine_config)
            }
            IndicatorConfig::HtTrendmode(ht_trendmode_config) => {
                CalculateIndicatorFunction::calculate_ht_trendmode(kline_series, &ht_trendmode_config)
            }

            // Price Transform
            IndicatorConfig::AVGPRICE(_) => {
                CalculateIndicatorFunction::calculate_avgprice(kline_series)
            }
            IndicatorConfig::MEDPRICE(_) => {
                CalculateIndicatorFunction::calculate_medprice(kline_series)
            }
            IndicatorConfig::TYPPRICE(_) => {
                CalculateIndicatorFunction::calculate_typprice(kline_series)
            }
            IndicatorConfig::WCLPRICE(_) => {
                CalculateIndicatorFunction::calculate_wclprice(kline_series)
            }

            // Volatility
            IndicatorConfig::ATR(atr_config) => {
                CalculateIndicatorFunction::calculate_atr(kline_series, atr_config)
            }
            IndicatorConfig::NATR(natr_config) => {
                CalculateIndicatorFunction::calculate_natr(kline_series, natr_config)
            }
            IndicatorConfig::TRANGE(_) => {
                CalculateIndicatorFunction::calculate_trange(kline_series)
            }

            // Pattern Recognition
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
            IndicatorConfig::CDL3WHITESOLDIERS(_) => {
                CalculateIndicatorFunction::calculate_cdl3whitesoldiers(kline_series)
            }
            IndicatorConfig::CDLABANDONEDBABY(cdlabandonedbaby_config) => {
                CalculateIndicatorFunction::calculate_cdlabandonedbaby(kline_series, cdlabandonedbaby_config)
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
            IndicatorConfig::CDLDOJISTAR(_) => {
                CalculateIndicatorFunction::calculate_cdldojistar(kline_series)
            }
            IndicatorConfig::CDLDRAGONFLYDOJI(_) => {
                CalculateIndicatorFunction::calculate_cdldragonflydoji(kline_series)
            }
            IndicatorConfig::CDLENGULFING(_) => {
                CalculateIndicatorFunction::calculate_cdlengulfing(kline_series)
            }
            IndicatorConfig::CDLEVENINGDOJISTAR(cdleveningdojistar_config) => {
                CalculateIndicatorFunction::calculate_cdleveningdojistar(kline_series, cdleveningdojistar_config)
            }
            IndicatorConfig::CDLEVENINGSTAR(cdleveningstar_config) => {
                CalculateIndicatorFunction::calculate_cdleveningstar(kline_series, cdleveningstar_config)
            }
            IndicatorConfig::CDLGAPSIDESIDEWHITE(_) => {
                CalculateIndicatorFunction::calculate_cdlgapsidesidewhite(kline_series)
            }
            IndicatorConfig::CDLGRAVESTONEDOJI(_) => {
                CalculateIndicatorFunction::calculate_cdlgravestonedoji(kline_series)
            }
            IndicatorConfig::CDLHAMMER(_) => {
                CalculateIndicatorFunction::calculate_cdlhammer(kline_series)
            }
            IndicatorConfig::CDLHANGINGMAN(_) => {
                CalculateIndicatorFunction::calculate_cdlhangingman(kline_series)
            }
            IndicatorConfig::CDLHARAMI(_) => {
                CalculateIndicatorFunction::calculate_cdlharami(kline_series)
            }
            IndicatorConfig::CDLHARAMICROSS(_) => {
                CalculateIndicatorFunction::calculate_cdlharamicross(kline_series)
            }
            IndicatorConfig::CDLHIGHWAVE(_) => {
                CalculateIndicatorFunction::calculate_cdlhighwave(kline_series)
            }
            IndicatorConfig::CDLHIKKAKE(_) => {
                CalculateIndicatorFunction::calculate_cdlhikkake(kline_series)
            }
            IndicatorConfig::CDLHIKKAKEMOD(_) => {
                CalculateIndicatorFunction::calculate_cdlhikkakemod(kline_series)
            }
            IndicatorConfig::CDLHOMINGPIGEON(_) => {
                CalculateIndicatorFunction::calculate_cdlhomingpigeon(kline_series)
            }
            IndicatorConfig::CDLIDENTICAL3CROWS(_) => {
                CalculateIndicatorFunction::calculate_cdlidentical3crows(kline_series)
            }
            IndicatorConfig::CDLINNECK(_) => {
                CalculateIndicatorFunction::calculate_cdlinneck(kline_series)
            }
            IndicatorConfig::CDLINVERTEDHAMMER(_) => {
                CalculateIndicatorFunction::calculate_cdlinvertedhammer(kline_series)
            }
            IndicatorConfig::CDLKICKING(_) => {
                CalculateIndicatorFunction::calculate_cdlkicking(kline_series)
            }
            IndicatorConfig::CDLKICKINGBYLENGTH(_) => {
                CalculateIndicatorFunction::calculate_cdlkickingbylength(kline_series)
            }
            IndicatorConfig::CDLLADDERBOTTOM(_) => {
                CalculateIndicatorFunction::calculate_cdlladderbottom(kline_series)
            }
            IndicatorConfig::CDLLONGLEGGEDDOJI(_) => {
                CalculateIndicatorFunction::calculate_cdllongleggeddoji(kline_series)
            }
            IndicatorConfig::CDLLONGLINE(_) => {
                CalculateIndicatorFunction::calculate_cdllongline(kline_series)
            }
            IndicatorConfig::CDLMARUBOZU(_) => {
                CalculateIndicatorFunction::calculate_cdlmarubozu(kline_series)
            }
            IndicatorConfig::CDLMATCHINGLOW(_) => {
                CalculateIndicatorFunction::calculate_cdlmatchinglow(kline_series)
            }
            IndicatorConfig::CDLMATHOLD(cdlmathold_config) => {
                CalculateIndicatorFunction::calculate_cdlmathold(kline_series, cdlmathold_config)
            }
            IndicatorConfig::CDLMORNINGDOJISTAR(cdlmorningdojistar_config) => {
                CalculateIndicatorFunction::calculate_cdlmorningdojistar(kline_series, cdlmorningdojistar_config)
            }
            IndicatorConfig::CDLMORNINGSTAR(cdlmorningstar_config) => {
                CalculateIndicatorFunction::calculate_cdlmorningstar(kline_series, cdlmorningstar_config)
            }
            IndicatorConfig::CDLONNECK(_) => {
                CalculateIndicatorFunction::calculate_cdlonneck(kline_series)
            }
            IndicatorConfig::CDLPIERCING(_) => {
                CalculateIndicatorFunction::calculate_cdlpiercing(kline_series)
            }
            IndicatorConfig::CDLRICKSHAWMAN(_) => {
                CalculateIndicatorFunction::calculate_cdlrickshawman(kline_series)
            }
            IndicatorConfig::CDLRISEFALL3METHODS(_) => {
                CalculateIndicatorFunction::calculate_cdlrisefall3methods(kline_series)
            }
            IndicatorConfig::CDLSEPARATINGLINES(_) => {
                CalculateIndicatorFunction::calculate_cdlseparatinglines(kline_series)
            }
            IndicatorConfig::CDLSHOOTINGSTAR(_) => {
                CalculateIndicatorFunction::calculate_cdlshootingstar(kline_series)
            }
            IndicatorConfig::CDLSHORTLINE(_) => {
                CalculateIndicatorFunction::calculate_cdlshortline(kline_series)
            }
            IndicatorConfig::CDLSPINNINGTOP(_) => {
                CalculateIndicatorFunction::calculate_cdlspinningtop(kline_series)
            }
            IndicatorConfig::CDLSTALLEDPATTERN(_) => {
                CalculateIndicatorFunction::calculate_cdlstalledpattern(kline_series)
            }
            IndicatorConfig::CDLSTICKSANDWICH(_) => {
                CalculateIndicatorFunction::calculate_cdlsticksandwich(kline_series)
            }
            IndicatorConfig::CDLTAKURI(_) => {
                CalculateIndicatorFunction::calculate_cdltakuri(kline_series)
            }
            IndicatorConfig::CDLTASUKIGAP(_) => {
                CalculateIndicatorFunction::calculate_cdltasukigap(kline_series)
            }
            IndicatorConfig::CDLTHRUSTING(_) => {
                CalculateIndicatorFunction::calculate_cdlthrusting(kline_series)
            }
            IndicatorConfig::CDLTRISTAR(_) => {
                CalculateIndicatorFunction::calculate_cdltristar(kline_series)
            }
            IndicatorConfig::CDLUNIQUE3RIVER(_) => {
                CalculateIndicatorFunction::calculate_cdlunique3river(kline_series)
            }
            IndicatorConfig::CDLUPSIDEGAP2CROWS(_) => {
                CalculateIndicatorFunction::calculate_cdlupsidegap2crows(kline_series)
            }
            IndicatorConfig::CDLXSIDEGAP3METHODS(_) => {
                CalculateIndicatorFunction::calculate_cdlxsidegap3methods(kline_series)
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


    // calculate_fn!(ACCBANDS,
    //     input => [high,low,close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(AD,
    //     input => [high,low,close,volume]
    // );

    // calculate_fn!(ADOSC,
    //     input => [high,low,close,volume],
    //     talib_params => [
    //         (fast_period: i32),
    //         (slow_period: i32),
    //     ]
    // );

    // calculate_fn!(ADX,
    //     input => [high,low,close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(ADXR,
    //     input => [high,low,close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(APO,
    //     input => [close],
    //     talib_params => [
    //         (fast_period: i32),
    //         (slow_period: i32),
    //         (ma_type: MAType),
    //     ]
    // );

    // calculate_fn!(AROON,
    //     input => [high,low],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(AROONOSC,
    //     input => [high,low],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(ATR,
    //     input => [high,low,close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(BBands,
    //     talib_params => [
    //         (time_period: i32),
    //         (dev_up: f64),
    //         (dev_down: f64),
    //         (ma_type: MAType),
    //     ]
    // );

    // calculate_fn!(MA,
    //     talib_params => [
    //         (time_period: i32),
    //         (ma_type: MAType),
    //     ]
    // );

    // calculate_fn!(MACD,
    //     talib_params => [
    //         (fast_period: i32),
    //         (slow_period: i32),
    //         (signal_period: i32),
    //     ]
    // );

    // calculate_fn!(RSI,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    

    // calculate_fn!(BOP,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CCI,
    //     input => [high,low,close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(CMO,
    //     input => [close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(DX,
    //     input => [high,low,close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(MACDEXT,
    //     input => [close],
    //     talib_params => [
    //         (fast_period: i32),
    //         (fast_ma_type: MAType),
    //         (slow_period: i32),
    //         (slow_ma_type: MAType),
    //         (signal_period: i32),
    //         (signal_ma_type: MAType),
    //     ]
    // );

    // calculate_fn!(MACDFIX,
    //     input => [close],
    //     talib_params => [
    //         (signal_period: i32),
    //     ]
    // );

    // calculate_fn!(MFI,
    //     input => [high,low,close,volume],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn_snake!(MinusDi,
    //     input => [high,low,close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn_snake!(MinusDm,
    //     input => [high,low],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(MOM,
    //     input => [close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn_snake!(PlusDi,
    //     input => [high,low,close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn_snake!(PlusDm,
    //     input => [high,low],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(PPO,
    //     talib_params => [
    //         (fast_period: i32),
    //         (slow_period: i32),
    //         (ma_type: MAType),
    //     ]
    // );

    // calculate_fn!(ROC,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(ROCP,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(ROCR,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(ROCR100,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );
    
    
    // calculate_fn!(STOCH,
    //     input => [high,low,close],
    //     talib_params => [
    //         (fast_k_period: i32), 
    //         (slow_k_period: i32), 
    //         (slow_k_ma_type: MAType), 
    //         (slow_d_period: i32), 
    //         (slow_d_ma_type: MAType),
    //     ]
    // );

    // calculate_fn!(STOCHF,
    //     input => [high,low,close],
    //     talib_params => [
    //         (fast_k_period: i32), 
    //         (fast_d_period: i32), 
    //         (fast_d_ma_type: MAType),
    //     ]
    // );

    // calculate_fn!(STOCHRSI,
    //     input => [close],
    //     talib_params => [
    //         (time_period: i32), 
    //         (fast_k_period: i32), 
    //         (fast_d_period: i32), 
    //         (fast_d_ma_type: MAType),
    //     ]
    // );

    // calculate_fn!(ULTOSC,
    //     input => [high,low,close],
    //     talib_params => [
    //         (time_period1: i32), 
    //         (time_period2: i32),
    //         (time_period3: i32),
    //     ]
    // );

    // // // 蜡烛形态识别

    // calculate_fn!(CDL2CROWS,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDL3BLACKCROWS,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDL3INSIDE,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDL3LINESTRIKE,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDL3OUTSIDE,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDL3STARSINSOUTH,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDL3WHITESOLDIERS,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDLABANDONEDBABY,
    //     input => [open,high,low,close],
    //     talib_params => [(penetration: f64)]
    // );

    // calculate_fn!(CDLADVANCEBLOCK,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDLBELTHOLD,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDLBREAKAWAY,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDLCLOSINGMARUBOZU,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDLCONCEALBABYSWALL,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDLCOUNTERATTACK,
    //     input => [open,high,low,close]
    // );

    // calculate_fn!(CDLDARKCLOUDCOVER,
    //     input => [open,high,low,close],
    //     talib_params => [(penetration: f64)]
    // );

    // calculate_fn!(CDLDOJI,
    //     input => [open,high,low,close]
    // );

    // // Overlap indicators
    // calculate_fn!(DEMA,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(EMA,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(HtTrendline,
    //     input => [close]
    // );

    // calculate_fn!(KAMA,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(MAMA,
    //     talib_params => [
    //         (fast_limit: f64),
    //         (slow_limit: f64),
    //     ]
    // );

    // calculate_fn!(MAVP,
    //     input => [close, periods],
    //     talib_params => [
    //         (min_period: i32),
    //         (max_period: i32),
    //         (ma_type: MAType),
    //     ]
    // );

    // calculate_fn!(MIDPOINT,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(MIDPRICE,
    //     input => [high, low],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(SAR,
    //     input => [high, low],
    //     talib_params => [
    //         (acceleration: f64),
    //         (maximum: f64),
    //     ]
    // );

    // calculate_fn!(SAREXT,
    //     input => [high, low],
    //     talib_params => [
    //         (start_value: f64),
    //         (offset_on_reverse: f64),
    //         (acceleration_init_long: f64),
    //         (acceleration_long: f64),
    //         (acceleration_max_long: f64),
    //         (acceleration_init_short: f64),
    //         (acceleration_short: f64),
    //         (acceleration_max_short: f64),
    //     ]
    // );

    // calculate_fn!(SMA,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(T3,
    //     talib_params => [
    //         (time_period: i32),
    //         (v_factor: f64),
    //     ]
    // );

    // calculate_fn!(TEMA,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(TRIMA,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(WMA,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // // Volume indicators
    // calculate_fn!(OBV,
    //     input => [close, volume]
    // );

    // // Cycle indicators
    // calculate_fn!(HtDcPeriod,
    //     input => [close]
    // );

    // calculate_fn!(HtDcPhase,
    //     input => [close]
    // );

    // calculate_fn!(HtPhasor,
    //     input => [close]
    // );

    // calculate_fn!(HtSine,
    //     input => [close]
    // );

    // calculate_fn!(HtTrendMode,
    //     input => [close]
    // );

    // // Price Transform indicators
    // calculate_fn!(AVGPRICE,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(MEDPRICE,
    //     input => [high, low]
    // );

    // calculate_fn!(TYPPRICE,
    //     input => [high, low, close]
    // );

    // calculate_fn!(WCLPRICE,
    //     input => [high, low, close]
    // );

    // // Volatility indicators
    // calculate_fn!(NATR,
    //     input => [high, low, close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(TRANGE,
    //     input => [high, low, close]
    // );

    // // Momentum indicators
    // calculate_fn!(TRIX,
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // calculate_fn!(WILLR,
    //     input => [high, low, close],
    //     talib_params => [
    //         (time_period: i32),
    //     ]
    // );

    // // Additional Pattern Recognition indicators
    // calculate_fn!(CDLDOJISTAR,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLDRAGONFLYDOJI,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLENGULFING,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLEVENINGDOJISTAR,
    //     input => [open, high, low, close],
    //     talib_params => [(penetration: f64)]
    // );

    // calculate_fn!(CDLEVENINGSTAR,
    //     input => [open, high, low, close],
    //     talib_params => [(penetration: f64)]
    // );

    // calculate_fn!(CDLGAPSIDESIDEWHITE,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLGRAVESTONEDOJI,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLHAMMER,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLHANGINGMAN,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLHARAMI,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLHARAMICROSS,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLHIGHWAVE,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLHIKKAKE,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLHIKKAKEMOD,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLHOMINGPIGEON,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLIDENTICAL3CROWS,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLINNECK,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLINVERTEDHAMMER,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLKICKING,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLKICKINGBYLENGTH,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLLADDERBOTTOM,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLLONGLEGGEDDOJI,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLLONGLINE,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLMARUBOZU,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLMATCHINGLOW,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLMATHOLD,
    //     input => [open, high, low, close],
    //     talib_params => [(penetration: f64)]
    // );

    // calculate_fn!(CDLMORNINGDOJISTAR,
    //     input => [open, high, low, close],
    //     talib_params => [(penetration: f64)]
    // );

    // calculate_fn!(CDLMORNINGSTAR,
    //     input => [open, high, low, close],
    //     talib_params => [(penetration: f64)]
    // );

    // calculate_fn!(CDLONNECK,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLPIERCING,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLRICKSHAWMAN,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLRISEFALL3METHODS,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLSEPARATINGLINES,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLSHOOTINGSTAR,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLSHORTLINE,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLSPINNINGTOP,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLSTALLEDPATTERN,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLSTICKSANDWICH,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLTAKURI,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLTASUKIGAP,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLTHRUSTING,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLTRISTAR,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLUNIQUE3RIVER,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLUPSIDEGAP2CROWS,
    //     input => [open, high, low, close]
    // );

    // calculate_fn!(CDLXSIDEGAP3METHODS,
    //     input => [open, high, low, close]
    // );

}
