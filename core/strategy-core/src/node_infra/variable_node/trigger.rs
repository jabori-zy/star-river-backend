use serde::{Deserialize, Serialize};

// ==================== Timer Module ====================
pub mod timer {
    use serde::{Deserialize, Serialize};

    /// Time unit
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum TimeUnit {
        Second,
        Minute,
        Hour,
        Day,
    }

    /// Repeat mode
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum RepeatMode {
        Hourly,
        Daily,
        Weekly,
        Monthly,
    }

    /// Interval timer configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct IntervalTimerConfig {
        // mode handled by outer enum's #[serde(tag = "mode")]
        pub interval: i32,
        pub unit: TimeUnit,
    }

    /// Day of month (number, "first", or "last")
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum DayOfMonth {
        Number(u32),
        #[serde(rename = "first")]
        First,
        #[serde(rename = "last")]
        Last,
    }

    /// Monthly fallback strategy
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum MonthlyFallbackStrategy {
        LastDay,
        Skip,
    }

    /// Hourly schedule configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HourlyScheduledConfig {
        pub cron_expression: String,
        pub hourly_interval: i32,
        pub minute_of_hour: i32,
    }

    /// Daily schedule configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DailyScheduledConfig {
        pub cron_expression: String,
        pub time: String,
        pub days_of_week: Vec<i32>,
    }

    /// Weekly schedule configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WeeklyScheduledConfig {
        pub cron_expression: String,
        pub time: String,
        pub day_of_week: i32,
    }

    /// Monthly schedule configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MonthlyScheduledConfig {
        pub cron_expression: String,
        pub time: String,
        pub day_of_month: DayOfMonth,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub monthly_fallback: Option<MonthlyFallbackStrategy>,
    }

    /// Scheduled timer configuration (distinguished by repeatMode)
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "repeatMode", rename_all = "lowercase")]
    pub enum ScheduledTimerConfig {
        Hourly(HourlyScheduledConfig),
        Daily(DailyScheduledConfig),
        Weekly(WeeklyScheduledConfig),
        Monthly(MonthlyScheduledConfig),
    }

    impl ScheduledTimerConfig {
        /// Check if hourly schedule
        pub fn is_hourly(&self) -> bool {
            matches!(self, ScheduledTimerConfig::Hourly(_))
        }

        /// Check if daily schedule
        pub fn is_daily(&self) -> bool {
            matches!(self, ScheduledTimerConfig::Daily(_))
        }

        /// Check if weekly schedule
        pub fn is_weekly(&self) -> bool {
            matches!(self, ScheduledTimerConfig::Weekly(_))
        }

        /// Check if monthly schedule
        pub fn is_monthly(&self) -> bool {
            matches!(self, ScheduledTimerConfig::Monthly(_))
        }

        /// Get cron expression
        pub fn cron_expression(&self) -> &str {
            match self {
                ScheduledTimerConfig::Hourly(config) => &config.cron_expression,
                ScheduledTimerConfig::Daily(config) => &config.cron_expression,
                ScheduledTimerConfig::Weekly(config) => &config.cron_expression,
                ScheduledTimerConfig::Monthly(config) => &config.cron_expression,
            }
        }
    }

    /// Timer trigger configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "mode", rename_all = "lowercase")]
    pub enum TimerTrigger {
        Interval(IntervalTimerConfig),
        Scheduled(ScheduledTimerConfig),
    }
}

// ==================== DataFlow Module ====================
pub mod dataflow {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use star_river_core::kline::Kline;
    use ta_lib::Indicator;

    use super::timer::TimeUnit;
    use crate::variable::custom_variable::{VariableValue, VariableValueType};
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DataFlow {
        Kline(Kline),
        Indicator(Indicator),
    }

    impl From<Kline> for DataFlow {
        fn from(kline: Kline) -> Self {
            DataFlow::Kline(kline)
        }
    }

    impl From<Indicator> for DataFlow {
        fn from(indicator: Indicator) -> Self {
            DataFlow::Indicator(indicator)
        }
    }

    /// Data flow trigger
    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DataFlowTrigger {
        pub from_node_type: String,
        pub from_node_id: String,
        pub from_node_name: String,
        pub from_handle_id: String,
        pub from_var: String,
        pub from_var_display_name: String,
        pub from_var_value_type: VariableValueType,
        pub from_var_config_id: i32,
        pub expire_duration: ExpireDuration,
        pub error_policy: HashMap<DataflowErrorType, DataflowErrorPolicy>,
    }

    #[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
    #[serde(rename_all = "camelCase")]
    pub enum DataflowErrorType {
        NullValue,
        Expired,
        ZeroValue,
    }

    #[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
    #[serde(rename_all = "camelCase")]
    #[serde(tag = "strategy")]
    pub enum DataflowErrorPolicy {
        StillUpdate(StillUpdatePolicy),
        Skip(SkipPolicy),
        ValueReplace(ValueReplacePolicy),
        UsePreviousValue(UsePreviousValuePolicy),
    }

    #[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
    #[serde(rename_all = "camelCase")]
    pub struct ValueReplacePolicy {
        pub replace_value: VariableValue,
        pub error_log: ErrorLog,
    }

    #[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
    #[serde(rename_all = "camelCase")]
    pub struct UsePreviousValuePolicy {
        pub max_use_times: Option<u32>, // Maximum usage count, None means unlimited
        pub error_log: ErrorLog,
    }

    #[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
    #[serde(rename_all = "camelCase")]
    pub struct StillUpdatePolicy {
        pub error_log: ErrorLog,
    }

    #[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
    #[serde(rename_all = "camelCase")]
    pub struct SkipPolicy {
        pub error_log: ErrorLog,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct ExpireDuration {
        pub unit: TimeUnit,
        pub duration: u32,
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub enum ErrorLog {
        NoNotify,
        Notify { level: LogLevel },
    }

    // Custom deserialization implementation
    impl<'de> Deserialize<'de> for ErrorLog {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::Error;

            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct ErrorLogHelper {
                notify: bool,
                level: Option<LogLevel>,
            }

            let helper = ErrorLogHelper::deserialize(deserializer)?;

            if helper.notify {
                // When notify is true, level is required
                let level = helper
                    .level
                    .ok_or_else(|| D::Error::custom("level is required when notify is true"))?;
                Ok(ErrorLog::Notify { level })
            } else {
                // When notify is false, map to NoNotify
                Ok(ErrorLog::NoNotify)
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
    #[serde(rename_all = "lowercase")]
    pub enum LogLevel {
        Warn,
        Error,
    }
}

// ==================== Unified Trigger Configuration Type ====================

// Re-export submodule types
pub use dataflow::DataFlowTrigger;
pub use timer::TimerTrigger;

pub use crate::node_infra::condition_trigger::ConditionTrigger;

/// Trigger type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerType {
    Condition,
    Timer,
    Dataflow,
}

/// Trigger configuration (supports timer, condition, and dataflow)
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "config", rename_all = "lowercase")]
pub enum TriggerConfig {
    Timer(TimerTrigger),
    Condition(ConditionTrigger),
    Dataflow(DataFlowTrigger),
}

impl TriggerConfig {
    /// Check if timer trigger
    pub fn is_timer(&self) -> bool {
        matches!(self, TriggerConfig::Timer { .. })
    }

    /// Check if condition trigger
    pub fn is_condition(&self) -> bool {
        matches!(self, TriggerConfig::Condition { .. })
    }

    /// Check if dataflow trigger
    pub fn is_dataflow(&self) -> bool {
        matches!(self, TriggerConfig::Dataflow { .. })
    }
}
