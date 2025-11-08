use crate::binance::Binance;
use crate::exchange_trait::ExchangeMarketDataExt;
use star_river_core::market::KlineInterval;
use strategy_core::strategy::TimeRange;
use chrono::{Utc, Duration, TimeZone};

#[tokio::test]
async fn test_get_kline_series() {
    let exchange = Binance::new();

    let symbol = "BTCUSDT";
    let interval = KlineInterval::Minutes1;
    let limit = 10;

    let result = exchange.get_kline_series(symbol, interval, limit).await;

    assert!(result.is_ok());
    let klines = result.unwrap();

    // 验证返回的K线数量
    assert!(!klines.is_empty());
    assert!(klines.len() <= limit as usize);

    // 验证K线数据的合理性
    for kline in &klines {
        assert!(kline.open > 0.0);
        assert!(kline.high > 0.0);
        assert!(kline.low > 0.0);
        assert!(kline.close > 0.0);
        assert!(kline.volume >= 0.0);
        assert!(kline.high >= kline.low);
        assert!(kline.high >= kline.open);
        assert!(kline.high >= kline.close);
        assert!(kline.low <= kline.open);
        assert!(kline.low <= kline.close);
    }

    // 验证时间顺序(应该是按时间升序排列)
    for i in 1..klines.len() {
        assert!(klines[i].datetime >= klines[i - 1].datetime);
    }
}

#[tokio::test]
async fn test_get_kline_series_different_intervals() {
    let exchange = Binance::new();

    let symbol = "ETHUSDT";
    let intervals = vec![
        KlineInterval::Minutes1,
        KlineInterval::Minutes5,
        KlineInterval::Minutes15,
        KlineInterval::Hours1,
    ];

    for interval in intervals {
        let result = exchange.get_kline_series(symbol, interval.clone(), 5).await;
        assert!(result.is_ok(), "Failed for interval: {:?}", interval);

        let klines = result.unwrap();
        assert!(!klines.is_empty(), "No klines returned for interval: {:?}", interval);
    }
}

#[tokio::test]
async fn test_get_kline_series_invalid_symbol() {
    let exchange = Binance::new();
    let symbol = "INVALIDSYMBOL123";
    let interval = KlineInterval::Minutes1;
    let limit = 10;

    let result = exchange.get_kline_series(symbol, interval, limit).await;

    // 应该返回错误或者空列表
    // 根据实际API行为,这里可能需要调整断言
    assert!(result.is_err() || result.unwrap().is_empty());
}

#[tokio::test]
async fn test_get_kline_history() {
    let exchange = Binance::new();

    let symbol = "BTCUSDT";
    let interval = KlineInterval::Hours1;

    // 获取最近7天的K线数据
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(7);

    let time_range = TimeRange {
        start_date,
        end_date,
    };

    let result = exchange.get_kline_history(symbol, interval, time_range).await;

    assert!(result.is_ok());
    let klines = result.unwrap();
    println!("历史K线数量: {}", klines.len());

    // 验证返回的K线不为空
    assert!(!klines.is_empty());

    // 验证K线数据的合理性
    for kline in &klines {
        assert!(kline.open > 0.0);
        assert!(kline.high > 0.0);
        assert!(kline.low > 0.0);
        assert!(kline.close > 0.0);
        assert!(kline.volume >= 0.0);
        assert!(kline.high >= kline.low);
    }

    // 验证时间范围
    let first_kline = &klines[0];
    let last_kline = &klines[klines.len() - 1];

    assert!(first_kline.datetime >= start_date);
    assert!(last_kline.datetime <= end_date);

    // 验证时间顺序(应该是按时间升序排列)
    for i in 1..klines.len() {
        assert!(klines[i].datetime >= klines[i - 1].datetime);
    }
}

#[tokio::test]
async fn test_get_kline_history_short_range() {
    let exchange = Binance::new();

    let symbol = "ETHUSDT";
    let interval = KlineInterval::Minutes15;

    // 获取最近24小时的K线数据
    let end_date = Utc::now();
    let start_date = end_date - Duration::hours(24);

    let time_range = TimeRange {
        start_date,
        end_date,
    };

    let result = exchange.get_kline_history(symbol, interval, time_range).await;

    assert!(result.is_ok());
    let klines = result.unwrap();

    // 验证返回的K线不为空
    assert!(!klines.is_empty());

    // 24小时内，15分钟间隔，应该大约有96根K线
    // 允许一定的误差范围
    assert!(klines.len() >= 90 && klines.len() <= 100, "期望约96根K线，实际: {}", klines.len());
}

