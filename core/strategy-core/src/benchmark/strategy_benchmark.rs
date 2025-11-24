use std::collections::{BTreeMap, HashMap, VecDeque};

use serde::Serialize;
use snafu::{Backtrace, OptionExt, Snafu};
use star_river_core::{
    custom_type::{CycleId, NodeId, StrategyId, StrategyName},
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode},
};
use tokio::time::{Duration, Instant};
use utoipa::ToSchema;

use super::node_benchmark::{CompletedCycle, NodeBenchmark, NodeCycleReport, NodePerformanceReport};

// ============================================================
// 第一部分：StrategyCycleTracker - 策略单周期追踪器
// ============================================================

/// 策略单周期追踪器（支持 phase 追踪）
#[derive(Debug)]
pub struct StrategyCycleTracker {
    cycle_id: CycleId,
    start_time: Instant,
    phase_durations: Vec<(String, Duration)>, // 使用 String 支持动态内容
}

impl StrategyCycleTracker {
    pub fn new(cycle_id: CycleId) -> Self {
        Self {
            cycle_id,
            start_time: Instant::now(),
            phase_durations: Vec::new(),
        }
    }

    /// 开始追踪一个阶段
    #[inline]
    pub fn start_phase(&mut self, phase_name: impl Into<String>) {
        // 直接记录当前时间作为阶段开始
        self.phase_durations
            .push((phase_name.into(), Instant::now().duration_since(self.start_time)));
    }

    /// 结束追踪一个阶段
    #[inline]
    pub fn end_phase(&mut self, phase_name: impl AsRef<str>) {
        let phase_name = phase_name.as_ref();
        // 查找对应的开始时间并计算持续时间
        if let Some(pos) = self.phase_durations.iter().rposition(|(name, _)| name == phase_name) {
            let phase_start = self.phase_durations[pos].1;
            let phase_duration = Instant::now().duration_since(self.start_time) - phase_start;
            self.phase_durations[pos].1 = phase_duration;
        }
    }

    /// 完成当前周期并返回不可变的完成记录
    #[inline]
    pub fn end(&self) -> CompletedStrategyCycle {
        let total_duration = self.start_time.elapsed();
        CompletedStrategyCycle {
            cycle_id: self.cycle_id,
            total_duration,
            phase_durations: self.phase_durations.clone(),
        }
    }
}

// ============================================================
// 第二部分：CompletedStrategyCycleTracker - 已完成的周期
// ============================================================

/// 已完成的策略周期追踪器（不可变）
#[derive(Debug, Clone)]
pub struct CompletedStrategyCycle {
    cycle_id: CycleId,
    total_duration: Duration,
    phase_durations: Vec<(String, Duration)>,
}

impl CompletedStrategyCycle {
    pub fn get_total_duration(&self) -> Duration {
        self.total_duration
    }

    pub fn get_cycle_id(&self) -> CycleId {
        self.cycle_id
    }

    pub fn get_phase_duration(&self, phase_name: impl AsRef<str>) -> Option<Duration> {
        let phase_name = phase_name.as_ref();
        self.phase_durations
            .iter()
            .find(|(name, _)| name == phase_name)
            .map(|(_, duration)| *duration)
    }

    pub fn get_all_phase_durations(&self) -> &[(String, Duration)] {
        &self.phase_durations
    }

    /// 生成单个周期的详细报告(不包含节点报告,需要在 StrategyBenchmark 层面添加)
    pub fn get_cycle_report(&self) -> StrategyCycleReport {
        StrategyCycleReport {
            cycle_id: self.cycle_id,
            total_duration: self.total_duration,
            phase_durations: self.phase_durations.clone(),
            node_cycle_reports: Vec::new(),      // 这里留空,在 StrategyBenchmark 中填充
            node_execute_percentage: Vec::new(), // 这里留空,在 StrategyBenchmark 中填充
        }
    }
}

