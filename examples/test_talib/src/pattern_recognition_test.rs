use crate::value::*;
use engine::indicator_engine::calculate::CalculateIndicatorFunction;
use ordered_float::OrderedFloat;
use star_river_core::indicator::IndicatorTrait;
use star_river_core::indicator::indicator_define::pattern_recognition::*;

/// 测试K线形态识别指标
pub fn test_pattern_recognition_indicators() {
    println!("=== K线形态识别指标测试 ===");

    // // 准备测试数据
    // let datasets = generate_test_datasets();

    // for (name, kline_series) in datasets {
    //     println!("\n--- 数据集: {} (长度: {}) ---", name, kline_series.len());
    let kline_series = generate_simple_kline_series(100, None, None);
    // 测试常用K线形态指标
    test_cdl2crows(&kline_series);
    test_cdl3blackcrows(&kline_series);
    test_cdl3inside(&kline_series);
    test_cdl3linestrike(&kline_series);
    test_cdl3outside(&kline_series);
    test_cdl3starsinsouth(&kline_series);
    test_cdl3whitesoldiers(&kline_series);
    test_cdlabandonedbaby(&kline_series);
    test_cdladvanceblock(&kline_series);
    test_cdlbelthold(&kline_series);
    test_cdlbreakaway(&kline_series);
    test_cdlclosingmarubozu(&kline_series);
    test_cdlconcealbabyswall(&kline_series);
    test_cdlcounterattack(&kline_series);
    test_cdldarkcloudcover(&kline_series);
    test_cdldoji(&kline_series);
    test_cdldojistar(&kline_series);
    test_cdldragonflydoji(&kline_series);
    test_cdlengulfing(&kline_series);
    test_cdleveningdojistar(&kline_series);
    test_cdleveningstar(&kline_series);
    test_cdlgapsidesidewhite(&kline_series);
    test_cdlgravestonedoji(&kline_series);
    test_cdlhammer(&kline_series);
    test_cdlhangingman(&kline_series);
    test_cdlharami(&kline_series);
    test_cdlharamicross(&kline_series);
    test_cdlhighwave(&kline_series);
    test_cdlhikkake(&kline_series);
    test_cdlhikkakemod(&kline_series);
    test_cdlhomingpigeon(&kline_series);
    test_cdlidentical3crows(&kline_series);
    test_cdlinneck(&kline_series);
    test_cdlinvertedhammer(&kline_series);
    test_cdlkicking(&kline_series);
    test_cdlkickingbylength(&kline_series);
    test_cdlladderbottom(&kline_series);
    test_cdllongleggeddoji(&kline_series);
    test_cdllongline(&kline_series);
    test_cdlmarubozu(&kline_series);
    test_cdlmatchinglow(&kline_series);
    test_cdlmathold(&kline_series);
    test_cdlmorningdojistar(&kline_series);
    test_cdlmorningstar(&kline_series);
    test_cdlonneck(&kline_series);
    test_cdlpiercing(&kline_series);
    test_cdlrickshawman(&kline_series);
    test_cdlrisefall3methods(&kline_series);
    test_cdlseparatinglines(&kline_series);
    test_cdlshootingstar(&kline_series);
    test_cdlshortline(&kline_series);
    test_cdlspinningtop(&kline_series);
    test_cdlstalledpattern(&kline_series);
    test_cdlsticksandwich(&kline_series);
    test_cdltakuri(&kline_series);
    test_cdltasukigap(&kline_series);
    test_cdlthrusting(&kline_series);
    test_cdltristar(&kline_series);
    test_cdlunique3river(&kline_series);
    test_cdlupsidegap2crows(&kline_series);
    test_cdlxsidegap3methods(&kline_series);
}

/// 测试CDL2CROWS指标
fn test_cdl2crows(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdl2crows(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDL2CROWS: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDL2CROWS: 计算失败 - {:?}", e),
    }
}

/// 测试CDL3BLACKCROWS指标
fn test_cdl3blackcrows(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdl3blackcrows(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDL3BLACKCROWS: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDL3BLACKCROWS: 计算失败 - {:?}", e),
    }
}

/// 测试CDL3INSIDE指标
fn test_cdl3inside(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdl3inside(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDL3INSIDE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDL3INSIDE: 计算失败 - {:?}", e),
    }
}

/// 测试CDL3LINESTRIKE指标
fn test_cdl3linestrike(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdl3linestrike(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDL3LINESTRIKE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDL3LINESTRIKE: 计算失败 - {:?}", e),
    }
}

/// 测试CDL3OUTSIDE指标
fn test_cdl3outside(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdl3outside(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDL3OUTSIDE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDL3OUTSIDE: 计算失败 - {:?}", e),
    }
}

