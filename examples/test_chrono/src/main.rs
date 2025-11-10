use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};

fn main() {
    println!("=== 时间戳与时区对比测试 ===\n");

    // test_timestamp_basics();
    // test_timezone_conversion();
    // test_storage_formats();
    // test_parsing_formatting();
    // test_utc8_to_utc_conversion();
    demonstrate_chrono_tz_benefits();
    simple_timezone_demo();
    // test_current_utc8_time();
}

/// 测试时间戳基础概念
fn test_timestamp_basics() {
    println!("1. 时间戳基础测试:");

    // 获取当前时间戳
    let now = Utc::now();
    let timestamp = now.timestamp();
    let timestamp_ms = now.timestamp_millis();

    println!("  当前 UTC 时间: {}", now);
    println!("  时间戳(秒): {}", timestamp);
    println!("  时间戳(毫秒): {}", timestamp_ms);
    println!();
}

/// 测试时区转换
fn test_timezone_conversion() {
    println!("2. 时区转换测试:");

    let timestamp = 1757428971; // 固定时间戳

    // UTC 时间
    let utc_time = Utc.timestamp_opt(timestamp, 0).unwrap();

    // UTC+8 时间
    let offset_8 = FixedOffset::east_opt(8 * 3600).unwrap();
    let beijing_time = utc_time.with_timezone(&offset_8);

    // UTC-5 时间
    let offset_minus_5 = FixedOffset::west_opt(5 * 3600).unwrap();
    let ny_time = utc_time.with_timezone(&offset_minus_5);

    println!("  相同时间戳: {}", timestamp);
    println!("  UTC 时间:   {}", utc_time);
    println!("  北京时间:   {}", beijing_time);
    println!("  纽约时间:   {}", ny_time);
    println!(
        "  时间戳相等: {} == {} == {}",
        utc_time.timestamp(),
        beijing_time.timestamp(),
        ny_time.timestamp()
    );
    println!();
}

/// 测试不同存储格式
fn test_storage_formats() {
    println!("3. 存储格式对比:");

    let now = Utc::now();
    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let beijing_time = now.with_timezone(&beijing_offset);

    // 格式1: Unix时间戳 (当前项目使用)
    let timestamp_i64 = now.timestamp_millis();

    // 格式2: RFC 3339 字符串
    let rfc3339_utc = now.to_rfc3339();
    let rfc3339_beijing = beijing_time.to_rfc3339();

    // 格式3: 自定义格式
    let custom_format = beijing_time.format("%Y-%m-%d %H:%M:%S %z").to_string();

    println!("  格式1 - 时间戳(ms): {}", timestamp_i64);
    println!("  格式2 - RFC3339 UTC: {}", rfc3339_utc);
    println!("  格式2 - RFC3339 +8:  {}", rfc3339_beijing);
    println!("  格式3 - 自定义格式: {}", custom_format);
    println!();
}

/// 测试解析和格式化
fn test_parsing_formatting() {
    println!("4. 解析和格式化测试:");

    // 从时间戳解析
    let timestamp = 1705334400000i64; // 毫秒时间戳
    let from_timestamp = Utc.timestamp_millis_opt(timestamp).unwrap();

    // 从RFC3339字符串解析
    let rfc3339_str = "2024-01-15T22:00:00+08:00";
    let from_rfc3339 = DateTime::parse_from_rfc3339(rfc3339_str).unwrap();

    // 从自定义格式解析
    let custom_str = "2024-01-15 14:00:00";
    let naive_dt = NaiveDateTime::parse_from_str(custom_str, "%Y-%m-%d %H:%M:%S").unwrap();
    let from_custom = Utc.from_utc_datetime(&naive_dt);

    println!("  从时间戳解析: {} -> {}", timestamp, from_timestamp);
    println!("  从RFC3339解析: {} -> {}", rfc3339_str, from_rfc3339);
    println!("  从自定义解析: {} -> {}", custom_str, from_custom);

    // 验证时间戳是否相同
    println!("  时间戳对比:");
    println!("    from_timestamp: {}", from_timestamp.timestamp());
    println!("    from_rfc3339:   {}", from_rfc3339.timestamp());
    println!("    from_custom:    {}", from_custom.timestamp());
    println!();
}

