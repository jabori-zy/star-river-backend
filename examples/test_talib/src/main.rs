mod value;

use engine::indicator_engine::talib_bindings::*;
use types::indicator::IndicatorTrait;
use value::*;
use types::indicator::indicator::*;
use types::indicator::{PriceSource, MAType};
use ordered_float::OrderedFloat;
use engine::indicator_engine::calculate::*;


fn main() {
    unsafe {
        println!("=== 小数据集测试 (10个数据点) ===");
        // 真实的时间戳
        let timestamp_list = vec![1622534400, 1622534400, 1622534400, 1622534400, 1622534400, 1622534400, 1622534400, 1622534400, 1622534400, 1622534400];
        let small_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        let medium_data: Vec<f64> = (1..=40).map(|x| x as f64).collect();
        // 真实的时间戳
        let medium_timestamp_list: Vec<i64> = (1622534400..=1622534400+39).collect();

        // let result = sma_test(&small_data, 9);
        // println!("result: {:?}", result);

        // let result = macd_test(&medium_data, 12, 26, 9);
        // println!("result: {:?}", result);

        // let result = ma(&timestamp_list, &small_data, 9, 1);
        // let result = calculate_macd(&medium_timestamp_list, &medium_data, 12, 26, 9);
        // if let Ok(result) = result {
        //     println!("result: {:?}", result);
        // } else {
        //     println!("error: {:?}", result.err());
        // }

        let kline_series = generate_simple_kline_series(20, None, None);
        let macd_config = MACDConfig {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            price_source: PriceSource::Close,
        };

        let bbands_config = BBandsConfig {
            time_period: 20,
            dev_up: OrderedFloat(2.0),
            dev_down: OrderedFloat(2.0),
            ma_type: MAType::SMA,
            price_source: PriceSource::Close,
        };


        // let result = CalculateIndicatorFunction::calculate_bbands(kline_series, &bbands_config);
        // let result = CalculateIndicatorFunction::calculate_cdl2crows(kline_series);
        // match result {
        //     Ok(result) => {
        //         // let result_list = result.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
        //         println!("result: {:#?}", result);
        //     }
        //     Err(e) => {
        //         println!("error: {:?}", e);
        //     }
        // }
        let ma_config = MAConfig {
            time_period: 9,
            ma_type: MAType::SMA,
            price_source: PriceSource::Close,
        };

        println!("ma_config: {:#?}", ma_config.to_string());
    }
}

unsafe fn sma_test(data: &[f64], period: i32) -> Vec<f64> {
    let input_size = data.len();
    let lookback = TA_MA_Lookback(period, TA_MAType_TA_MAType_SMA) as usize;
    
    // 直接创建全0数组，不需要复杂的预分配逻辑
    let mut output: Vec<f64> = vec![0.0; input_size];
    let mut out_beg_idx: i32 = 0;
    let mut out_nb_element: i32 = 0;
    
    // 直接写入到正确位置
    TA_MA(
        0,
        (input_size - 1) as i32,
        data.as_ptr(),
        period,
        TA_MAType_TA_MAType_SMA,
        &mut out_beg_idx,
        &mut out_nb_element,
        output.as_mut_ptr().add(lookback),
    );
    
    output
}

unsafe fn macd_test(data: &[f64], fast_period: i32, slow_period: i32, signal_period: i32) -> Vec<Vec<f64>> {
    let input_size = data.len();
    let lookback = TA_MACD_Lookback(fast_period, slow_period, signal_period) as usize;
    println!("macd lookback: {}", lookback);

    let mut out_beg_idx: i32 = 0;
    let mut out_nb_element: i32 = 0;

    let mut out_macd: Vec<f64> = vec![0.0; input_size];
    let mut out_signal: Vec<f64> = vec![0.0; input_size];
    let mut out_hist: Vec<f64> = vec![0.0; input_size];

    let ret = TA_MACD(
        0,
        (input_size - 1) as i32,
        data.as_ptr(),
        fast_period,
        slow_period,
        signal_period,
        &mut out_beg_idx,
        &mut out_nb_element,
        out_macd.as_mut_ptr().add(lookback),
        out_signal.as_mut_ptr().add(lookback),
        out_hist.as_mut_ptr().add(lookback),
    );

    if ret != TA_RetCode_TA_SUCCESS {
        println!("macd error: {:?}", ret);
    }

    vec![out_macd, out_signal, out_hist]
}
