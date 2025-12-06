#![allow(unused)]
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};

fn main() {
    println!("=== Timestamp and Timezone Comparison Test ===\n");

    // test_timestamp_basics();
    // test_timezone_conversion();
    // test_storage_formats();
    // test_parsing_formatting();
    // test_utc8_to_utc_conversion();
    demonstrate_chrono_tz_benefits();
    simple_timezone_demo();
    // test_current_utc8_time();
}

/// Test timestamp basics
fn test_timestamp_basics() {
    println!("1. Timestamp Basics Test:");

    // Get current timestamp
    let now = Utc::now();
    let timestamp = now.timestamp();
    let timestamp_ms = now.timestamp_millis();

    println!("  Current UTC time: {}", now);
    println!("  Timestamp (seconds): {}", timestamp);
    println!("  Timestamp (milliseconds): {}", timestamp_ms);
    println!();
}

/// Test timezone conversion
fn test_timezone_conversion() {
    println!("2. Timezone Conversion Test:");

    let timestamp = 1757428971; // Fixed timestamp

    // UTC time
    let utc_time = Utc.timestamp_opt(timestamp, 0).unwrap();

    // UTC+8 time
    let offset_8 = FixedOffset::east_opt(8 * 3600).unwrap();
    let beijing_time = utc_time.with_timezone(&offset_8);

    // UTC-5 time
    let offset_minus_5 = FixedOffset::west_opt(5 * 3600).unwrap();
    let ny_time = utc_time.with_timezone(&offset_minus_5);

    println!("  Same timestamp: {}", timestamp);
    println!("  UTC time:   {}", utc_time);
    println!("  Beijing time:   {}", beijing_time);
    println!("  New York time:   {}", ny_time);
    println!(
        "  Timestamps equal: {} == {} == {}",
        utc_time.timestamp(),
        beijing_time.timestamp(),
        ny_time.timestamp()
    );
    println!();
}

/// Test different storage formats
fn test_storage_formats() {
    println!("3. Storage Format Comparison:");

    let now = Utc::now();
    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let beijing_time = now.with_timezone(&beijing_offset);

    // Format 1: Unix timestamp (used in current project)
    let timestamp_i64 = now.timestamp_millis();

    // Format 2: RFC 3339 string
    let rfc3339_utc = now.to_rfc3339();
    let rfc3339_beijing = beijing_time.to_rfc3339();

    // Format 3: Custom format
    let custom_format = beijing_time.format("%Y-%m-%d %H:%M:%S %z").to_string();

    println!("  Format 1 - Timestamp (ms): {}", timestamp_i64);
    println!("  Format 2 - RFC3339 UTC: {}", rfc3339_utc);
    println!("  Format 2 - RFC3339 +8:  {}", rfc3339_beijing);
    println!("  Format 3 - Custom format: {}", custom_format);
    println!();
}

/// Test parsing and formatting
fn test_parsing_formatting() {
    println!("4. Parsing and Formatting Test:");

    // Parse from timestamp
    let timestamp = 1705334400000i64; // Millisecond timestamp
    let from_timestamp = Utc.timestamp_millis_opt(timestamp).unwrap();

    // Parse from RFC3339 string
    let rfc3339_str = "2024-01-15T22:00:00+08:00";
    let from_rfc3339 = DateTime::parse_from_rfc3339(rfc3339_str).unwrap();

    // Parse from custom format
    let custom_str = "2024-01-15 14:00:00";
    let naive_dt = NaiveDateTime::parse_from_str(custom_str, "%Y-%m-%d %H:%M:%S").unwrap();
    let from_custom = Utc.from_utc_datetime(&naive_dt);

    println!("  Parse from timestamp: {} -> {}", timestamp, from_timestamp);
    println!("  Parse from RFC3339: {} -> {}", rfc3339_str, from_rfc3339);
    println!("  Parse from custom format: {} -> {}", custom_str, from_custom);

    // Verify timestamps are equal
    println!("  Timestamp comparison:");
    println!("    from_timestamp: {}", from_timestamp.timestamp());
    println!("    from_rfc3339:   {}", from_rfc3339.timestamp());
    println!("    from_custom:    {}", from_custom.timestamp());
    println!();
}