// ============================================================
// 第三部分：StrategyCycleReport - 单周期报告
// ============================================================

/// 策略单个周期的详细报告
#[derive(Debug, Clone)]
pub struct StrategyCycleReport {
    pub cycle_id: CycleId,
    pub total_duration: Duration,
    pub phase_durations: Vec<(String, Duration)>,
    pub node_cycle_reports: Vec<NodeCycleReport>,    // 该周期内所有节点的周期报告
    pub node_execute_percentage: Vec<(NodeId, f64)>, // 该周期内所有节点的执行占比
}

impl StrategyCycleReport {
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }

    pub fn cycle_id(&self) -> CycleId {
        self.cycle_id
    }

    /// 获取某个阶段的耗时占比
    pub fn phase_percentage(&self, phase_name: &str) -> f64 {
        if self.total_duration.is_zero() {
            return 0.0;
        }

        if let Some(duration) = self.phase_durations.iter().find(|(name, _)| name == phase_name) {
            let phase_ns = duration.1.as_nanos() as f64;
            let total_ns = self.total_duration.as_nanos() as f64;
            (phase_ns / total_ns) * 100.0
        } else {
            0.0
        }
    }

    /// 获取最耗时的阶段
    pub fn slowest_phase(&self) -> Option<(&String, Duration)> {
        self.phase_durations
            .iter()
            .max_by_key(|(_, duration)| duration)
            .map(|(name, duration)| (name, *duration))
    }
}

impl std::fmt::Display for StrategyCycleReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n")?;
        writeln!(f, "┌─ Strategy Cycle Report [Cycle ID: {}]", self.cycle_id)?;
        writeln!(f, "│  Total Duration: {:?}", self.total_duration)?;

        if !self.phase_durations.is_empty() {
            writeln!(f, "├─ Strategy Phase Details:")?;

            for (i, (phase_name, duration)) in self.phase_durations.iter().enumerate() {
                let percentage = if !self.total_duration.is_zero() {
                    (duration.as_nanos() as f64 / self.total_duration.as_nanos() as f64) * 100.0
                } else {
                    0.0
                };

                let prefix = if i == self.phase_durations.len() - 1 { "└" } else { "├" };
                writeln!(f, "│  {} {}: {:?} ({:.2}%)", prefix, phase_name, duration, percentage)?;
            }
        }

        if let Some((slowest_name, slowest_duration)) = self.slowest_phase() {
            writeln!(f, "├─ Slowest Strategy Phase: {} ({:?})", slowest_name, slowest_duration)?;
        }

        // 显示节点执行占比
        if !self.node_execute_percentage.is_empty() {
            writeln!(f, "├─ Node Execute Percentage:")?;

            // 按占比降序排序
            let mut sorted_percentages = self.node_execute_percentage.clone();
            sorted_percentages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            for (i, (node_id, percentage)) in sorted_percentages.iter().enumerate() {
                let is_last = i == sorted_percentages.len() - 1;
                let prefix = if is_last { "└" } else { "├" };
                writeln!(f, "│  {} {}: {:.2}%", prefix, node_id, percentage)?;
            }
        }

        // 显示所有节点的周期报告
        if !self.node_cycle_reports.is_empty() {
            writeln!(f, "├─ Node Cycle Reports ({} nodes):", self.node_cycle_reports.len())?;

            for (i, node_report) in self.node_cycle_reports.iter().enumerate() {
                let is_last_node = i == self.node_cycle_reports.len() - 1;
                let node_prefix = if is_last_node { "└" } else { "├" };

                writeln!(f, "│  {} {} ({})", node_prefix, node_report.node_name, node_report.node_id)?;
                writeln!(
                    f,
                    "│  {}   Total Duration: {:?}",
                    if is_last_node { " " } else { "│" },
                    node_report.duration
                )?;

                if !node_report.phase_durations.is_empty() {
                    writeln!(f, "│  {}   Phases:", if is_last_node { " " } else { "│" })?;
                    for (j, (phase_name, phase_duration)) in node_report.phase_durations.iter().enumerate() {
                        let phase_percentage = if !node_report.duration.is_zero() {
                            (phase_duration.as_nanos() as f64 / node_report.duration.as_nanos() as f64) * 100.0
                        } else {
                            0.0
                        };

                        let is_last_phase = j == node_report.phase_durations.len() - 1;
                        let phase_prefix = if is_last_phase { "└" } else { "├" };

                        writeln!(
                            f,
                            "│  {}     {} {}: {:?} ({:.2}%)",
                            if is_last_node { " " } else { "│" },
                            phase_prefix,
                            phase_name,
                            phase_duration,
                            phase_percentage
                        )?;
                    }
                }
            }
        }

        write!(f, "└─────────────────────────────")
    }
}