#[tokio::test]
async fn test_get_kline_history_different_intervals() {
    let exchange = Binance::new();

    let symbol = "BTCUSDT";
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(1);

    let time_range = TimeRange {
        start_date,
        end_date,
    };

    let intervals = vec![
        KlineInterval::Minutes5,
        KlineInterval::Minutes15,
        KlineInterval::Hours1,
        KlineInterval::Hours4,
    ];

    for interval in intervals {
        let result = exchange.get_kline_history(symbol, interval.clone(), time_range.clone()).await;
        assert!(result.is_ok(), "Failed for interval: {:?}", interval);

        let klines = result.unwrap();
        assert!(!klines.is_empty(), "No klines returned for interval: {:?}", interval);
        println!("Interval {:?}: {} klines", interval, klines.len());
    }
}

#[tokio::test]
async fn test_get_kline_series_default_limit() {
    let exchange = Binance::new();

    let symbol = "BTCUSDT";
    let interval = KlineInterval::Minutes1;

    // 不指定limit，应该返回默认数量（500）
    let limit = 500;
    let result = exchange.get_kline_series(symbol, interval, limit).await;

    assert!(result.is_ok());
    let klines = result.unwrap();

    println!("默认limit(500)返回K线数量: {}", klines.len());
    assert_eq!(klines.len(), 500, "默认limit应该返回500根K线");
}

#[tokio::test]
async fn test_get_kline_series_max_limit() {
    let exchange = Binance::new();

    let symbol = "BTCUSDT";
    let interval = KlineInterval::Minutes1;

    // 请求最大数量的K线（1000）
    let limit = 1000;
    let result = exchange.get_kline_series(symbol, interval, limit).await;

    assert!(result.is_ok());
    let klines = result.unwrap();

    println!("最大limit(1000)返回K线数量: {}", klines.len());
    assert_eq!(klines.len(), 1000, "最大limit应该返回1000根K线");
}

#[tokio::test]
async fn test_get_kline_series_exceed_max_limit() {
    let exchange = Binance::new();

    let symbol = "BTCUSDT";
    let interval = KlineInterval::Minutes1;

    // 尝试请求超过最大限制的K线数量
    let limit = 1500;
    let result = exchange.get_kline_series(symbol, interval, limit).await;

    // 根据API行为，可能会返回错误或者只返回1000根K线
    if result.is_ok() {
        let klines = result.unwrap();
        println!("超过最大limit(1500)实际返回K线数量: {}", klines.len());
        // API应该最多返回1000根K线
        assert!(klines.len() <= 1000, "即使请求超过1000，也不应该返回超过1000根K线");
    } else {
        println!("请求超过最大limit返回错误: {:?}", result.err());
    }
}

#[tokio::test]
async fn test_get_kline_series_small_limit() {
    let exchange = Binance::new();

    let symbol = "BTCUSDT";
    let interval = KlineInterval::Minutes1;

    // 请求少量K线
    let limit = 10;
    let result = exchange.get_kline_series(symbol, interval, limit).await;

    assert!(result.is_ok());
    let klines = result.unwrap();

    println!("小limit(10)返回K线数量: {}", klines.len());
    assert_eq!(klines.len(), 10, "应该准确返回请求的10根K线");
}

#[tokio::test]
async fn test_get_kline_history_large_range() {
    let exchange = Binance::new();

    let symbol = "BTCUSDT";
    let interval = KlineInterval::Days1;

    // 获取最近2年的日K线数据（约730根）
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(730);

    let time_range = TimeRange {
        start_date,
        end_date,
    };

    let result = exchange.get_kline_history(symbol, interval, time_range).await;

    assert!(result.is_ok());
    let klines = result.unwrap();

    println!("2年日K线数量: {}", klines.len());
    // 2年大约730天，但可能受限于API的1000条限制
    assert!(!klines.is_empty());
    assert!(klines.len() <= 1000, "历史K线也不应该超过1000根");
}

#[tokio::test]
async fn test_get_kline_history_2010() {
    let exchange = Binance::new();

    let symbol = "BTCUSDT";
    let interval = KlineInterval::Days1;

    // 请求2010年的K线数据
    // Binance成立于2017年，BTCUSDT交易对应该在2017年之后才有数据
    let start_date = Utc.with_ymd_and_hms(2010, 1, 1, 0, 0, 0).unwrap();
    let end_date = Utc.with_ymd_and_hms(2010, 12, 31, 23, 59, 59).unwrap();

    let time_range = TimeRange {
        start_date,
        end_date,
    };

    let result = exchange.get_kline_history(symbol, interval, time_range).await;

    // 2010年Binance还不存在，应该返回空数据或者错误
    if result.is_ok() {
        let klines = result.unwrap();
        println!("2010年K线数量: {}", klines.len());
        assert!(klines.is_empty(), "2010年不应该有BTCUSDT交易数据");
    } else {
        let report = snafu::Report::from_error(result.err().unwrap());
        println!("2010年K线请求返回错误: {:#?}", report);
        // 返回错误也是合理的
    }
}

