use crate::value::*;
use engine::indicator_engine::calculate::CalculateIndicatorFunction;
use types::indicator::indicator_define::momentum::*;
use types::indicator::{IndicatorTrait, MAType, PriceSource};

/// 测试所有动量指标
pub fn test_momentum_indicators() {
    println!("=== 动量指标测试 ===");

    // // 准备测试数据
    // let datasets = generate_test_datasets();

    // for (name, kline_series) in datasets {
    //     println!("\n--- 数据集: {} (长度: {}) ---", name, kline_series.len());
    let kline_series = generate_simple_kline_series(50, None, None);
    // 测试所有动量指标
    test_adx(&kline_series);
    test_adxr(&kline_series);
    test_apo(&kline_series);
    test_aroon(&kline_series);
    test_aroonosc(&kline_series);
    test_bop(&kline_series);
    test_cci(&kline_series);
    test_cmo(&kline_series);
    test_dx(&kline_series);
    test_macd(&kline_series);
    test_macdext(&kline_series);
    test_macdfix(&kline_series);
    test_mfi(&kline_series);
    test_minus_di(&kline_series);
    test_minus_dm(&kline_series);
    test_mom(&kline_series);
    test_plus_di(&kline_series);
    test_plus_dm(&kline_series);
    test_ppo(&kline_series);
    test_roc(&kline_series);
    test_rocp(&kline_series);
    test_rocr(&kline_series);
    test_rocr100(&kline_series);
    test_rsi(&kline_series);
    test_stoch(&kline_series);
    test_stochf(&kline_series);
    test_stochrsi(&kline_series);
    test_trix(&kline_series);
    test_ultosc(&kline_series);
    test_willr(&kline_series);
}

/// 测试ADX指标
fn test_adx(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = ADXConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_adx(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  ADX: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  ADX: 计算失败 - {:?}", e),
    }
}

/// 测试ADXR指标
fn test_adxr(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = ADXRConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_adxr(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  ADXR: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  ADXR: 计算失败 - {:?}", e),
    }
}