// ============================================================
// 第四部分：StrategyPhaseBenchmark - 策略阶段性能统计
// ============================================================

#[derive(Debug, Clone)]
pub struct StrategyPhaseBenchmark {
    pub phase_name: String,
    pub total_cycles: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub total_duration: Duration,
    pub total_duration_ns: u128,
    pub sum_squared_diff_ns: f64,
    pub all_phase_durations: BTreeMap<u64, usize>, // 存储纳秒值(key: 纳秒, value: 该耗时的周期数)
    pub duration_percentage: f32,                  // 耗时占比
}

impl StrategyPhaseBenchmark {
    /// 创建新的策略阶段性能基准
    pub fn new(phase_name: String) -> Self {
        Self {
            phase_name,
            total_cycles: 0,
            avg_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            total_duration: Duration::ZERO,
            total_duration_ns: 0,
            sum_squared_diff_ns: 0.0,
            all_phase_durations: BTreeMap::new(),
            duration_percentage: 0.0,
        }
    }

    /// 添加一个阶段耗时样本
    pub fn add_duration(&mut self, duration: Duration) {
        let duration_ns = duration.as_nanos();

        // 更新计数
        self.total_cycles += 1;
        self.total_duration += duration;
        self.total_duration_ns += duration_ns;

        // 更新最小/最大值
        if duration < self.min_duration {
            self.min_duration = duration;
        }
        if duration > self.max_duration {
            self.max_duration = duration;
        }

        // 更新平均值
        self.avg_duration = Duration::from_nanos((self.total_duration_ns / self.total_cycles as u128) as u64);

        // 更新标准差计算（在线算法）
        let avg_ns = self.total_duration_ns as f64 / self.total_cycles as f64;
        let diff = duration_ns as f64 - avg_ns;
        self.sum_squared_diff_ns += diff * diff;

        // 插入到 BTreeMap (存储纳秒值)
        let duration_ns_u64 = duration_ns as u64;
        *self.all_phase_durations.entry(duration_ns_u64).or_insert(0) += 1;
    }

    /// 获取标准差
    pub fn std_deviation(&self) -> Duration {
        if self.total_cycles <= 1 {
            return Duration::ZERO;
        }
        let variance = self.sum_squared_diff_ns / (self.total_cycles - 1) as f64;
        Duration::from_nanos(variance.sqrt() as u64)
    }

    /// 获取百分位数（基于 BTreeMap，O(log n) 复杂度）
    pub fn percentile(&self, p: f64) -> Duration {
        if self.all_phase_durations.is_empty() || !(0.0..=1.0).contains(&p) {
            return Duration::ZERO;
        }

        // 计算目标索引（基于总样本数）
        let target_index = ((self.total_cycles as f64 - 1.0) * p) as usize;

        // 遍历 BTreeMap（已排序），累计计数直到达到目标索引
        let mut accumulated = 0;
        for (duration_ns, count) in &self.all_phase_durations {
            accumulated += count;
            if accumulated > target_index {
                return Duration::from_nanos(*duration_ns);
            }
        }

        // 如果未找到（理论上不应该发生），返回最大值
        self.all_phase_durations
            .keys()
            .last()
            .copied()
            .map(Duration::from_nanos)
            .unwrap_or(Duration::ZERO)
    }