/// 测试CDL3STARSINSOUTH指标
fn test_cdl3starsinsouth(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdl3starsinsouth(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDL3STARSINSOUTH: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDL3STARSINSOUTH: 计算失败 - {:?}", e),
    }
}

/// 测试CDL3WHITESOLDIERS指标
fn test_cdl3whitesoldiers(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdl3whitesoldiers(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDL3WHITESOLDIERS: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDL3WHITESOLDIERS: 计算失败 - {:?}", e),
    }
}

/// 测试CDLABANDONEDBABY指标
fn test_cdlabandonedbaby(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = CDLABANDONEDBABYConfig {
        penetration: OrderedFloat(0.3),
    };

    match CalculateIndicatorFunction::calculate_cdlabandonedbaby(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  CDLABANDONEDBABY: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLABANDONEDBABY: 计算失败 - {:?}", e),
    }
}

/// 测试CDLADVANCEBLOCK指标
fn test_cdladvanceblock(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdladvanceblock(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLADVANCEBLOCK: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLADVANCEBLOCK: 计算失败 - {:?}", e),
    }
}

/// 测试CDLBELTHOLD指标
fn test_cdlbelthold(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlbelthold(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLBELTHOLD: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLBELTHOLD: 计算失败 - {:?}", e),
    }
}

/// 测试CDLBREAKAWAY指标
fn test_cdlbreakaway(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlbreakaway(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLBREAKAWAY: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLBREAKAWAY: 计算失败 - {:?}", e),
    }
}

/// 测试CDLCLOSINGMARUBOZU指标
fn test_cdlclosingmarubozu(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlclosingmarubozu(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLCLOSINGMARUBOZU: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLCLOSINGMARUBOZU: 计算失败 - {:?}", e),
    }
}

/// 测试CDLCONCEALBABYSWALL指标
fn test_cdlconcealbabyswall(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlconcealbabyswall(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLCONCEALBABYSWALL: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLCONCEALBABYSWALL: 计算失败 - {:?}", e),
    }
}

/// 测试CDLCOUNTERATTACK指标
fn test_cdlcounterattack(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlcounterattack(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLCOUNTERATTACK: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLCOUNTERATTACK: 计算失败 - {:?}", e),
    }
}

/// 测试CDLDARKCLOUDCOVER指标
fn test_cdldarkcloudcover(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = CDLDARKCLOUDCOVERConfig {
        penetration: OrderedFloat(0.5),
    };

    match CalculateIndicatorFunction::calculate_cdldarkcloudcover(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  CDLDARKCLOUDCOVER: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLDARKCLOUDCOVER: 计算失败 - {:?}", e),
    }
}

/// 测试CDLDOJI指标
fn test_cdldoji(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdldoji(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLDOJI: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLDOJI: 计算失败 - {:?}", e),
    }
}

/// 测试CDLDOJISTAR指标
fn test_cdldojistar(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdldojistar(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLDOJISTAR: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLDOJISTAR: 计算失败 - {:?}", e),
    }
}

/// 测试CDLDRAGONFLYDOJI指标
fn test_cdldragonflydoji(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdldragonflydoji(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLDRAGONFLYDOJI: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLDRAGONFLYDOJI: 计算失败 - {:?}", e),
    }
}

/// 测试CDLENGULFING指标
fn test_cdlengulfing(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlengulfing(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLENGULFING: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLENGULFING: 计算失败 - {:?}", e),
    }
}

/// 测试CDLEVENINGDOJISTAR指标
fn test_cdleveningdojistar(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = CDLEVENINGDOJISTARConfig {
        penetration: OrderedFloat(0.3),
    };

    match CalculateIndicatorFunction::calculate_cdleveningdojistar(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  CDLEVENINGDOJISTAR: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLEVENINGDOJISTAR: 计算失败 - {:?}", e),
    }
}

/// 测试CDLEVENINGSTAR指标
fn test_cdleveningstar(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = CDLEVENINGSTARConfig {
        penetration: OrderedFloat(0.3),
    };

    match CalculateIndicatorFunction::calculate_cdleveningstar(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  CDLEVENINGSTAR: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLEVENINGSTAR: 计算失败 - {:?}", e),
    }
}

/// 测试CDLGAPSIDESIDEWHITE指标
fn test_cdlgapsidesidewhite(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlgapsidesidewhite(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLGAPSIDESIDEWHITE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLGAPSIDESIDEWHITE: 计算失败 - {:?}", e),
    }
}