/// 测试UTC+8时间转换为UTC时间
fn test_utc8_to_utc_conversion() {
    println!("5. UTC+8 转 UTC 时间测试:");

    // 方法1: 从 RFC3339 字符串转换
    let utc8_str = "2025-09-09T22:47:17.245928900+08:00";
    let utc8_time = DateTime::parse_from_rfc3339(utc8_str).unwrap();
    let utc_time1 = utc8_time.with_timezone(&Utc);

    println!("  方法1 - 从RFC3339字符串:");
    println!("    UTC+8: {}", utc8_str);
    println!("    UTC:   {}", utc_time1);

    // 方法2: 从FixedOffset创建再转换
    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let utc8_time2: DateTime<FixedOffset> = beijing_offset.with_ymd_and_hms(2025, 9, 9, 22, 47, 17).unwrap();
    let utc_time2 = utc8_time2.with_timezone(&Utc);

    println!("  方法2 - 从FixedOffset创建:");
    println!("    UTC+8: {}", utc8_time2);
    println!("    UTC:   {}", utc_time2);

    // 方法3: 当前UTC+8时间转UTC
    let current_utc8 = Utc::now().with_timezone(&beijing_offset);
    let current_utc = current_utc8.with_timezone(&Utc);

    println!("  方法3 - 当前时间转换:");
    println!("    当前UTC+8: {}", current_utc8.format("%Y-%m-%d %H:%M:%S%.3f %z"));
    println!("    当前UTC:   {}", current_utc.format("%Y-%m-%d %H:%M:%S%.3f %z"));

    // 验证时间戳相同
    println!("  时间戳验证:");
    println!("    UTC+8时间戳: {}", utc8_time.timestamp());
    println!("    UTC时间戳:   {}", utc_time1.timestamp());
    println!("    时间戳相等:   {}", utc8_time.timestamp() == utc_time1.timestamp());

    println!();
}

/// 演示 chrono-tz 的实际功能
fn demonstrate_chrono_tz_benefits() {
    // chrono-tz 实际功能演示
    use chrono_tz::{
        Asia::Shanghai,
        Australia::Sydney,
        Europe::London,
        Tz,
        US::{Eastern, Pacific},
    };

    let now = Utc::now();

    println!("=== chrono-tz 实际功能演示 ===");

    // 1. 夏令时自动处理演示
    println!("1. 夏令时自动处理:");

    // 美东时间：冬季 EST (UTC-5)，夏季 EDT (UTC-4)
    let winter_date = Eastern.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
    let summer_date = Eastern.with_ymd_and_hms(2025, 7, 15, 12, 0, 0).unwrap();

    println!("   美东冬季: {} (偏移: {})", winter_date, winter_date.offset());
    println!("   美东夏季: {} (偏移: {})", summer_date, summer_date.offset());

    // 伦敦时间：冬季 GMT (UTC+0)，夏季 BST (UTC+1)
    let london_winter = London.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
    let london_summer = London.with_ymd_and_hms(2025, 7, 15, 12, 0, 0).unwrap();

    println!("   伦敦冬季: {} (偏移: {})", london_winter, london_winter.offset());
    println!("   伦敦夏季: {} (偏移: {})", london_summer, london_summer.offset());

    // 2. 全球同一时刻对比
    println!("2. 同一UTC时刻的全球时间:");
    let utc_moment = Utc::now();

    let timezones = vec![
        ("上海", Shanghai as Tz),
        ("纽约", Eastern),
        ("伦敦", London),
        ("洛杉矶", Pacific),
        ("悉尼", Sydney),
    ];

    println!("   UTC: {}", utc_moment);
    for (name, tz) in timezones {
        let local_time = utc_moment.with_timezone(&tz);
        println!("   {}: {} ({})", name, local_time, tz);
    }

    // 3. 跨时区业务场景
    println!("3. 业务场景 - 北京上午9点开盘:");
    let beijing_open = Shanghai.with_ymd_and_hms(2025, 7, 15, 9, 0, 0).unwrap();
    let ny_time = beijing_open.with_timezone(&Eastern);
    let london_time = beijing_open.with_timezone(&London);

    println!("   北京: {} {}", beijing_open.format("%H:%M"), Shanghai);
    println!("   纽约: {} {}", ny_time.format("%H:%M"), Eastern);
    println!("   伦敦: {} {}", london_time.format("%H:%M"), London);

    // 4. 时区标识符解析
    println!("4. 时区字符串解析:");
    let tz_names = vec!["Asia/Shanghai", "America/New_York", "Europe/London"];

    for name in tz_names {
        if let Ok(tz) = name.parse::<Tz>() {
            let local_time = now.with_timezone(&tz);
            println!("   {} -> {}", name, local_time.format("%H:%M %Z"));
        }
    }

    println!();
}