    /// 生成阶段报告
    pub fn report(&self, strategy_total_duration_ns: u128) -> StrategyPhaseReport {
        let duration_percentage = if strategy_total_duration_ns == 0 {
            0.0
        } else {
            (self.total_duration_ns as f64 / strategy_total_duration_ns as f64 * 100.0) as f32
        };

        StrategyPhaseReport {
            phase_name: self.phase_name.clone(),
            total_cycles: self.total_cycles,
            avg_duration: self.avg_duration,
            min_duration: self.min_duration,
            max_duration: self.max_duration,
            total_duration: self.total_duration,
            duration_percentage,
            p25: self.percentile(0.25),
            p50: self.percentile(0.5),
            p75: self.percentile(0.75),
            p95: self.percentile(0.95),
            p99: self.percentile(0.99),
        }
    }
}

// ============================================================
// 第五部分：StrategyBenchmark - 策略性能统计
// ============================================================

/// 策略性能统计（聚合多个周期的数据）
#[derive(Debug)]
pub struct StrategyBenchmark {
    pub strategy_id: StrategyId,
    pub strategy_name: String,

    // 使用 VecDeque 作为环形缓冲区，限制内存使用
    all_strategy_cycles: VecDeque<CompletedStrategyCycle>,
    all_strategy_cycle_durations: BTreeMap<u64, usize>, // 存储纳秒值(key: 纳秒, value: 该耗时的周期数)
    all_phase_benchmarks: HashMap<String, StrategyPhaseBenchmark>, // 所有阶段的耗时(key: 阶段名, value: 阶段性能统计)
    node_benchmarks: HashMap<NodeId, NodeBenchmark>,
    max_history: usize,

    // 实时统计数据
    total_cycles: usize,
    total_duration_ns: u128,
    avg_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,

    // 用于计算标准差的在线统计
    sum_squared_diff_ns: f64,
}

impl StrategyBenchmark {
    pub fn new(strategy_id: StrategyId, strategy_name: StrategyName) -> Self {
        Self {
            strategy_id,
            strategy_name,
            all_strategy_cycles: VecDeque::with_capacity(1000),
            all_strategy_cycle_durations: BTreeMap::new(),
            all_phase_benchmarks: HashMap::new(),
            node_benchmarks: HashMap::new(),
            max_history: 1000,
            total_cycles: 0,
            total_duration_ns: 0,
            avg_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            sum_squared_diff_ns: 0.0,
        }
    }

    pub fn add_node_benchmark(&mut self, node_id: NodeId, node_name: String, node_type: String) {
        let node_benchmark = NodeBenchmark::new(node_type, node_id.clone(), node_name);
        self.node_benchmarks.insert(node_id, node_benchmark);
    }

    pub fn add_complete_node_cycle(&mut self, node_id: NodeId, cycle_tracker: CompletedCycle) -> Result<(), NodeBenchmarkNotFountError> {
        self.node_benchmarks
            .get_mut(&node_id)
            .context(NodeBenchmarkNotFountSnafu { node_id: node_id.clone() })?
            .add_completed_cycle(cycle_tracker);
        Ok(())
    }