#[tokio::test]
async fn test_get_kline_history_before_exchange_existed() {
    let exchange = Binance::new();

    let symbol = "BTCUSDT";
    let interval = KlineInterval::Hours1;

    // 请求2015年的K线数据（Binance成立前）
    let start_date = Utc.with_ymd_and_hms(2015, 1, 1, 0, 0, 0).unwrap();
    let end_date = Utc.with_ymd_and_hms(2015, 1, 31, 23, 59, 59).unwrap();

    let time_range = TimeRange {
        start_date,
        end_date,
    };

    let result = exchange.get_kline_history(symbol, interval, time_range).await;

    if result.is_ok() {
        let klines = result.unwrap();
        println!("2015年K线数量: {}", klines.len());
        // 应该没有数据或者很少
        assert!(klines.is_empty() || klines.len() < 10, "2015年不应该有大量BTCUSDT数据");
    } else {
        println!("2015年K线请求返回错误: {:?}", result.err());
    }
}

#[tokio::test]
async fn test_get_kline_history_early_binance() {
    let exchange = Binance::new();

    let symbol = "SOLUSDT";
    let interval = KlineInterval::Days1;

    // 请求2017-2018年的K线数据（Binance成立初期）
    let start_date = Utc.with_ymd_and_hms(2017, 1, 1, 0, 0, 0).unwrap();
    let end_date = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();

    let time_range = TimeRange {
        start_date,
        end_date,
    };

    let result = exchange.get_kline_history(symbol, interval, time_range).await;

    assert!(result.is_ok());
    let klines = result.unwrap();
    println!("2017-2018年K线数量: {}", klines.len());

    if !klines.is_empty() {
        // 如果有数据，验证数据的有效性
        // for kline in &klines {
        //     assert!(kline.open > 0.0);
        //     assert!(kline.high > 0.0);
        //     assert!(kline.low > 0.0);
        //     assert!(kline.close > 0.0);
        //     assert!(kline.datetime >= start_date);
        //     assert!(kline.datetime <= end_date);
        // }
        println!("最早K线时间: {}", klines[0].datetime);
    } else {
        println!("2017-2018年期间没有BTCUSDT数据");
    }
}

#[tokio::test]
async fn test_get_exchange_info() {
    let exchange = Binance::new();

    let result = exchange.http_client.get_exchange_info().await;

    assert!(result.is_ok());
    let info = result.unwrap();

    // 验证返回的数据不为空
    assert!(!info.is_null());

    // 尝试解析JSON
    let json: serde_json::Value = info;

    // 验证包含必要的字段
    assert!(json.get("timezone").is_some());
    assert!(json.get("serverTime").is_some());
    assert!(json.get("symbols").is_some());

    // 验证symbols是数组
    let symbols = json["symbols"].as_array().expect("symbols应该是数组");
    assert!(!symbols.is_empty(), "symbols数组不应该为空");

    // 验证第一个symbol包含必要字段
    let first_symbol = &symbols[0];
    assert!(first_symbol.get("symbol").is_some());
    assert!(first_symbol.get("status").is_some());
    assert!(first_symbol.get("baseAsset").is_some());
    assert!(first_symbol.get("quoteAsset").is_some());

    println!("交易所信息获取成功，包含 {} 个交易对", symbols.len());
    // 打印第一个symbol
    println!("第一个symbol: {:#?}", first_symbol);
}

#[tokio::test]
async fn test_get_symbol_list() {
    use crate::exchange_trait::ExchangeSymbolExt;

    let exchange = Binance::new();

    let result = exchange.get_symbol_list().await;

    assert!(result.is_ok());
    let symbols = result.unwrap();

    // 验证返回的数据不为空
    assert!(!symbols.is_empty(), "交易对列表不应该为空");

    // 验证第一个Symbol的字段
    let first_symbol = &symbols[0];
    assert!(!first_symbol.name.is_empty(), "symbol名称不应该为空");
    assert!(first_symbol.base.is_some(), "base应该存在");
    assert!(first_symbol.quote.is_some(), "quote应该存在");

    println!("获取到 {} 个交易对", symbols.len());
    println!("第一个交易对: {:?}", first_symbol);
}