/// 简单的时区演示
fn simple_timezone_demo() {
    use chrono_tz::{TZ_VARIANTS, Tz};

    println!("=== chrono-tz 支持的所有时区 ===");
    println!("总计: {} 个时区\n", TZ_VARIANTS.len());

    // 显示常用时区及其当前时间
    println!("常用时区当前时间:");
    let common_timezones = vec![
        ("北京", "Asia/Shanghai"),
        ("东京", "Asia/Tokyo"),
        ("纽约", "America/New_York"),
        ("洛杉矶", "America/Los_Angeles"),
        ("伦敦", "Europe/London"),
        ("悉尼", "Australia/Sydney"),
    ];

    let now = chrono::Utc::now();
    for (city, tz_name) in common_timezones {
        if let Ok(tz) = tz_name.parse::<Tz>() {
            let local_time = now.with_timezone(&tz);
            let fixed_offset_time = local_time.fixed_offset();
            let tz_local_time = local_time.with_timezone(&tz);
            println!("local_time: {}", local_time);
            println!("fixed_offset_time: {}", fixed_offset_time);
            println!("tz_local_time: {}", tz_local_time);
            println!(
                "   {:<6}: {} {} ({})",
                city,
                local_time.format("%m-%d %H:%M"),
                tz_local_time.timezone().name(),
                fixed_offset_time.offset().to_string()
            );
        }
    }

    println!("\n前20个时区名称:");
    for (i, tz) in TZ_VARIANTS.iter().take(20).enumerate() {
        println!("   {:<2}. {}", i + 1, tz.name());
    }

    println!("\n使用方法:");
    println!("   use chrono_tz::{{TZ_VARIANTS, Tz}};");
    println!("   let tz: Tz = \"Asia/Shanghai\".parse().unwrap();");
    println!("   let now = Utc::now().with_timezone(&tz);");

    println!();
}

/// 测试获取当前UTC+8时间的方法
fn test_current_utc8_time() {
    println!("7. 获取当前UTC+8时间的方法:");

    // 方法1: 使用FixedOffset直接获取
    let utc8_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let current_utc8_1 = Utc::now().with_timezone(&utc8_offset);

    // 方法2: 先获取UTC再转换
    let utc_now = Utc::now();
    let current_utc8_2 = utc_now.with_timezone(&utc8_offset);

    // 方法3: 使用timestamp再转换
    let timestamp = Utc::now().timestamp_millis();
    let from_timestamp = Utc.timestamp_millis_opt(timestamp).unwrap();
    let current_utc8_3 = from_timestamp.with_timezone(&utc8_offset);

    println!("  方法1 - 直接转换: {}", current_utc8_1.format("%Y-%m-%d %H:%M:%S%.3f %z"));
    println!("  方法2 - UTC转换:  {}", current_utc8_2.format("%Y-%m-%d %H:%M:%S%.3f %z"));
    println!("  方法3 - 时间戳转换: {}", current_utc8_3.format("%Y-%m-%d %H:%M:%S%.3f %z"));

    // 不同格式展示
    println!("  格式化展示:");
    println!("    标准格式: {}", current_utc8_1);
    println!("    RFC3339:  {}", current_utc8_1.to_rfc3339());
    println!("    自定义1:  {}", current_utc8_1.format("%Y年%m月%d日 %H时%M分%S秒"));
    println!("    自定义2:  {}", current_utc8_1.format("%Y-%m-%d %H:%M:%S"));
    println!("    只有时间: {}", current_utc8_1.format("%H:%M:%S"));
    println!("    只有日期: {}", current_utc8_1.format("%Y-%m-%d"));

    // 时间戳对比
    println!("  时间戳对比(毫秒):");
    println!("    方法1: {}", current_utc8_1.timestamp_millis());
    println!("    方法2: {}", current_utc8_2.timestamp_millis());
    println!("    方法3: {}", current_utc8_3.timestamp_millis());
    println!("    UTC:   {}", utc_now.timestamp_millis());

    println!();
}