/// 测试CDLGRAVESTONEDOJI指标
fn test_cdlgravestonedoji(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlgravestonedoji(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLGRAVESTONEDOJI: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLGRAVESTONEDOJI: 计算失败 - {:?}", e),
    }
}

/// 测试CDLHAMMER指标
fn test_cdlhammer(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlhammer(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLHAMMER: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLHAMMER: 计算失败 - {:?}", e),
    }
}

/// 测试CDLHANGINGMAN指标
fn test_cdlhangingman(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlhangingman(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLHANGINGMAN: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLHANGINGMAN: 计算失败 - {:?}", e),
    }
}

/// 测试CDLHARAMI指标
fn test_cdlharami(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlharami(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLHARAMI: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLHARAMI: 计算失败 - {:?}", e),
    }
}

/// 测试CDLHARAMICROSS指标
fn test_cdlharamicross(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlharamicross(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLHARAMICROSS: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLHARAMICROSS: 计算失败 - {:?}", e),
    }
}

/// 测试CDLHIGHWAVE指标
fn test_cdlhighwave(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlhighwave(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLHIGHWAVE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLHIGHWAVE: 计算失败 - {:?}", e),
    }
}

/// 测试CDLHIKKAKE指标
fn test_cdlhikkake(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlhikkake(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLHIKKAKE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLHIKKAKE: 计算失败 - {:?}", e),
    }
}

/// 测试CDLHIKKAKEMOD指标
fn test_cdlhikkakemod(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlhikkakemod(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLHIKKAKEMOD: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLHIKKAKEMOD: 计算失败 - {:?}", e),
    }
}

/// 测试CDLHOMINGPIGEON指标
fn test_cdlhomingpigeon(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlhomingpigeon(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLHOMINGPIGEON: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLHOMINGPIGEON: 计算失败 - {:?}", e),
    }
}

/// 测试CDLIDENTICAL3CROWS指标
fn test_cdlidentical3crows(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlidentical3crows(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLIDENTICAL3CROWS: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLIDENTICAL3CROWS: 计算失败 - {:?}", e),
    }
}

/// 测试CDLINNECK指标
fn test_cdlinneck(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlinneck(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLINNECK: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLINNECK: 计算失败 - {:?}", e),
    }
}

/// 测试CDLINVERTEDHAMMER指标
fn test_cdlinvertedhammer(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlinvertedhammer(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLINVERTEDHAMMER: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLINVERTEDHAMMER: 计算失败 - {:?}", e),
    }
}

/// 测试CDLKICKING指标
fn test_cdlkicking(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlkicking(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLKICKING: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLKICKING: 计算失败 - {:?}", e),
    }
}

/// 测试CDLKICKINGBYLENGTH指标
fn test_cdlkickingbylength(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlkickingbylength(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLKICKINGBYLENGTH: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLKICKINGBYLENGTH: 计算失败 - {:?}", e),
    }
}

/// 测试CDLLADDERBOTTOM指标
fn test_cdlladderbottom(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlladderbottom(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLLADDERBOTTOM: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLLADDERBOTTOM: 计算失败 - {:?}", e),
    }
}

/// 测试CDLLONGLEGGEDDOJI指标
fn test_cdllongleggeddoji(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdllongleggeddoji(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLLONGLEGGEDDOJI: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLLONGLEGGEDDOJI: 计算失败 - {:?}", e),
    }
}

/// 测试CDLLONGLINE指标
fn test_cdllongline(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdllongline(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLLONGLINE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLLONGLINE: 计算失败 - {:?}", e),
    }
}

/// 测试CDLMARUBOZU指标
fn test_cdlmarubozu(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlmarubozu(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLMARUBOZU: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLMARUBOZU: 计算失败 - {:?}", e),
    }
}

/// 测试CDLMATCHINGLOW指标
fn test_cdlmatchinglow(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlmatchinglow(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLMATCHINGLOW: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLMATCHINGLOW: 计算失败 - {:?}", e),
    }
}

/// 测试CDLMATHOLD指标
fn test_cdlmathold(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = CDLMATHOLDConfig {
        penetration: OrderedFloat(0.5),
    };

    match CalculateIndicatorFunction::calculate_cdlmathold(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  CDLMATHOLD: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLMATHOLD: 计算失败 - {:?}", e),
    }
}

/// 测试CDLMORNINGDOJISTAR指标
fn test_cdlmorningdojistar(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = CDLMORNINGDOJISTARConfig {
        penetration: OrderedFloat(0.3),
    };

    match CalculateIndicatorFunction::calculate_cdlmorningdojistar(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  CDLMORNINGDOJISTAR: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLMORNINGDOJISTAR: 计算失败 - {:?}", e),
    }
}

