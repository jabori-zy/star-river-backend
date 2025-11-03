use serde::{Deserialize, Serialize};

// ==================== Timer 模块 ====================
pub mod timer {
    use serde::{Deserialize, Serialize};

    /// 时间单位
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum TimeUnit {
        Second,
        Minute,
        Hour,
        Day,
    }

    /// 重复模式
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum RepeatMode {
        Hourly,
        Daily,
        Weekly,
        Monthly,
    }

    /// 间隔定时器配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct IntervalTimerConfig {
        // mode 由外层枚举的 #[serde(tag = "mode")] 处理
        pub interval: i32,
        pub unit: TimeUnit,
    }

    /// 月份中的某天（数字、"first" 或 "last"）
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum DayOfMonth {
        Number(u32),
        #[serde(rename = "first")]
        First,
        #[serde(rename = "last")]
        Last,
    }

    /// 月度回退策略
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum MonthlyFallbackStrategy {
        LastDay,
        Skip,
    }

    /// 小时调度配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HourlyScheduledConfig {
        pub cron_expression: String,
        pub hourly_interval: i32,
        pub minute_of_hour: i32,
    }

    /// 每日调度配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DailyScheduledConfig {
        pub cron_expression: String,
        pub time: String,
        pub days_of_week: Vec<i32>,
    }

    /// 每周调度配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct WeeklyScheduledConfig {
        pub cron_expression: String,
        pub time: String,
        pub day_of_week: i32,
    }

    /// 每月调度配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MonthlyScheduledConfig {
        pub cron_expression: String,
        pub time: String,
        pub day_of_month: DayOfMonth,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub monthly_fallback: Option<MonthlyFallbackStrategy>,
    }

    /// 调度定时器配置（按 repeatMode 区分）
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "repeatMode", rename_all = "lowercase")]
    pub enum ScheduledTimerConfig {
        Hourly(HourlyScheduledConfig),
        Daily(DailyScheduledConfig),
        Weekly(WeeklyScheduledConfig),
        Monthly(MonthlyScheduledConfig),
    }

    impl ScheduledTimerConfig {
        /// 判断是否为小时调度
        pub fn is_hourly(&self) -> bool {
            matches!(self, ScheduledTimerConfig::Hourly(_))
        }

        /// 判断是否为每日调度
        pub fn is_daily(&self) -> bool {
            matches!(self, ScheduledTimerConfig::Daily(_))
        }

        /// 判断是否为每周调度
        pub fn is_weekly(&self) -> bool {
            matches!(self, ScheduledTimerConfig::Weekly(_))
        }

        /// 判断是否为每月调度
        pub fn is_monthly(&self) -> bool {
            matches!(self, ScheduledTimerConfig::Monthly(_))
        }

        /// 获取 cron 表达式
        pub fn cron_expression(&self) -> &str {
            match self {
                ScheduledTimerConfig::Hourly(config) => &config.cron_expression,
                ScheduledTimerConfig::Daily(config) => &config.cron_expression,
                ScheduledTimerConfig::Weekly(config) => &config.cron_expression,
                ScheduledTimerConfig::Monthly(config) => &config.cron_expression,
            }
        }
    }

    /// 定时器触发配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "mode", rename_all = "lowercase")]
    pub enum TimerTrigger {
        Interval(IntervalTimerConfig),
        Scheduled(ScheduledTimerConfig),
    }
}

// ==================== Condition 模块 ====================
pub mod condition {
    use serde::{Deserialize, Serialize};

    /// Case 分支触发器
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CaseBranchTrigger {
        pub from_node_type: String,
        pub from_handle_id: String,
        pub from_node_id: String,
        pub from_node_name: String,
        pub case_id: i32,
    }

    /// Else 分支触发器
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ElseBranchTrigger {
        pub from_node_type: String,
        pub from_handle_id: String,
        pub from_node_id: String,
        pub from_node_name: String,
    }

    /// 条件触发器（Case 或 Else 分支）
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "triggerType", rename_all = "lowercase")]
    pub enum ConditionTrigger {
        Case(CaseBranchTrigger),
        Else(ElseBranchTrigger),
    }

    impl ConditionTrigger {
        /// 判断是否为 Case 分支
        pub fn is_case(&self) -> bool {
            matches!(self, ConditionTrigger::Case(_))
        }

        /// 判断是否为 Else 分支
        pub fn is_else(&self) -> bool {
            matches!(self, ConditionTrigger::Else(_))
        }

        /// 获取来源节点 ID
        pub fn from_node_id(&self) -> &str {
            match self {
                ConditionTrigger::Case(trigger) => &trigger.from_node_id,
                ConditionTrigger::Else(trigger) => &trigger.from_node_id,
            }
        }

        /// 获取来源节点名称
        pub fn from_node_name(&self) -> &str {
            match self {
                ConditionTrigger::Case(trigger) => &trigger.from_node_name,
                ConditionTrigger::Else(trigger) => &trigger.from_node_name,
            }
        }
    }
}

// ==================== DataFlow 模块 ====================
pub mod dataflow {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use ta_lib::Indicator;

    use crate::{
        market::Kline,
        node::variable_node::trigger::timer::TimeUnit,
        strategy::custom_variable::{VariableValue, VariableValueType},
    };

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

    /// 数据流触发器
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
        pub max_use_times: Option<u32>, // 最大使用次数，如果为None，则表示无限使用
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

    // 自定义反序列化实现
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
                // notify 为 true 时，必须有 level
                let level = helper
                    .level
                    .ok_or_else(|| D::Error::custom("level is required when notify is true"))?;
                Ok(ErrorLog::Notify { level })
            } else {
                // notify 为 false 时，映射到 NoNotify
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

// ==================== 触发配置统一类型 ====================

// 重新导出子模块类型
pub use condition::ConditionTrigger;
pub use dataflow::DataFlowTrigger;
pub use timer::TimerTrigger;

/// 触发类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerType {
    Condition,
    Timer,
    Dataflow,
}

/// 触发配置（支持定时器、条件、数据流三种类型）
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "config", rename_all = "lowercase")]
pub enum TriggerConfig {
    Timer(TimerTrigger),
    Condition(ConditionTrigger),
    Dataflow(DataFlowTrigger),
}

impl TriggerConfig {
    /// 判断是否为定时器触发
    pub fn is_timer(&self) -> bool {
        matches!(self, TriggerConfig::Timer { .. })
    }

    /// 判断是否为条件触发
    pub fn is_condition(&self) -> bool {
        matches!(self, TriggerConfig::Condition { .. })
    }

    /// 判断是否为数据流触发
    pub fn is_dataflow(&self) -> bool {
        matches!(self, TriggerConfig::Dataflow { .. })
    }
}
