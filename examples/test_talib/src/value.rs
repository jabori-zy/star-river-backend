use star_river_core::cache::CacheValue;
use star_river_core::market::Kline;
use chrono::{DateTime, FixedOffset, Duration};
use std::sync::Arc;

fn fixed_offset_time_from_millis(ms: i64) -> DateTime<FixedOffset> {
    let utc_dt = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms)
        .expect("invalid unix millis timestamp");
    utc_dt.with_timezone(&FixedOffset::east_opt(0).expect("invalid fixed offset"))
}

/// 生成指定长度的CacheValue向量，包含模拟的Kline数据
///
/// # 参数
/// * `length` - 要生成的数据点数量
/// * `start_timestamp` - 起始时间戳 (毫秒)
/// * `interval_ms` - 时间间隔 (毫秒)
/// * `base_price` - 基础价格
/// * `price_variation` - 价格变动范围
///
/// # 返回
/// * `Vec<Arc<CacheValue>>` - 包含Kline数据的CacheValue向量
pub fn generate_kline_cache_values(
    length: usize,
    start_timestamp: DateTime<FixedOffset>,
    interval_ms: i64,
    base_price: f64,
    price_variation: f64,
) -> Vec<Arc<CacheValue>> {
    let mut cache_values = Vec::with_capacity(length);

    for i in 0..length {
        let datetime = start_timestamp + Duration::milliseconds(i as i64 * interval_ms);   

        // 生成模拟的价格数据
        let price_multiplier = 1.0 + (i as f64 * price_variation / length as f64);
        let open = base_price * price_multiplier;
        let high = open * (1.0 + 0.02); // 高点比开盘价高2%
        let low = open * (1.0 - 0.01); // 低点比开盘价低1%
        let close = open + (high - low) * 0.3; // 收盘价在开盘和高点之间
        let volume = 1000.0 + (i as f64 * 100.0); // 递增的成交量

        let kline = Kline {
            datetime,
            open,
            high,
            low,
            close,
            volume,
        };

        cache_values.push(Arc::new(CacheValue::Kline(kline)));
    }

    cache_values
}

/// 生成简单的升序价格Kline数据
///
/// # 参数
/// * `length` - 要生成的数据点数量
/// * `start_timestamp` - 起始时间戳 (毫秒，默认：1622534400000)
/// * `base_price` - 基础价格 (默认：100.0)
///
/// # 返回
/// * `Vec<Arc<CacheValue>>` - 包含Kline数据的CacheValue向量
pub fn generate_simple_kline_series(
    length: usize,
    start_timestamp: Option<DateTime<FixedOffset>>,
    base_price: Option<f64>,
) -> Vec<Arc<CacheValue>> {
    let start_ts = start_timestamp.unwrap_or_else(|| fixed_offset_time_from_millis(1622534400000)); // 2021-06-01 12:00:00
    let base = base_price.unwrap_or(100.0);
    let interval = 60000; // 1分钟间隔

    generate_kline_cache_values(length, start_ts, interval, base, 0.1)
}

/// 生成包含趋势的Kline数据
///
/// # 参数
/// * `length` - 要生成的数据点数量
/// * `trend_factor` - 趋势因子 (正数为上升趋势，负数为下降趋势)
///
/// # 返回
/// * `Vec<Arc<CacheValue>>` - 包含趋势Kline数据的CacheValue向量
pub fn generate_trending_kline_series(length: usize, trend_factor: f64) -> Vec<Arc<CacheValue>> {
    let mut cache_values = Vec::with_capacity(length);
    let start_timestamp = fixed_offset_time_from_millis(1622534400000); // 2021-06-01 12:00:00
    let interval = 60000; // 1分钟间隔
    let base_price = 100.0;

    for i in 0..length {
        let datetime = start_timestamp + Duration::milliseconds(i as i64 * interval);

        // 应用趋势因子
        let trend_adjustment = i as f64 * trend_factor;
        let open = base_price + trend_adjustment;
        let high = open * 1.02;
        let low = open * 0.98;
        let close = open + (high - low) * 0.5;
        let volume = 1000.0 + (i as f64 * 50.0);

        let kline = Kline {
            datetime,
            open,
            high,
            low,
            close,
            volume,
        };

        cache_values.push(Arc::new(CacheValue::Kline(kline)));
    }

    cache_values
}

/// 生成批量测试数据，包含多个不同长度的数据集
///
/// # 返回
/// * `Vec<(String, Vec<Arc<CacheValue>>)>` - 包含标签和数据的元组向量
pub fn generate_test_datasets() -> Vec<(String, Vec<Arc<CacheValue>>)> {
    vec![
        (
            "small_10".to_string(),
            generate_simple_kline_series(10, None, None),
        ),
        (
            "medium_50".to_string(),
            generate_simple_kline_series(50, None, None),
        ),
        (
            "large_200".to_string(),
            generate_simple_kline_series(200, None, None),
        ),
        (
            "uptrend_30".to_string(),
            generate_trending_kline_series(30, 0.5),
        ),
        (
            "downtrend_30".to_string(),
            generate_trending_kline_series(30, -0.3),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_kline_cache_values() {
        let values = generate_kline_cache_values(5, fixed_offset_time_from_millis(1622534400000), 60000, 100.0, 0.1);
        assert_eq!(values.len(), 5);

        // 验证第一个值
        if let Some(first_value) = values.first() {
            if let Some(kline) = first_value.as_kline() {
                assert_eq!(kline.datetime, fixed_offset_time_from_millis(1622534400000));
                assert!(kline.open > 0.0);
                assert!(kline.high >= kline.open);
                assert!(kline.low <= kline.open);
            }
        }
    }

    #[test]
    fn test_generate_simple_kline_series() {
        let values = generate_simple_kline_series(10, None, None);
        assert_eq!(values.len(), 10);

        // 验证时间戳递增
        for i in 1..values.len() {
            let prev_ts = values[i - 1].get_timestamp();
            let curr_ts = values[i].get_timestamp();
            assert!(curr_ts > prev_ts);
        }
    }

    #[test]
    fn test_generate_test_datasets() {
        let datasets = generate_test_datasets();
        assert_eq!(datasets.len(), 5);

        // 验证每个数据集都有正确的长度
        assert_eq!(datasets[0].1.len(), 10); // small_10
        assert_eq!(datasets[1].1.len(), 50); // medium_50
        assert_eq!(datasets[2].1.len(), 200); // large_200
        assert_eq!(datasets[3].1.len(), 30); // uptrend_30
        assert_eq!(datasets[4].1.len(), 30); // downtrend_30
    }
}