    /// 添加一个完成的周期追踪器
    pub fn add_cycle_tracker(&mut self, cycle_tracker: CompletedStrategyCycle) {
        let duration = cycle_tracker.get_total_duration();
        let duration_ns = duration.as_nanos();

        // 更新统计数据
        self.total_cycles += 1;
        self.total_duration_ns += duration_ns;
        self.avg_duration = Duration::from_nanos((self.total_duration_ns / self.total_cycles as u128) as u64);

        // 更新最小/最大值
        if duration < self.min_duration {
            self.min_duration = duration;
        }
        if duration > self.max_duration {
            self.max_duration = duration;
        }

        // 更新标准差计算（在线算法）
        let avg_ns = self.total_duration_ns as f64 / self.total_cycles as f64;
        let diff = duration_ns as f64 - avg_ns;
        self.sum_squared_diff_ns += diff * diff;

        // 插入到 all_cycle_durations (BTreeMap) - 存储纳秒值
        let duration_ns_u64 = duration_ns as u64;
        *self.all_strategy_cycle_durations.entry(duration_ns_u64).or_insert(0) += 1;

        // 更新 all_phase_benchmarks (HashMap<String, StrategyPhaseBenchmark>)
        for (phase_name, phase_duration) in cycle_tracker.get_all_phase_durations() {
            self.all_phase_benchmarks
                .entry(phase_name.clone())
                .or_insert_with(|| StrategyPhaseBenchmark::new(phase_name.clone()))
                .add_duration(*phase_duration);
        }

        // 保存详细数据（使用环形缓冲区）
        if self.all_strategy_cycles.len() >= self.max_history {
            self.all_strategy_cycles.pop_front();
        }
        self.all_strategy_cycles.push_back(cycle_tracker);
    }

    /// 获取平均执行时间
    pub fn avg_duration(&self) -> Duration {
        if self.total_cycles == 0 {
            return Duration::ZERO;
        }
        Duration::from_nanos((self.total_duration_ns / self.total_cycles as u128) as u64)
    }

    /// 获取标准差
    pub fn std_deviation(&self) -> Duration {
        if self.total_cycles <= 1 {
            return Duration::ZERO;
        }
        let variance = self.sum_squared_diff_ns / (self.total_cycles - 1) as f64;
        Duration::from_nanos(variance.sqrt() as u64)
    }

    /// 获取中位数
    pub fn median_duration(&self) -> Duration {
        if self.all_strategy_cycles.is_empty() {
            return Duration::ZERO;
        }

        let mut durations: Vec<Duration> = self.all_strategy_cycles.iter().map(|t| t.get_total_duration()).collect();

        durations.sort_unstable();
        durations[durations.len() / 2]
    }

    /// 获取百分位数（基于 BTreeMap，O(log n) 复杂度）
    pub fn percentile(&self, p: f64) -> Duration {
        if self.all_strategy_cycle_durations.is_empty() || !(0.0..=1.0).contains(&p) {
            return Duration::ZERO;
        }

        // 计算目标索引（基于总样本数）
        let target_index = ((self.total_cycles as f64 - 1.0) * p) as usize;

        // 遍历 BTreeMap（已排序），累计计数直到达到目标索引
        let mut accumulated = 0;
        for (duration_ns, count) in &self.all_strategy_cycle_durations {
            accumulated += count;
            if accumulated > target_index {
                return Duration::from_nanos(*duration_ns);
            }
        }

        // 如果未找到（理论上不应该发生），返回最大值
        self.all_strategy_cycle_durations
            .keys()
            .last()
            .copied()
            .map(Duration::from_nanos)
            .unwrap_or(Duration::ZERO)
    }

    /// 获取最近N个周期的平均值
    pub fn recent_avg_duration(&self, n: usize) -> Duration {
        if self.all_strategy_cycles.is_empty() {
            return Duration::ZERO;
        }

        let count = n.min(self.all_strategy_cycles.len());
        let sum: Duration = self
            .all_strategy_cycles
            .iter()
            .rev()
            .take(count)
            .map(|t| t.get_total_duration())
            .sum();

        sum / count as u32
    }

    /// 检测性能是否退化
    pub fn detect_performance_degradation(&self, recent_count: usize, threshold: f64) -> bool {
        if self.total_cycles < recent_count * 2 {
            return false;
        }

        let recent_avg = self.recent_avg_duration(recent_count);
        let overall_avg = self.avg_duration;

        if overall_avg.is_zero() {
            return false;
        }

        let ratio = recent_avg.as_nanos() as f64 / overall_avg.as_nanos() as f64;
        ratio > (1.0 + threshold)
    }

