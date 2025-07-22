use super::TALib;
use crate::indicator_engine::talib_bindings::*;
use types::indicator::IndicatorConfig;


impl TALib {
    pub fn lookback(config: &IndicatorConfig) -> usize {
        unsafe {
            match config {
                IndicatorConfig::ACCBANDS(accbands_config) => {
                    let lookback = TA_ACCBANDS_Lookback(accbands_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::AD(_) => {
                    let lookback = TA_AD_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::ADOSC(adosc_config) => {
                    let lookback = TA_ADOSC_Lookback(adosc_config.fast_period, adosc_config.slow_period);
                    return lookback as usize;
                }
                IndicatorConfig::ADX(adx_config) => {
                    let lookback = TA_ADX_Lookback(adx_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::ADXR(adxr_config) => {
                    let lookback = TA_ADXR_Lookback(adxr_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::APO(apo_config) => {
                    let lookback = TA_APO_Lookback(apo_config.fast_period, apo_config.slow_period, apo_config.ma_type.clone() as i32);
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
                IndicatorConfig::ATR(atr_config) => {
                    let lookback = TA_ATR_Lookback(atr_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::BBands(bbands_config) => {
                    let lookback = TA_BBANDS_Lookback(
                        bbands_config.time_period, 
                        bbands_config.dev_up.into(), 
                        bbands_config.dev_down.into(), 
                        bbands_config.ma_type.clone() as i32);
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
                IndicatorConfig::CDL2CROWS(_) => {
                    let lookback = TA_CDL2CROWS_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDL3BLACKCROWS(_) => {
                    let lookback = TA_CDL3BLACKCROWS_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDL3INSIDE(_) => {
                    let lookback = TA_CDL3INSIDE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDL3LINESTRIKE(_) => {
                    let lookback = TA_CDL3LINESTRIKE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDL3OUTSIDE(_) => {
                    let lookback = TA_CDL3OUTSIDE_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDL3STARSINSOUTH(_) => {
                    let lookback = TA_CDL3STARSINSOUTH_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLADVANCEBLOCK(_) => {
                    let lookback = TA_CDLADVANCEBLOCK_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLBELTHOLD(_) => {
                    let lookback = TA_CDLBELTHOLD_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLBREAKAWAY(_) => {
                    let lookback = TA_CDLBREAKAWAY_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLCLOSINGMARUBOZU(_) => {
                    let lookback = TA_CDLCLOSINGMARUBOZU_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLCONCEALBABYSWALL(_) => {
                    let lookback = TA_CDLCONCEALBABYSWALL_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLCOUNTERATTACK(_) => {
                    let lookback = TA_CDLCOUNTERATTACK_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLDARKCLOUDCOVER(cdldarkcloudcover_config) => {
                    let lookback = TA_CDLDARKCLOUDCOVER_Lookback(cdldarkcloudcover_config.penetration.into());
                    return lookback as usize;
                }
                IndicatorConfig::CDLDOJI(_) => {
                    let lookback = TA_CDLDOJI_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDL3WHITESOLDIERS(_) => {
                    let lookback = TA_CDL3WHITESOLDIERS_Lookback();
                    return lookback as usize;
                }
                IndicatorConfig::CDLABANDONEDBABY(cdlabandonedbaby_config) => {
                    let lookback = TA_CDLABANDONEDBABY_Lookback(cdlabandonedbaby_config.penetration.into());
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
                IndicatorConfig::MACDEXT(macdext_config) => {
                    let lookback = TA_MACDEXT_Lookback(
                        macdext_config.fast_period, 
                        macdext_config.slow_period, 
                        macdext_config.signal_period, 
                        macdext_config.fast_ma_type.clone() as i32, 
                        macdext_config.slow_ma_type.clone() as i32, 
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
                IndicatorConfig::MinusDi(minusdi_config) => {
                    let lookback = TA_MINUS_DI_Lookback(minusdi_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::MinusDm(minusdm_config) => {
                    let lookback = TA_MINUS_DM_Lookback(minusdm_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::MOM(mom_config) => {
                    let lookback = TA_MOM_Lookback(mom_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::PlusDi(plusdi_config) => {
                    let lookback = TA_PLUS_DI_Lookback(plusdi_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::PlusDm(plusdm_config) => {
                    let lookback = TA_PLUS_DM_Lookback(plusdm_config.time_period);
                    return lookback as usize;
                }
                IndicatorConfig::PPO(ppo_config) => {
                    let lookback = TA_PPO_Lookback(ppo_config.fast_period, ppo_config.slow_period, ppo_config.ma_type.clone() as i32);
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
                        stoch_config.slow_d_ma_type.clone() as i32
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
                        stochrsi_config.fast_d_ma_type.clone() as i32
                    );
                    return lookback as usize;
                }
                IndicatorConfig::ULTOSC(ultosc_config) => {
                    let lookback = TA_ULTOSC_Lookback(
                        ultosc_config.time_period1, 
                        ultosc_config.time_period2, 
                        ultosc_config.time_period3
                    );
                    return lookback as usize;
                }
                IndicatorConfig::MA(ma_config) => {
                    let lookback = TA_MA_Lookback(ma_config.time_period, ma_config.ma_type.clone() as i32);
                    return lookback as usize;
                }
                IndicatorConfig::MACD(macd_config) => {
                    let lookback = TA_MACD_Lookback(macd_config.fast_period, macd_config.slow_period, macd_config.signal_period);
                    return lookback as usize;
                }
            }
        }
    }
}