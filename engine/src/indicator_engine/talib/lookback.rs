use super::TALib;
use crate::indicator_engine::talib_bindings::*;
use star_river_core::indicator::IndicatorConfig;

impl TALib {
    pub fn lookback(config: &IndicatorConfig) -> usize {
        unsafe {
            match config {
                // Overlap
                IndicatorConfig::BBANDS(bbands_config) => {
                    let lookback = TA_BBANDS_Lookback(
                        bbands_config.time_period,
                        bbands_config.dev_up.into(),
                        bbands_config.dev_down.into(),
                        bbands_config.ma_type.clone() as i32,
                    );
                    return lookback as usize;
                }
                IndicatorConfig::DEMA(dema_config) => {
                    let lookback = TA_DEMA_Lookback(dema_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::EMA(ema_config) => {
                    let lookback = TA_EMA_Lookback(ema_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::HtTrendline(ht_trendline_config) => {
                    let lookback = TA_HT_TRENDLINE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::KAMA(kama_config) => {
                    let lookback = TA_KAMA_Lookback(kama_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::MA(ma_config) => {
                    let lookback =
                        TA_MA_Lookback(ma_config.time_period, ma_config.ma_type.clone() as i32);
                    return lookback as usize;
                }
                IndicatorConfig::MAMA(mama_config) => {
                    let lookback = TA_MAMA_Lookback(
                        mama_config.fast_limit.into(),
                        mama_config.slow_limit.into(),
                    );
                    return lookback as usize;
                }
                // IndicatorConfig::MAVP(mavp_config) => {
                //     let lookback = TA_MAVP_Lookback(mavp_config.min_period, mavp_config.max_period, mavp_config.ma_type.clone() as i32);
                //     return lookback as usize;
                // }
                IndicatorConfig::MIDPOINT(midpoint_config) => {
                    let lookback = TA_MIDPOINT_Lookback(midpoint_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::MIDPRICE(midprice_config) => {
                    let lookback = TA_MIDPRICE_Lookback(midprice_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::SAR(sar_config) => {
                    let lookback =
                        TA_SAR_Lookback(sar_config.acceleration.into(), sar_config.maximum.into());
                    return lookback as usize;
                }
                IndicatorConfig::SAREXT(sarext_config) => {
                    let lookback = TA_SAREXT_Lookback(
                        sarext_config.start_value.into(),
                        sarext_config.offset_on_reverse.into(),
                        sarext_config.acceleration_init_long.into(),
                        sarext_config.acceleration_long.into(),
                        sarext_config.acceleration_max_long.into(),
                        sarext_config.acceleration_init_short.into(),
                        sarext_config.acceleration_short.into(),
                        sarext_config.acceleration_max_short.into(),
                    );
                    return lookback as usize;
                }
                IndicatorConfig::SMA(sma_config) => {
                    let lookback = TA_SMA_Lookback(sma_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::T3(t3_config) => {
                    let lookback = TA_T3_Lookback(t3_config.time_period, t3_config.v_factor.into());
                    return lookback as usize;
                }
                IndicatorConfig::TEMA(tema_config) => {
                    let lookback = TA_TEMA_Lookback(tema_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::TRIMA(trima_config) => {
                    let lookback = TA_TRIMA_Lookback(trima_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::WMA(wma_config) => {
                    let lookback = TA_WMA_Lookback(wma_config.time_period);
                    return lookback as usize;
                }
                // Momentum
                IndicatorConfig::ADX(adx_config) => {
                    let lookback = TA_ADX_Lookback(adx_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::ADXR(adxr_config) => {
                    let lookback = TA_ADXR_Lookback(adxr_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::APO(apo_config) => {
                    let lookback = TA_APO_Lookback(
                        apo_config.fast_period,
                        apo_config.slow_period,
                        apo_config.ma_type.clone() as i32,
                    );
                    return lookback as usize;
                }
                IndicatorConfig::AROON(aroon_config) => {
                    let lookback = TA_AROON_Lookback(aroon_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::AROONOSC(aroonosc_config) => {
                    let lookback = TA_AROONOSC_Lookback(aroonosc_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::BOP(_) => {
                    let lookback = TA_BOP_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CCI(cci_config) => {
                    let lookback = TA_CCI_Lookback(cci_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::CMO(cmo_config) => {
                    let lookback = TA_CMO_Lookback(cmo_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::DX(dx_config) => {
                    let lookback = TA_DX_Lookback(dx_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::MACD(macd_config) => {
                    let lookback = TA_MACD_Lookback(
                        macd_config.fast_period,
                        macd_config.slow_period,
                        macd_config.signal_period,
                    );
                    return lookback as usize;
                }
                IndicatorConfig::MACDEXT(macdext_config) => {
                    let lookback = TA_MACDEXT_Lookback(
                        macdext_config.fast_period,
                        macdext_config.fast_ma_type.clone() as i32,
                        macdext_config.slow_period,
                        macdext_config.slow_ma_type.clone() as i32,
                        macdext_config.signal_period,
                        macdext_config.signal_ma_type.clone() as i32,
                    );
                    return lookback as usize;
                }
                IndicatorConfig::MACDFIX(macdfix_config) => {
                    let lookback = TA_MACDFIX_Lookback(macdfix_config.signal_period);
                    return lookback as usize;
                }
                IndicatorConfig::MFI(mfi_config) => {
                    let lookback = TA_MFI_Lookback(mfi_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::MinusDi(minus_di_config) => {
                    let lookback = TA_MINUS_DI_Lookback(minus_di_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::MinusDm(minus_dm_config) => {
                    let lookback = TA_MINUS_DM_Lookback(minus_dm_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::MOM(mom_config) => {
                    let lookback = TA_MOM_Lookback(mom_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::PlusDi(plus_di_config) => {
                    let lookback = TA_PLUS_DI_Lookback(plus_di_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::PlusDm(plus_dm_config) => {
                    let lookback = TA_PLUS_DM_Lookback(plus_dm_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::PPO(ppo_config) => {
                    let lookback = TA_PPO_Lookback(
                        ppo_config.fast_period,
                        ppo_config.slow_period,
                        ppo_config.ma_type.clone() as i32,
                    );
                    return lookback as usize;
                }
                IndicatorConfig::ROC(roc_config) => {
                    let lookback = TA_ROC_Lookback(roc_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::ROCP(rocp_config) => {
                    let lookback = TA_ROCP_Lookback(rocp_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::ROCR(rocr_config) => {
                    let lookback = TA_ROCR_Lookback(rocr_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::ROCR100(rocr100_config) => {
                    let lookback = TA_ROCR100_Lookback(rocr100_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::RSI(rsi_config) => {
                    let lookback = TA_RSI_Lookback(rsi_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::STOCH(stoch_config) => {
                    let lookback = TA_STOCH_Lookback(
                        stoch_config.fast_k_period,
                        stoch_config.slow_k_period,
                        stoch_config.slow_k_ma_type.clone() as i32,
                        stoch_config.slow_d_period,
                        stoch_config.slow_d_ma_type.clone() as i32,
                    );
                    return lookback as usize;
                }
                IndicatorConfig::STOCHF(stochf_config) => {
                    let lookback = TA_STOCHF_Lookback(
                        stochf_config.fast_k_period,
                        stochf_config.fast_d_period,
                        stochf_config.fast_d_ma_type.clone() as i32,
                    );
                    return lookback as usize;
                }
                IndicatorConfig::STOCHRSI(stochrsi_config) => {
                    let lookback = TA_STOCHRSI_Lookback(
                        stochrsi_config.time_period,
                        stochrsi_config.fast_k_period,
                        stochrsi_config.fast_d_period,
                        stochrsi_config.fast_d_ma_type.clone() as i32,
                    );
                    return lookback as usize;
                }
                IndicatorConfig::TRIX(trix_config) => {
                    let lookback = TA_TRIX_Lookback(trix_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::ULTOSC(ultosc_config) => {
                    let lookback = TA_ULTOSC_Lookback(
                        ultosc_config.time_period1,
                        ultosc_config.time_period2,
                        ultosc_config.time_period3,
                    );
                    return lookback as usize;
                }
                IndicatorConfig::WILLR(willr_config) => {
                    let lookback = TA_WILLR_Lookback(willr_config.time_period);
                    return lookback as usize;
                }

                // Volume
                IndicatorConfig::AD(ad_config) => {
                    let lookback = TA_AD_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::ADOSC(adosc_config) => {
                    let lookback =
                        TA_ADOSC_Lookback(adosc_config.fast_period, adosc_config.slow_period);
                    return lookback as usize;
                }
                IndicatorConfig::OBV(obv_config) => {
                    let lookback = TA_OBV_Lookback();
                    return lookback as usize;
                }
                // Cycle
                IndicatorConfig::HtDcperiod(ht_dcperiod_config) => {
                    let lookback = TA_HT_DCPERIOD_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::HtDcphase(ht_dcphase_config) => {
                    let lookback = TA_HT_DCPHASE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::HtPhasor(ht_phasor_config) => {
                    let lookback = TA_HT_PHASOR_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::HtSine(ht_sine_config) => {
                    let lookback = TA_HT_SINE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::HtTrendmode(ht_trendmode_config) => {
                    let lookback = TA_HT_TRENDMODE_Lookback();
                    return lookback as usize;
                }

                // Price Transform
                IndicatorConfig::AVGPRICE(avgprice_config) => {
                    let lookback = TA_AVGPRICE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::MEDPRICE(medprice_config) => {
                    let lookback = TA_MEDPRICE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::TYPPRICE(typprice_config) => {
                    let lookback = TA_TYPPRICE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::WCLPRICE(wclprice_config) => {
                    let lookback = TA_WCLPRICE_Lookback();
                    return lookback as usize;
                }
                // Volatility
                IndicatorConfig::ATR(atr_config) => {
                    let lookback = TA_ATR_Lookback(atr_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::NATR(natr_config) => {
                    let lookback = TA_NATR_Lookback(natr_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::TRANGE(trange_config) => {
                    let lookback = TA_TRANGE_Lookback();
                    return lookback as usize;
                }

                // Pattern Recognition
                IndicatorConfig::CDL2CROWS(cdl2crows_config) => {
                    let lookback = TA_CDL2CROWS_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDL3BLACKCROWS(cdl3blackcrows_config) => {
                    let lookback = TA_CDL3BLACKCROWS_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDL3INSIDE(cdl3inside_config) => {
                    let lookback = TA_CDL3INSIDE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDL3OUTSIDE(cdl3outside_config) => {
                    let lookback = TA_CDL3OUTSIDE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLABANDONEDBABY(cdlabandonedbaby_config) => {
                    let lookback =
                        TA_CDLABANDONEDBABY_Lookback(cdlabandonedbaby_config.penetration.into());
                    return lookback as usize;
                }
                IndicatorConfig::CDLADVANCEBLOCK(cdladvanceblock_config) => {
                    let lookback = TA_CDLADVANCEBLOCK_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLBELTHOLD(cdlbelthold_config) => {
                    let lookback = TA_CDLBELTHOLD_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLBREAKAWAY(cdlbreakaway_config) => {
                    let lookback = TA_CDLBREAKAWAY_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLCLOSINGMARUBOZU(cdlclosingmarubozu_config) => {
                    let lookback = TA_CDLCLOSINGMARUBOZU_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLCONCEALBABYSWALL(cdlconcealbabyswall_config) => {
                    let lookback = TA_CDLCONCEALBABYSWALL_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLCOUNTERATTACK(cdlcounterattack_config) => {
                    let lookback = TA_CDLCOUNTERATTACK_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLDARKCLOUDCOVER(cdldarkcloudcover_config) => {
                    let lookback =
                        TA_CDLDARKCLOUDCOVER_Lookback(cdldarkcloudcover_config.penetration.into());
                    return lookback as usize;
                }
                IndicatorConfig::CDLDOJI(cdldoji_config) => {
                    let lookback = TA_CDLDOJI_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLDOJISTAR(cdldojistar_config) => {
                    let lookback = TA_CDLDOJISTAR_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLENGULFING(cdlengulfing_config) => {
                    let lookback = TA_CDLENGULFING_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLEVENINGDOJISTAR(cdleveningdojistar_config) => {
                    let lookback = TA_CDLEVENINGDOJISTAR_Lookback(
                        cdleveningdojistar_config.penetration.into(),
                    );
                    return lookback as usize;
                }
                IndicatorConfig::CDLEVENINGSTAR(cdleveningstar_config) => {
                    let lookback =
                        TA_CDLEVENINGSTAR_Lookback(cdleveningstar_config.penetration.into());
                    return lookback as usize;
                }
                IndicatorConfig::CDLGAPSIDESIDEWHITE(cdlgapsidesidewhite_config) => {
                    let lookback = TA_CDLGAPSIDESIDEWHITE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLGRAVESTONEDOJI(cdlgravestonedoji_config) => {
                    let lookback = TA_CDLGRAVESTONEDOJI_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLHAMMER(cdlhammer_config) => {
                    let lookback = TA_CDLHAMMER_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLHANGINGMAN(cdlhangingman_config) => {
                    let lookback = TA_CDLHANGINGMAN_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLHIKKAKE(cdlhiikake_config) => {
                    let lookback = TA_CDLHIKKAKE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLHIKKAKEMOD(cdlhiikakemod_config) => {
                    let lookback = TA_CDLHIKKAKEMOD_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLHOMINGPIGEON(cdlhomingpigeon_config) => {
                    let lookback = TA_CDLHOMINGPIGEON_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLIDENTICAL3CROWS(cdlidentical3crows_config) => {
                    let lookback = TA_CDLIDENTICAL3CROWS_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLINVERTEDHAMMER(cdlinvertedhammer_config) => {
                    let lookback = TA_CDLINVERTEDHAMMER_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLKICKING(cdlkicking_config) => {
                    let lookback = TA_CDLKICKING_Lookback();
                    return lookback as usize;
                }

                // 补充所有缺失的 CDL 指标
                IndicatorConfig::CDL3LINESTRIKE(cdl3linestrike_config) => {
                    let lookback = TA_CDL3LINESTRIKE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDL3STARSINSOUTH(cdl3starsinsouth_config) => {
                    let lookback = TA_CDL3STARSINSOUTH_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDL3WHITESOLDIERS(cdl3whitesoldiers_config) => {
                    let lookback = TA_CDL3WHITESOLDIERS_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLDRAGONFLYDOJI(cdldragonflydoji_config) => {
                    let lookback = TA_CDLDRAGONFLYDOJI_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLHARAMI(cdlharami_config) => {
                    let lookback = TA_CDLHARAMI_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLHARAMICROSS(cdlharamicross_config) => {
                    let lookback = TA_CDLHARAMICROSS_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLHIGHWAVE(cdlhighwave_config) => {
                    let lookback = TA_CDLHIGHWAVE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLINNECK(cdlinneck_config) => {
                    let lookback = TA_CDLINNECK_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLKICKINGBYLENGTH(cdlkickingbylength_config) => {
                    let lookback = TA_CDLKICKINGBYLENGTH_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLLADDERBOTTOM(cdlladderbottom_config) => {
                    let lookback = TA_CDLLADDERBOTTOM_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLLONGLEGGEDDOJI(cdllongleggeddoji_config) => {
                    let lookback = TA_CDLLONGLEGGEDDOJI_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLLONGLINE(cdllongline_config) => {
                    let lookback = TA_CDLLONGLINE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLMARUBOZU(cdlmarubozu_config) => {
                    let lookback = TA_CDLMARUBOZU_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLMATCHINGLOW(cdlmatchinglow_config) => {
                    let lookback = TA_CDLMATCHINGLOW_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLMATHOLD(cdlmathold_config) => {
                    let lookback = TA_CDLMATHOLD_Lookback(cdlmathold_config.penetration.into());
                    return lookback as usize;
                }
                IndicatorConfig::CDLMORNINGDOJISTAR(cdlmorningdojistar_config) => {
                    let lookback = TA_CDLMORNINGDOJISTAR_Lookback(
                        cdlmorningdojistar_config.penetration.into(),
                    );
                    return lookback as usize;
                }
                IndicatorConfig::CDLMORNINGSTAR(cdlmorningstar_config) => {
                    let lookback =
                        TA_CDLMORNINGSTAR_Lookback(cdlmorningstar_config.penetration.into());
                    return lookback as usize;
                }
                IndicatorConfig::CDLONNECK(cdlonneck_config) => {
                    let lookback = TA_CDLONNECK_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLPIERCING(cdlpiercing_config) => {
                    let lookback = TA_CDLPIERCING_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLRICKSHAWMAN(cdlrickshawman_config) => {
                    let lookback = TA_CDLRICKSHAWMAN_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLRISEFALL3METHODS(cdlrisefall3methods_config) => {
                    let lookback = TA_CDLRISEFALL3METHODS_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLSEPARATINGLINES(cdlseparatinglines_config) => {
                    let lookback = TA_CDLSEPARATINGLINES_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLSHOOTINGSTAR(cdlshootingstar_config) => {
                    let lookback = TA_CDLSHOOTINGSTAR_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLSHORTLINE(cdlshortline_config) => {
                    let lookback = TA_CDLSHORTLINE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLSPINNINGTOP(cdlspinningtop_config) => {
                    let lookback = TA_CDLSPINNINGTOP_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLSTALLEDPATTERN(cdlstalledpattern_config) => {
                    let lookback = TA_CDLSTALLEDPATTERN_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLSTICKSANDWICH(cdlsticksandwich_config) => {
                    let lookback = TA_CDLSTICKSANDWICH_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLTAKURI(cdltakuri_config) => {
                    let lookback = TA_CDLTAKURI_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLTASUKIGAP(cdltasukigap_config) => {
                    let lookback = TA_CDLTASUKIGAP_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLTHRUSTING(cdlthrusting_config) => {
                    let lookback = TA_CDLTHRUSTING_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLTRISTAR(cdltristar_config) => {
                    let lookback = TA_CDLTRISTAR_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLUNIQUE3RIVER(cdlunique3river_config) => {
                    let lookback = TA_CDLUNIQUE3RIVER_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLUPSIDEGAP2CROWS(cdlupsidegap2crows_config) => {
                    let lookback = TA_CDLUPSIDEGAP2CROWS_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLXSIDEGAP3METHODS(cdlxsidegap3methods_config) => {
                    let lookback = TA_CDLXSIDEGAP3METHODS_Lookback();
                    return lookback as usize;
                }
            }
        }
    }
}