    /// 分析策略级别的阶段性能瓶颈
    pub fn get_phase_report(&self) -> Vec<StrategyPhaseReport> {
        if self.all_phase_benchmarks.is_empty() {
            return Vec::new();
        }

        // 计算每个阶段的耗时占比，然后生成报告
        let strategy_total_duration_ns = self.total_duration_ns;
        let mut results: Vec<StrategyPhaseReport> = self
            .all_phase_benchmarks
            .values()
            .map(|benchmark| benchmark.report(strategy_total_duration_ns))
            .collect();

        // 按平均耗时降序排序
        results.sort_by(|a, b| b.avg_duration.cmp(&a.avg_duration));
        results
    }

    /// 生成性能报告
    pub fn report(&self) -> StrategyPerformanceReport {
        // 收集所有节点的性能报告
        let mut node_reports: Vec<NodePerformanceReport> = self
            .node_benchmarks
            .values()
            .map(|node_benchmark| node_benchmark.report(self.avg_duration))
            .collect();

        // 按平均执行时间降序排序
        node_reports.sort_by(|a, b| b.avg_duration.cmp(&a.avg_duration));

        // 获取策略级别的阶段报告
        let strategy_phase_reports = self.get_phase_report();

        StrategyPerformanceReport {
            total_cycles: self.total_cycles,
            avg_duration: self.avg_duration,
            min_duration: self.min_duration,
            max_duration: self.max_duration,
            std_deviation: self.std_deviation(),
            p25: self.percentile(0.25),
            p50: self.percentile(0.5),
            p75: self.percentile(0.75),
            p95: self.percentile(0.95),
            p99: self.percentile(0.99),
            recent_avg_100: self.recent_avg_duration(100),
            strategy_phase_reports,
            node_reports,
        }
    }

    /// 重置统计数据
    pub fn reset(&mut self) {
        self.all_strategy_cycles.clear();
        self.all_strategy_cycle_durations.clear();
        self.all_phase_benchmarks.clear();
        self.total_cycles = 0;
        self.total_duration_ns = 0;
        self.avg_duration = Duration::ZERO;
        self.min_duration = Duration::MAX;
        self.max_duration = Duration::ZERO;
        self.sum_squared_diff_ns = 0.0;
        self.node_benchmarks.values_mut().for_each(|node_benchmark| node_benchmark.reset());
    }

    /// 获取最近N个周期的详细报告(包含节点报告)
    pub fn get_recent_cycle_reports(&self, n: usize) -> Vec<StrategyCycleReport> {
        self.all_strategy_cycles
            .iter()
            .rev()
            .take(n)
            .map(|strategy_tracker| {
                let cycle_id = strategy_tracker.get_cycle_id();
                let strategy_total_duration = strategy_tracker.get_total_duration();

                // 收集该 cycle_id 下所有节点的周期报告和执行占比
                let mut node_cycle_reports = Vec::new();
                let mut node_execute_percentage = Vec::new();

                for (node_id, node_benchmark) in &self.node_benchmarks {
                    // 从节点的历史数据中查找匹配 cycle_id 的报告
                    if let Some(node_report) = node_benchmark.cycle_report_by_cycle_id(cycle_id) {
                        // 计算节点执行占比
                        let percentage = if !strategy_total_duration.is_zero() {
                            (node_report.duration.as_nanos() as f64 / strategy_total_duration.as_nanos() as f64) * 100.0
                        } else {
                            0.0
                        };

                        node_cycle_reports.push(node_report);
                        node_execute_percentage.push((node_id.clone(), percentage));
                    }
                }

                StrategyCycleReport {
                    cycle_id: strategy_tracker.get_cycle_id(),
                    total_duration: strategy_total_duration,
                    phase_durations: strategy_tracker.get_all_phase_durations().to_vec(),
                    node_cycle_reports,
                    node_execute_percentage,
                }
            })
            .collect()
    }