/// 测试CDLMORNINGSTAR指标
fn test_cdlmorningstar(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = CDLMORNINGSTARConfig {
        penetration: OrderedFloat(0.3),
    };

    match CalculateIndicatorFunction::calculate_cdlmorningstar(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  CDLMORNINGSTAR: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLMORNINGSTAR: 计算失败 - {:?}", e),
    }
}

/// 测试CDLONNECK指标
fn test_cdlonneck(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlonneck(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLONNECK: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLONNECK: 计算失败 - {:?}", e),
    }
}

/// 测试CDLPIERCING指标
fn test_cdlpiercing(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlpiercing(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLPIERCING: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLPIERCING: 计算失败 - {:?}", e),
    }
}

/// 测试CDLRICKSHAWMAN指标
fn test_cdlrickshawman(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlrickshawman(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLRICKSHAWMAN: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLRICKSHAWMAN: 计算失败 - {:?}", e),
    }
}

/// 测试CDLRISEFALL3METHODS指标
fn test_cdlrisefall3methods(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlrisefall3methods(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLRISEFALL3METHODS: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLRISEFALL3METHODS: 计算失败 - {:?}", e),
    }
}

/// 测试CDLSEPARATINGLINES指标
fn test_cdlseparatinglines(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlseparatinglines(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLSEPARATINGLINES: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLSEPARATINGLINES: 计算失败 - {:?}", e),
    }
}

/// 测试CDLSHOOTINGSTAR指标
fn test_cdlshootingstar(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlshootingstar(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLSHOOTINGSTAR: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLSHOOTINGSTAR: 计算失败 - {:?}", e),
    }
}

/// 测试CDLSHORTLINE指标
fn test_cdlshortline(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlshortline(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLSHORTLINE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLSHORTLINE: 计算失败 - {:?}", e),
    }
}

/// 测试CDLSPINNINGTOP指标
fn test_cdlspinningtop(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlspinningtop(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLSPINNINGTOP: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLSPINNINGTOP: 计算失败 - {:?}", e),
    }
}

/// 测试CDLSTALLEDPATTERN指标
fn test_cdlstalledpattern(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlstalledpattern(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLSTALLEDPATTERN: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLSTALLEDPATTERN: 计算失败 - {:?}", e),
    }
}

/// 测试CDLSTICKSANDWICH指标
fn test_cdlsticksandwich(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlsticksandwich(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLSTICKSANDWICH: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLSTICKSANDWICH: 计算失败 - {:?}", e),
    }
}

/// 测试CDLTAKURI指标
fn test_cdltakuri(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdltakuri(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLTAKURI: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLTAKURI: 计算失败 - {:?}", e),
    }
}

/// 测试CDLTASUKIGAP指标
fn test_cdltasukigap(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdltasukigap(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLTASUKIGAP: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLTASUKIGAP: 计算失败 - {:?}", e),
    }
}

/// 测试CDLTHRUSTING指标
fn test_cdlthrusting(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlthrusting(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLTHRUSTING: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLTHRUSTING: 计算失败 - {:?}", e),
    }
}

/// 测试CDLTRISTAR指标
fn test_cdltristar(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdltristar(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLTRISTAR: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLTRISTAR: 计算失败 - {:?}", e),
    }
}

/// 测试CDLUNIQUE3RIVER指标
fn test_cdlunique3river(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlunique3river(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLUNIQUE3RIVER: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLUNIQUE3RIVER: 计算失败 - {:?}", e),
    }
}

/// 测试CDLUPSIDEGAP2CROWS指标
fn test_cdlupsidegap2crows(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlupsidegap2crows(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLUPSIDEGAP2CROWS: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLUPSIDEGAP2CROWS: 计算失败 - {:?}", e),
    }
}

/// 测试CDLXSIDEGAP3METHODS指标
fn test_cdlxsidegap3methods(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_cdlxsidegap3methods(kline_series.to_vec()) {
        Ok(results) => {
            println!("  CDLXSIDEGAP3METHODS: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  CDLXSIDEGAP3METHODS: 计算失败 - {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_recognition_indicators_integration() {
        test_pattern_recognition_indicators();
    }

    #[test]
    fn test_doji_individual() {
        let kline_series = generate_simple_kline_series(50, None, None);
        test_cdldoji(&kline_series);
    }

    #[test]
    fn test_engulfing_individual() {
        let kline_series = generate_simple_kline_series(50, None, None);
        test_cdlengulfing(&kline_series);
    }

    #[test]
    fn test_hammer_patterns() {
        let kline_series = generate_simple_kline_series(50, None, None);
        test_cdldragonflydoji(&kline_series);
    }
}
