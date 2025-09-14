pub mod calculate_macros;
pub mod define_calculate_fun;

use crate::cache_engine::CacheEngine;
use crate::calculate_fn;
use crate::calculate_fn_snake;
use crate::indicator_engine::talib::TALib;
use star_river_core::cache::CacheValue;
use star_river_core::cache::Key;
use star_river_core::indicator::Indicator;
use star_river_core::indicator::IndicatorConfig;
use star_river_core::indicator::PriceSource;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

pub struct CalculateIndicatorFunction;

impl CalculateIndicatorFunction {
    pub async fn calculate_indicator(
        cache_engine: Arc<Mutex<CacheEngine>>,
        kline_key: Key,
        indicator_config: IndicatorConfig,
        ignore_config: bool, // 是否忽略指标计算配置中所需要的长度，而是使用缓存中的所有数据
    ) -> Result<Vec<Indicator>, String> {
        tracing::info!("indicator_config: {:?}", indicator_config);

        let lookback = TALib::lookback(&indicator_config);
        let kline_series: Vec<Arc<CacheValue>>;

        if ignore_config {
            kline_series = cache_engine
                .lock()
                .await
                .get_cache_value(&kline_key, None, None)
                .await;
        } else {
            kline_series = cache_engine
                .lock()
                .await
                .get_cache_value(&kline_key, None, Some(lookback as u32 + 1))
                .await;
            if kline_series.len() < (lookback + 1) as usize {
                return Err(format!(
                    "kline_series length is less than lookback: {:?}",
                    lookback
                ));
            }
        }

        match &indicator_config {
            // Overlap
            IndicatorConfig::BBANDS(bbands_config) => {
                CalculateIndicatorFunction::calculate_bbands(kline_series, bbands_config)
            }
            IndicatorConfig::DEMA(dema_config) => {
                CalculateIndicatorFunction::calculate_dema(kline_series, dema_config)
            }
            IndicatorConfig::EMA(ema_config) => {
                CalculateIndicatorFunction::calculate_ema(kline_series, ema_config)
            }
            IndicatorConfig::HtTrendline(ht_trendline_config) => {
                CalculateIndicatorFunction::calculate_ht_trendline(
                    kline_series,
                    &ht_trendline_config,
                )
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
            IndicatorConfig::BOP(_) => CalculateIndicatorFunction::calculate_bop(kline_series),
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
            IndicatorConfig::AD(_) => CalculateIndicatorFunction::calculate_ad(kline_series),
            IndicatorConfig::ADOSC(adosc_config) => {
                CalculateIndicatorFunction::calculate_adosc(kline_series, adosc_config)
            }
            IndicatorConfig::OBV(_) => CalculateIndicatorFunction::calculate_obv(kline_series),

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
                CalculateIndicatorFunction::calculate_ht_trendmode(
                    kline_series,
                    &ht_trendmode_config,
                )
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
                CalculateIndicatorFunction::calculate_cdlabandonedbaby(
                    kline_series,
                    cdlabandonedbaby_config,
                )
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
                CalculateIndicatorFunction::calculate_cdldarkcloudcover(
                    kline_series,
                    cdldarkcloudcover_config,
                )
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
                CalculateIndicatorFunction::calculate_cdleveningdojistar(
                    kline_series,
                    cdleveningdojistar_config,
                )
            }
            IndicatorConfig::CDLEVENINGSTAR(cdleveningstar_config) => {
                CalculateIndicatorFunction::calculate_cdleveningstar(
                    kline_series,
                    cdleveningstar_config,
                )
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
                CalculateIndicatorFunction::calculate_cdlmorningdojistar(
                    kline_series,
                    cdlmorningdojistar_config,
                )
            }
            IndicatorConfig::CDLMORNINGSTAR(cdlmorningstar_config) => {
                CalculateIndicatorFunction::calculate_cdlmorningstar(
                    kline_series,
                    cdlmorningstar_config,
                )
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

    fn get_price_source_and_datetime(
        price_source: &PriceSource,
        kline_series: Vec<Arc<CacheValue>>,
    ) -> Result<(Vec<DateTime<Utc>>, Vec<f64>), String> {
        let (timestamp_list, price_list) = match price_source {
            PriceSource::Close => {
                let (timestamp_list, close_list): (Vec<DateTime<Utc>>, Vec<f64>) = kline_series
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        v.as_kline()
                            .ok_or_else(|| format!("Invalid kline data at index {}", i))
                            .map(|kline| (kline.datetime, kline.close))
                    })
                    .collect::<Result<Vec<(DateTime<Utc>, f64)>, _>>()?
                    .into_iter()
                    .unzip();
                (timestamp_list, close_list)
            }
            PriceSource::Open => {
                let (timestamp_list, open_list): (Vec<DateTime<Utc>>, Vec<f64>) = kline_series
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        v.as_kline()
                            .ok_or_else(|| format!("Invalid kline data at index {}", i))
                            .map(|kline| (kline.datetime, kline.open))
                    })
                    .collect::<Result<Vec<(DateTime<Utc>, f64)>, _>>()?
                    .into_iter()
                    .unzip();
                (timestamp_list, open_list)
            }
            PriceSource::High => {
                let (timestamp_list, high_list): (Vec<DateTime<Utc>>, Vec<f64>) = kline_series
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        v.as_kline()
                            .ok_or_else(|| format!("Invalid kline data at index {}", i))
                            .map(|kline| (kline.datetime, kline.high))
                    })
                    .collect::<Result<Vec<(DateTime<Utc>, f64)>, _>>()?
                    .into_iter()
                    .unzip();
                (timestamp_list, high_list)
            }
            PriceSource::Low => {
                let (timestamp_list, low_list): (Vec<DateTime<Utc>>, Vec<f64>) = kline_series
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        v.as_kline()
                            .ok_or_else(|| format!("Invalid kline data at index {}", i))
                            .map(|kline| (kline.datetime, kline.low))
                    })
                    .collect::<Result<Vec<(DateTime<Utc>, f64)>, _>>()?
                    .into_iter()
                    .unzip();
                (timestamp_list, low_list)
            }
        };

        Ok((timestamp_list, price_list))
    }

    // 获取高开低收+时间戳
    fn get_tohlcv(
        kline_series: Vec<Arc<CacheValue>>,
    ) -> Result<(Vec<DateTime<Utc>>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), String> {
        let mut timestamp_list = Vec::new();
        let mut open_list = Vec::new();
        let mut high_list = Vec::new();
        let mut low_list = Vec::new();
        let mut close_list = Vec::new();
        let mut volume_list = Vec::new();

        for (i, v) in kline_series.iter().enumerate() {
            let kline = v
                .as_kline()
                .ok_or_else(|| format!("Invalid kline data at index {}", i))?;
            timestamp_list.push(kline.datetime);
            open_list.push(kline.open);
            high_list.push(kline.high);
            low_list.push(kline.low);
            close_list.push(kline.close);
            volume_list.push(kline.volume);
        }

        Ok((
            timestamp_list,
            open_list,
            high_list,
            low_list,
            close_list,
            volume_list,
        ))
    }
}