    /// 获取最近一个周期的报告(包含节点报告)
    pub fn get_last_cycle_report(&self) -> Option<StrategyCycleReport> {
        let strategy_tracker = self.all_strategy_cycles.back()?;
        let cycle_id = strategy_tracker.get_cycle_id();
        let strategy_total_duration = strategy_tracker.get_total_duration();

        // 收集该 cycle_id 下所有节点的周期报告和执行占比
        let mut node_cycle_reports = Vec::new();
        let mut node_execute_percentage = Vec::new();

        for (node_id, node_benchmark) in &self.node_benchmarks {
            // 从节点的历史数据中查找匹配 cycle_id 的报告
            if let Some(node_report) = node_benchmark.cycle_report_by_cycle_id(cycle_id) {
                // 计算节点执行占比
                let percentage = if !strategy_total_duration.is_zero() {
                    (node_report.duration.as_nanos() as f64 / strategy_total_duration.as_nanos() as f64) * 100.0
                } else {
                    0.0
                };

                node_cycle_reports.push(node_report);
                node_execute_percentage.push((node_id.clone(), percentage));
            }
        }

        Some(StrategyCycleReport {
            cycle_id: strategy_tracker.get_cycle_id(),
            total_duration: strategy_total_duration,
            phase_durations: strategy_tracker.get_all_phase_durations().to_vec(),
            node_cycle_reports,
            node_execute_percentage,
        })
    }

    /// 获取总周期数
    pub fn get_total_cycles(&self) -> usize {
        self.total_cycles
    }
}

// ============================================================
// 第五部分：StrategyPerformanceReport - 策略性能报告
// ============================================================

/// 策略级别的阶段统计信息
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyPhaseReport {
    pub phase_name: String,
    pub total_cycles: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub total_duration: Duration,
    pub duration_percentage: f32,
    pub p25: Duration,
    pub p50: Duration,
    pub p75: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

impl std::fmt::Display for StrategyPhaseReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "  ├─ {}: avg={:?}, min={:?}, max={:?}, cycles={}, percentage={:.2}%",
            self.phase_name, self.avg_duration, self.min_duration, self.max_duration, self.total_cycles, self.duration_percentage
        )
    }
}

/// 策略整体性能报告
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyPerformanceReport {
    pub total_cycles: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub std_deviation: Duration,
    pub p25: Duration,
    pub p50: Duration,
    pub p75: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub recent_avg_100: Duration,
    pub strategy_phase_reports: Vec<StrategyPhaseReport>,
    pub node_reports: Vec<NodePerformanceReport>,
}