/// 测试APO指标
fn test_apo(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = APOConfig {
        fast_period: 12,
        slow_period: 26,
        ma_type: MAType::SMA,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_apo(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  APO: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  APO: 计算失败 - {:?}", e),
    }
}

/// 测试AROON指标
fn test_aroon(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = AROONConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_aroon(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  AROON: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  AROON: 计算失败 - {:?}", e),
    }
}

/// 测试AROONOSC指标
fn test_aroonosc(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = AROONOSCConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_aroonosc(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  AROONOSC: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  AROONOSC: 计算失败 - {:?}", e),
    }
}

/// 测试BOP指标
fn test_bop(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_bop(kline_series.to_vec()) {
        Ok(results) => {
            println!("  BOP: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  BOP: 计算失败 - {:?}", e),
    }
}

/// 测试CCI指标
fn test_cci(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = CCIConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_cci(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  CCI: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CCI: 计算失败 - {:?}", e),
    }
}

/// 测试CMO指标
fn test_cmo(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = CMOConfig {
        time_period: 14,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_cmo(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  CMO: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CMO: 计算失败 - {:?}", e),
    }
}

/// 测试DX指标
fn test_dx(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = DXConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_dx(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  DX: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  DX: 计算失败 - {:?}", e),
    }
}

/// 测试MACD指标
fn test_macd(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = MACDConfig {
        fast_period: 12,
        slow_period: 26,
        signal_period: 9,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_macd(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  MACD: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MACD: 计算失败 - {:?}", e),
    }
}

/// 测试MACDEXT指标
fn test_macdext(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = MACDEXTConfig {
        fast_period: 12,
        fast_ma_type: MAType::EMA,
        slow_period: 26,
        slow_ma_type: MAType::EMA,
        signal_period: 9,
        signal_ma_type: MAType::EMA,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_macdext(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  MACDEXT: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MACDEXT: 计算失败 - {:?}", e),
    }
}

/// 测试MACDFIX指标
fn test_macdfix(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = MACDFIXConfig {
        signal_period: 9,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_macdfix(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  MACDFIX: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MACDFIX: 计算失败 - {:?}", e),
    }
}

/// 测试MFI指标
fn test_mfi(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = MFIConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_mfi(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  MFI: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MFI: 计算失败 - {:?}", e),
    }
}

/// 测试MINUS_DI指标
fn test_minus_di(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = MinusDiConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_minus_di(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  MINUS_DI: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MINUS_DI: 计算失败 - {:?}", e),
    }
}

/// 测试MINUS_DM指标
fn test_minus_dm(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = MinusDmConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_minus_dm(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  MINUS_DM: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MINUS_DM: 计算失败 - {:?}", e),
    }
}

/// 测试MOM指标
fn test_mom(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = MOMConfig {
        time_period: 10,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_mom(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  MOM: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MOM: 计算失败 - {:?}", e),
    }
}

/// 测试PLUS_DI指标
fn test_plus_di(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = PlusDiConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_plus_di(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  PLUS_DI: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  PLUS_DI: 计算失败 - {:?}", e),
    }
}

/// 测试PLUS_DM指标
fn test_plus_dm(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = PlusDmConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_plus_dm(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  PLUS_DM: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  PLUS_DM: 计算失败 - {:?}", e),
    }
}

/// 测试PPO指标
fn test_ppo(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = PPOConfig {
        fast_period: 12,
        slow_period: 26,
        ma_type: MAType::EMA,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_ppo(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  PPO: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  PPO: 计算失败 - {:?}", e),
    }
}

/// 测试ROC指标
fn test_roc(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = ROCConfig {
        time_period: 10,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_roc(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  ROC: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  ROC: 计算失败 - {:?}", e),
    }
}

/// 测试ROCP指标
fn test_rocp(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = ROCPConfig {
        time_period: 10,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_rocp(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  ROCP: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  ROCP: 计算失败 - {:?}", e),
    }
}

/// 测试ROCR指标
fn test_rocr(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = ROCRConfig {
        time_period: 10,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_rocr(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  ROCR: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  ROCR: 计算失败 - {:?}", e),
    }
}

/// 测试ROCR100指标
fn test_rocr100(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = ROCR100Config {
        time_period: 10,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_rocr100(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  ROCR100: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  ROCR100: 计算失败 - {:?}", e),
    }
}

/// 测试RSI指标
fn test_rsi(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = RSIConfig {
        time_period: 14,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_rsi(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  RSI: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  RSI: 计算失败 - {:?}", e),
    }
}

/// 测试STOCH指标
fn test_stoch(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = STOCHConfig {
        fast_k_period: 5,
        slow_k_period: 3,
        slow_k_ma_type: MAType::SMA,
        slow_d_period: 3,
        slow_d_ma_type: MAType::SMA,
    };

    match CalculateIndicatorFunction::calculate_stoch(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  STOCH: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  STOCH: 计算失败 - {:?}", e),
    }
}

/// 测试STOCHF指标
fn test_stochf(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = STOCHFConfig {
        fast_k_period: 5,
        fast_d_period: 3,
        fast_d_ma_type: MAType::SMA,
    };

    match CalculateIndicatorFunction::calculate_stochf(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  STOCHF: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  STOCHF: 计算失败 - {:?}", e),
    }
}

/// 测试STOCHRSI指标
fn test_stochrsi(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = STOCHRSIConfig {
        time_period: 14,
        fast_k_period: 5,
        fast_d_period: 3,
        fast_d_ma_type: MAType::SMA,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_stochrsi(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  STOCHRSI: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  STOCHRSI: 计算失败 - {:?}", e),
    }
}

/// 测试TRIX指标
fn test_trix(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = TRIXConfig {
        time_period: 14,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_trix(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  TRIX: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  TRIX: 计算失败 - {:?}", e),
    }
}

/// 测试ULTOSC指标
fn test_ultosc(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = ULTOSCConfig {
        time_period1: 7,
        time_period2: 14,
        time_period3: 28,
    };

    match CalculateIndicatorFunction::calculate_ultosc(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  ULTOSC: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  ULTOSC: 计算失败 - {:?}", e),
    }
}

/// 测试WILLR指标
fn test_willr(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = WILLRConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_willr(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  WILLR: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  WILLR: 计算失败 - {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_momentum_indicators_integration() {
        test_momentum_indicators();
    }

    // #[test]
    // fn test_rsi_individual() {
    //     let kline_series = generate_simple_kline_series(50, None, None);
    //     test_rsi(&kline_series);
    // }

    // #[test]
    // fn test_macd_individual() {
    //     let kline_series = generate_simple_kline_series(50, None, None);
    //     test_macd(&kline_series);
    // }

    // #[test]
    // fn test_stoch_individual() {
    //     let kline_series = generate_simple_kline_series(50, None, None);
    //     test_stoch(&kline_series);
    // }
}