/// Test UTC+8 to UTC time conversion
fn test_utc8_to_utc_conversion() {
    println!("5. UTC+8 to UTC Time Conversion Test:");

    // Method 1: Convert from RFC3339 string
    let utc8_str = "2025-09-09T22:47:17.245928900+08:00";
    let utc8_time = DateTime::parse_from_rfc3339(utc8_str).unwrap();
    let utc_time1 = utc8_time.with_timezone(&Utc);

    println!("  Method 1 - From RFC3339 string:");
    println!("    UTC+8: {}", utc8_str);
    println!("    UTC:   {}", utc_time1);

    // Method 2: Create from FixedOffset then convert
    let beijing_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let utc8_time2: DateTime<FixedOffset> = beijing_offset.with_ymd_and_hms(2025, 9, 9, 22, 47, 17).unwrap();
    let utc_time2 = utc8_time2.with_timezone(&Utc);

    println!("  Method 2 - Create from FixedOffset:");
    println!("    UTC+8: {}", utc8_time2);
    println!("    UTC:   {}", utc_time2);

    // Method 3: Convert current UTC+8 time to UTC
    let current_utc8 = Utc::now().with_timezone(&beijing_offset);
    let current_utc = current_utc8.with_timezone(&Utc);

    println!("  Method 3 - Current time conversion:");
    println!("    Current UTC+8: {}", current_utc8.format("%Y-%m-%d %H:%M:%S%.3f %z"));
    println!("    Current UTC:   {}", current_utc.format("%Y-%m-%d %H:%M:%S%.3f %z"));

    // Verify timestamps are equal
    println!("  Timestamp verification:");
    println!("    UTC+8 timestamp: {}", utc8_time.timestamp());
    println!("    UTC timestamp:   {}", utc_time1.timestamp());
    println!("    Timestamps equal:   {}", utc8_time.timestamp() == utc_time1.timestamp());

    println!();
}

/// Demonstrate actual functionality of chrono-tz
fn demonstrate_chrono_tz_benefits() {
    // chrono-tz actual functionality demonstration
    use chrono_tz::{
        Asia::Shanghai,
        Australia::Sydney,
        Europe::London,
        Tz,
        US::{Eastern, Pacific},
    };

    let now = Utc::now();

    println!("=== chrono-tz Actual Functionality Demonstration ===");

    // 1. Automatic daylight saving time handling
    println!("1. Automatic Daylight Saving Time Handling:");

    // US Eastern Time: Winter EST (UTC-5), Summer EDT (UTC-4)
    let winter_date = Eastern.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
    let summer_date = Eastern.with_ymd_and_hms(2025, 7, 15, 12, 0, 0).unwrap();

    println!("   US Eastern Winter: {} (offset: {})", winter_date, winter_date.offset());
    println!("   US Eastern Summer: {} (offset: {})", summer_date, summer_date.offset());

    // London Time: Winter GMT (UTC+0), Summer BST (UTC+1)
    let london_winter = London.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap();
    let london_summer = London.with_ymd_and_hms(2025, 7, 15, 12, 0, 0).unwrap();

    println!("   London Winter: {} (offset: {})", london_winter, london_winter.offset());
    println!("   London Summer: {} (offset: {})", london_summer, london_summer.offset());

    // 2. Same UTC moment across the globe
    println!("2. Global Time at Same UTC Moment:");
    let utc_moment = Utc::now();

    let timezones = vec![
        ("Shanghai", Shanghai as Tz),
        ("New York", Eastern),
        ("London", London),
        ("Los Angeles", Pacific),
        ("Sydney", Sydney),
    ];

    println!("   UTC: {}", utc_moment);
    for (name, tz) in timezones {
        let local_time = utc_moment.with_timezone(&tz);
        println!("   {}: {} ({})", name, local_time, tz);
    }

    // 3. Cross-timezone business scenario
    println!("3. Business Scenario - Beijing Market Opens at 9 AM:");
    let beijing_open = Shanghai.with_ymd_and_hms(2025, 7, 15, 9, 0, 0).unwrap();
    let ny_time = beijing_open.with_timezone(&Eastern);
    let london_time = beijing_open.with_timezone(&London);

    println!("   Beijing: {} {}", beijing_open.format("%H:%M"), Shanghai);
    println!("   New York: {} {}", ny_time.format("%H:%M"), Eastern);
    println!("   London: {} {}", london_time.format("%H:%M"), London);

    // 4. Timezone identifier parsing
    println!("4. Timezone String Parsing:");
    let tz_names = vec!["Asia/Shanghai", "America/New_York", "Europe/London"];

    for name in tz_names {
        if let Ok(tz) = name.parse::<Tz>() {
            let local_time = now.with_timezone(&tz);
            println!("   {} -> {}", name, local_time.format("%H:%M %Z"));
        }
    }

    println!();
}