impl std::fmt::Display for StrategyPerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n")?;
        writeln!(f, "╔═══════════════════════════════════════════════════════════╗")?;
        writeln!(f, "║ Strategy Performance Report")?;
        writeln!(f, "╠═══════════════════════════════════════════════════════════╣")?;
        writeln!(f, "║ Total Cycles: {}", self.total_cycles)?;
        writeln!(f, "╟───────────────────────────────────────────────────────────╢")?;
        writeln!(f, "║ Duration Statistics:")?;
        writeln!(f, "║  ├─ Average:      {:>12?}", self.avg_duration)?;
        writeln!(f, "║  ├─ Min:          {:>12?}", self.min_duration)?;
        writeln!(f, "║  ├─ Max:          {:>12?}", self.max_duration)?;
        writeln!(f, "║  ├─ StdDev:       {:>12?}", self.std_deviation)?;
        writeln!(f, "║  ├─ P25:          {:>12?}", self.p25)?;
        writeln!(f, "║  ├─ P50:          {:>12?}", self.p50)?;
        writeln!(f, "║  ├─ P75:          {:>12?}", self.p75)?;
        writeln!(f, "║  ├─ P95:          {:>12?}", self.p95)?;
        writeln!(f, "║  ├─ P99:          {:>12?}", self.p99)?;
        writeln!(f, "║  └─ Recent(100):  {:>12?}", self.recent_avg_100)?;

        if !self.strategy_phase_reports.is_empty() {
            writeln!(f, "╟───────────────────────────────────────────────────────────╢")?;
            writeln!(f, "║ Strategy Phase Reports (top 5):")?;
            for (i, phase) in self.strategy_phase_reports.iter().take(5).enumerate() {
                let prefix = if i == self.strategy_phase_reports.len().min(5) - 1 {
                    "└"
                } else {
                    "├"
                };
                writeln!(
                    f,
                    "║  {} {}: avg={:?}, cycles={}, percentage={:.2}%",
                    prefix, phase.phase_name, phase.avg_duration, phase.total_cycles, phase.duration_percentage
                )?;
            }
        }

        if !self.node_reports.is_empty() {
            writeln!(f, "╟───────────────────────────────────────────────────────────╢")?;
            writeln!(f, "║ Node Performance Reports:")?;
            writeln!(f, "╟───────────────────────────────────────────────────────────╢")?;

            for (i, node_report) in self.node_reports.iter().enumerate() {
                writeln!(f, "║")?;
                writeln!(f, "║ {}. {} [{}]", i + 1, node_report.node_name, node_report.node_type)?;
                writeln!(f, "║    Node ID: {}", node_report.node_id)?;
                writeln!(f, "║    Total Cycles: {}", node_report.total_cycles)?;
                writeln!(
                    f,
                    "║    ├─ Average:      {:>12?} ({:.2}%)",
                    node_report.avg_duration, node_report.avg_duration_percentage
                )?;
                writeln!(f, "║    ├─ Min:          {:>12?}", node_report.min_duration)?;
                writeln!(f, "║    ├─ Max:          {:>12?}", node_report.max_duration)?;
                writeln!(f, "║    ├─ StdDev:       {:>12?}", node_report.std_deviation)?;
                writeln!(f, "║    ├─ P25:          {:>12?}", node_report.p25)?;
                writeln!(f, "║    ├─ P50:          {:>12?}", node_report.p50)?;
                writeln!(f, "║    ├─ P75:          {:>12?}", node_report.p75)?;
                writeln!(f, "║    ├─ P95:          {:>12?}", node_report.p95)?;
                writeln!(f, "║    ├─ P99:          {:>12?}", node_report.p99)?;
                writeln!(f, "║    └─ Recent(100):  {:>12?}", node_report.recent_avg_100)?;

                if !node_report.phase_reports.is_empty() {
                    writeln!(f, "║    Phase Reports (top 3):")?;
                    for (j, phase) in node_report.phase_reports.iter().take(3).enumerate() {
                        let prefix = if j == node_report.phase_reports.len().min(3) - 1 {
                            "└"
                        } else {
                            "├"
                        };
                        writeln!(
                            f,
                            "║      {} {}: avg={:?} ({:.2}%)",
                            prefix, phase.phase_name, phase.avg_duration, phase.avg_duration_percentage
                        )?;
                    }
                }
            }
        }

        write!(f, "╚═══════════════════════════════════════════════════════════╝")
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
#[snafu(display("node benchmark not found: {node_id}"))]
pub struct NodeBenchmarkNotFountError {
    pub node_id: NodeId,
    pub backtrace: Backtrace,
}

impl StarRiverErrorTrait for NodeBenchmarkNotFountError {
    fn get_prefix(&self) -> &'static str {
        "NODE_BENCHMARK"
    }

    fn error_code(&self) -> ErrorCode {
        self.get_prefix().to_string()
    }

    fn http_status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => {
                format!("节点benchmark未找到: {}", self.node_id)
            }
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        vec![self.error_code()]
    }
}