/// Simple timezone demonstration
fn simple_timezone_demo() {
    use chrono_tz::{TZ_VARIANTS, Tz};

    println!("=== All Timezones Supported by chrono-tz ===");
    println!("Total: {} timezones\n", TZ_VARIANTS.len());

    // Display current time in commonly used timezones
    println!("Current Time in Common Timezones:");
    let common_timezones = vec![
        ("Beijing", "Asia/Shanghai"),
        ("Tokyo", "Asia/Tokyo"),
        ("New York", "America/New_York"),
        ("Los Angeles", "America/Los_Angeles"),
        ("London", "Europe/London"),
        ("Sydney", "Australia/Sydney"),
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

    println!("\nFirst 20 Timezone Names:");
    for (i, tz) in TZ_VARIANTS.iter().take(20).enumerate() {
        println!("   {:<2}. {}", i + 1, tz.name());
    }

    println!("\nUsage:");
    println!("   use chrono_tz::{{TZ_VARIANTS, Tz}};");
    println!("   let tz: Tz = \"Asia/Shanghai\".parse().unwrap();");
    println!("   let now = Utc::now().with_timezone(&tz);");

    println!();
}

/// Test methods to get current UTC+8 time
fn test_current_utc8_time() {
    println!("7. Methods to Get Current UTC+8 Time:");

    // Method 1: Get directly using FixedOffset
    let utc8_offset = FixedOffset::east_opt(8 * 3600).unwrap();
    let current_utc8_1 = Utc::now().with_timezone(&utc8_offset);

    // Method 2: Get UTC first then convert
    let utc_now = Utc::now();
    let current_utc8_2 = utc_now.with_timezone(&utc8_offset);

    // Method 3: Use timestamp then convert
    let timestamp = Utc::now().timestamp_millis();
    let from_timestamp = Utc.timestamp_millis_opt(timestamp).unwrap();
    let current_utc8_3 = from_timestamp.with_timezone(&utc8_offset);

    println!("  Method 1 - Direct conversion: {}", current_utc8_1.format("%Y-%m-%d %H:%M:%S%.3f %z"));
    println!("  Method 2 - UTC conversion:  {}", current_utc8_2.format("%Y-%m-%d %H:%M:%S%.3f %z"));
    println!("  Method 3 - Timestamp conversion: {}", current_utc8_3.format("%Y-%m-%d %H:%M:%S%.3f %z"));

    // Different format displays
    println!("  Formatted Display:");
    println!("    Standard format: {}", current_utc8_1);
    println!("    RFC3339:  {}", current_utc8_1.to_rfc3339());
    println!("    Custom 1:  {}", current_utc8_1.format("%Y-%m-%d %H:%M:%S"));
    println!("    Custom 2:  {}", current_utc8_1.format("%Y-%m-%d %H:%M:%S"));
    println!("    Time only: {}", current_utc8_1.format("%H:%M:%S"));
    println!("    Date only: {}", current_utc8_1.format("%Y-%m-%d"));

    // Timestamp comparison
    println!("  Timestamp Comparison (milliseconds):");
    println!("    Method 1: {}", current_utc8_1.timestamp_millis());
    println!("    Method 2: {}", current_utc8_2.timestamp_millis());
    println!("    Method 3: {}", current_utc8_3.timestamp_millis());
    println!("    UTC:   {}", utc_now.timestamp_millis());

    println!();
}
