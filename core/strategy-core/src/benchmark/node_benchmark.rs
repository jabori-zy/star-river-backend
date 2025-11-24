use std::collections::{BTreeMap, HashMap, VecDeque};

use serde::Serialize;
use star_river_core::custom_type::{CycleId, NodeId};
use tokio::time::{Duration, Instant};
use utoipa::ToSchema;

// ============================================================
// 第一部分：CycleTracker - 单周期追踪器
// ============================================================

#[derive(Debug, Clone)]
pub struct CycleTracker {
    cycle_id: CycleId,
    start_time: Instant,
    phase_durations: Vec<(String, Duration)>, // 使用 String 支持动态内容
}

impl CycleTracker {
    pub fn new(cycle_id: CycleId) -> Self {
        Self {
            cycle_id,
            start_time: Instant::now(),
            phase_durations: Vec::new(),
        }
    }

    #[inline]
    pub fn start_phase(&mut self, phase_name: impl Into<String>) {
        // 直接记录当前时间作为阶段开始
        self.phase_durations
            .push((phase_name.into(), Instant::now().duration_since(self.start_time)));
    }

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

    #[inline]
    pub fn end(self) -> CompletedCycle {
        let total_duration = self.start_time.elapsed();
        CompletedCycle {
            cycle_id: self.cycle_id,
            duration: total_duration,
            phase_durations: self.phase_durations,
        }
    }
}

/// 已完成的周期（不可变）
#[derive(Debug, Clone)]
pub struct CompletedCycle {
    cycle_id: CycleId,
    duration: Duration,                       // 单次循环的总耗时
    phase_durations: Vec<(String, Duration)>, // 各阶段的耗时
}

impl CompletedCycle {
    pub fn get_duration(&self) -> Duration {
        self.duration
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

    pub fn get_cycle_id(&self) -> CycleId {
        self.cycle_id
    }

    /// 生成单个周期的报告
    pub fn get_cycle_report(&self, node_id: NodeId, node_name: String) -> NodeCycleReport {
        NodeCycleReport {
            node_id,
            node_name,
            cycle_id: self.cycle_id,
            duration: self.duration,
            phase_durations: self.phase_durations.clone(),
        }
    }
}

/// 单个周期的详细报告
#[derive(Debug, Clone)]
pub struct NodeCycleReport {
    pub node_id: NodeId,
    pub node_name: String,
    pub cycle_id: CycleId,
    pub duration: Duration,
    pub phase_durations: Vec<(String, Duration)>,
}

impl NodeCycleReport {
    /// 获取某个阶段的耗时占比
    pub fn get_phase_percentage(&self, phase_name: &str) -> f64 {
        if self.duration.is_zero() {
            return 0.0;
        }

        if let Some(duration) = self.phase_durations.iter().find(|(name, _)| name == phase_name) {
            let phase_ns = duration.1.as_nanos() as f64;
            let total_ns = self.duration.as_nanos() as f64;
            (phase_ns / total_ns) * 100.0
        } else {
            0.0
        }
    }

    /// 获取最耗时的阶段
    pub fn get_slowest_phase(&self) -> Option<(&String, Duration)> {
        self.phase_durations
            .iter()
            .max_by_key(|(_, duration)| duration)
            .map(|(name, duration)| (name, *duration))
    }
}

impl std::fmt::Display for NodeCycleReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─ Cycle Report [Play Index: {}]", self.cycle_id)?;
        writeln!(f, "│  Total Duration: {:?}", self.duration)?;

        if !self.phase_durations.is_empty() {
            writeln!(f, "├─ Phase Details:")?;

            for (i, (phase_name, duration)) in self.phase_durations.iter().enumerate() {
                let percentage = if !self.duration.is_zero() {
                    (duration.as_nanos() as f64 / self.duration.as_nanos() as f64) * 100.0
                } else {
                    0.0
                };

                let prefix = if i == self.phase_durations.len() - 1 { "└" } else { "├" };
                writeln!(f, "│  {} {}: {:?} ({:.2}%)", prefix, phase_name, duration, percentage)?;
            }
        }

        if let Some((slowest_name, slowest_duration)) = self.get_slowest_phase() {
            writeln!(f, "├─ Slowest Phase: {} ({:?})", slowest_name, slowest_duration)?;
        }

        write!(f, "└─────────────────────────────")
    }
}

// ============================================================
// 第二部分：NodeBenchmark - 节点性能统计
// ============================================================

#[derive(Debug, Clone)]
pub struct NodePhaseBenchmark {
    pub phase_name: String,
    pub total_cycles: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub total_duration: Duration,
    pub total_duration_ns: u128,
    pub sum_squared_diff_ns: f64,
    pub all_phase_durations: BTreeMap<u64, usize>, // 存储纳秒值(key: 纳秒, value: 该耗时的周期数)
}

impl NodePhaseBenchmark {
    /// 创建新的阶段性能基准
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
    pub fn report(&self, node_avg_duration: Duration) -> NodePhaseReport {
        let duration_percentage = if node_avg_duration.is_zero() {
            0.0
        } else {
            (self.avg_duration.as_nanos() as f64 / node_avg_duration.as_nanos() as f64 * 100.0) as f32
        };

        NodePhaseReport {
            phase_name: self.phase_name.clone(),
            total_cycles: self.total_cycles,
            avg_duration: self.avg_duration,
            min_duration: self.min_duration,
            max_duration: self.max_duration,
            total_duration: self.total_duration,
            avg_duration_percentage: duration_percentage,
            p25: self.percentile(0.25),
            p50: self.percentile(0.5),
            p75: self.percentile(0.75),
            p95: self.percentile(0.95),
            p99: self.percentile(0.99),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodePhaseReport {
    pub phase_name: String,
    pub total_cycles: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub total_duration: Duration,
    pub avg_duration_percentage: f32, // 平均耗时占比
    pub p25: Duration,
    pub p50: Duration,
    pub p75: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

impl std::fmt::Display for NodePhaseReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "  ├─ {}: avg={:?}, min={:?}, max={:?}, cycles={}, percentage={:.2}%",
            self.phase_name, self.avg_duration, self.min_duration, self.max_duration, self.total_cycles, self.avg_duration_percentage
        )
    }
}

#[derive(Debug, Clone)]
pub struct NodeBenchmark {
    pub node_id: NodeId,
    pub node_name: String,
    pub node_type: String,

    // 使用 VecDeque 作为环形缓冲区，限制内存使用
    all_cycles: VecDeque<CompletedCycle>,
    all_cycle_durations: BTreeMap<u64, usize>, // 存储纳秒值(key: 纳秒, value: 该耗时的周期数)
    all_phase_benchmarks: HashMap<String, NodePhaseBenchmark>, // 所有阶段的耗时(key: 阶段名, value: 阶段性能统计)

    max_history: usize, // 最多保留多少个历史周期
    // 实时统计数据
    total_cycles: usize,
    total_duration_ns: u128, // 使用纳秒累加，避免精度损失
    avg_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,

    // 用于计算标准差的在线统计
    sum_squared_diff_ns: f64, // 平方差之和
}

impl NodeBenchmark {
    pub fn new(node_type: String, node_id: NodeId, node_name: String) -> Self {
        Self {
            node_id,
            node_name,
            node_type,
            all_cycles: VecDeque::with_capacity(1000),
            all_cycle_durations: BTreeMap::new(),
            all_phase_benchmarks: HashMap::new(),
            max_history: 1000, // 只保留最近1000个周期的详细数据
            total_cycles: 0,
            total_duration_ns: 0,
            avg_duration: Duration::ZERO,
            min_duration: Duration::MAX, // ❗️❗️❗️初始化为最大值
            max_duration: Duration::ZERO,
            sum_squared_diff_ns: 0.0,
        }
    }

    /// 添加一个完成的周期追踪器
    pub fn add_completed_cycle(&mut self, completed_cycle: CompletedCycle) {
        let duration = completed_cycle.get_duration();
        let duration_ns = duration.as_nanos();

        // 更新统计数据
        self.total_cycles += 1;
        self.total_duration_ns += duration_ns;

        // 更新平均耗时
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
        *self.all_cycle_durations.entry(duration_ns_u64).or_insert(0) += 1;

        // 更新 all_phase_benchmarks (HashMap<String, NodePhaseBenchmark>)
        for (phase_name, phase_duration) in completed_cycle.get_all_phase_durations() {
            self.all_phase_benchmarks
                .entry(phase_name.clone())
                .or_insert_with(|| NodePhaseBenchmark::new(phase_name.clone()))
                .add_duration(*phase_duration);
        }

        // 保存详细数据（使用环形缓冲区）
        if self.all_cycles.len() >= self.max_history {
            self.all_cycles.pop_front(); // 移除最旧的数据
        }
        self.all_cycles.push_back(completed_cycle);
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

    /// 获取百分位数（基于 BTreeMap，O(log n) 复杂度）
    pub fn percentile(&self, p: f64) -> Duration {
        if self.all_cycle_durations.is_empty() || !(0.0..=1.0).contains(&p) {
            return Duration::ZERO;
        }

        // 计算目标索引（基于总样本数）
        let target_index = ((self.total_cycles as f64 - 1.0) * p) as usize;

        // 遍历 BTreeMap（已排序），累计计数直到达到目标索引
        let mut accumulated = 0;
        for (duration_ns, count) in &self.all_cycle_durations {
            accumulated += count;
            if accumulated > target_index {
                return Duration::from_nanos(*duration_ns);
            }
        }

        // 如果未找到（理论上不应该发生），返回最大值
        self.all_cycle_durations
            .keys()
            .last()
            .copied()
            .map(Duration::from_nanos)
            .unwrap_or(Duration::ZERO)
    }

    /// 获取最近N个周期的平均值
    pub fn recent_avg_duration(&self, n: usize) -> Duration {
        if self.all_cycles.is_empty() {
            return Duration::ZERO;
        }

        let count = n.min(self.all_cycles.len());
        let sum: Duration = self.all_cycles.iter().rev().take(count).map(|t| t.get_duration()).sum();

        sum / count as u32
    }

    /// 检测性能是否退化（最近N个周期 vs 总体平均）
    pub fn detect_performance_degradation(&self, recent_count: usize, threshold: f64) -> bool {
        if self.total_cycles < recent_count * 2 {
            return false; // 数据不足
        }

        let recent_avg = self.recent_avg_duration(recent_count);
        let overall_avg = self.avg_duration;

        if overall_avg.is_zero() {
            return false;
        }

        let ratio = recent_avg.as_nanos() as f64 / overall_avg.as_nanos() as f64;
        ratio > (1.0 + threshold) // 如果最近的平均值比总体高出 threshold 倍，则认为性能退化
    }

    // 获取阶段报告
    pub fn get_phase_report(&self) -> Vec<NodePhaseReport> {
        if self.all_phase_benchmarks.is_empty() {
            return Vec::new();
        }

        // 计算每个阶段的耗时占比，然后生成报告
        let mut results: Vec<NodePhaseReport> = self
            .all_phase_benchmarks
            .values()
            .map(|benchmark| benchmark.report(self.avg_duration))
            .collect();

        // 按平均耗时降序排序
        results.sort_by(|a, b| b.avg_duration.cmp(&a.avg_duration));
        results
    }

    /// 生成性能报告
    pub fn report(&self, strategy_avg_duration: Duration) -> NodePerformanceReport {
        let avg_duration_percentage = if strategy_avg_duration.is_zero() {
            0.0
        } else {
            (self.avg_duration.as_nanos() as f64 / strategy_avg_duration.as_nanos() as f64 * 100.0) as f32
        };

        NodePerformanceReport {
            node_id: self.node_id.clone(),
            node_name: self.node_name.clone(),
            node_type: self.node_type.clone(),
            total_cycles: self.total_cycles,
            total_duration_ns: self.total_duration_ns,
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
            avg_duration_percentage,
            phase_reports: self.get_phase_report(),
        }
    }

    /// 重置统计数据
    pub fn reset(&mut self) {
        self.all_cycles.clear();
        self.all_cycle_durations.clear();
        self.all_phase_benchmarks.clear();
        self.total_cycles = 0;
        self.total_duration_ns = 0;
        self.avg_duration = Duration::ZERO;
        self.min_duration = Duration::MAX;
        self.max_duration = Duration::ZERO;
        self.sum_squared_diff_ns = 0.0;
    }

    /// 获取最近N个周期的详细报告
    pub fn recent_cycle_reports(&self, n: usize) -> Vec<NodeCycleReport> {
        self.all_cycles
            .iter()
            .rev()
            .take(n)
            .map(|tracker| tracker.get_cycle_report(self.node_id.clone(), self.node_name.clone()))
            .collect()
    }

    /// 获取最近一个周期的报告
    pub fn last_cycle_report(&self) -> Option<NodeCycleReport> {
        self.all_cycles
            .back()
            .map(|tracker| tracker.get_cycle_report(self.node_id.clone(), self.node_name.clone()))
    }

    /// 获取总周期数
    pub fn total_cycles(&self) -> usize {
        self.total_cycles
    }

    /// 根据 play_index 查找对应的周期报告
    pub fn cycle_report_by_cycle_id(&self, cycle_id: CycleId) -> Option<NodeCycleReport> {
        self.all_cycles
            .iter()
            .find(|tracker| tracker.get_cycle_id() == cycle_id)
            .map(|tracker| tracker.get_cycle_report(self.node_id.clone(), self.node_name.clone()))
    }
}

// ============================================================
// 第三部分：报告类型
// ============================================================

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodePerformanceReport {
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub total_cycles: usize,
    pub total_duration_ns: u128,
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
    pub avg_duration_percentage: f32,
    pub phase_reports: Vec<NodePhaseReport>,
}

impl std::fmt::Display for NodePerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─ Performance Report: {} [{}]", self.node_name, self.node_type)?;
        writeln!(f, "│  Node ID: {}", self.node_id)?;
        writeln!(f, "│  Total Cycles: {}", self.total_cycles)?;
        writeln!(f, "├─ Duration Statistics:")?;
        writeln!(f, "│  ├─ Average: {:?}", self.avg_duration)?;
        writeln!(f, "│  ├─ Median:  {:?}", self.p50)?;
        writeln!(f, "│  ├─ Min:     {:?}", self.min_duration)?;
        writeln!(f, "│  ├─ Max:     {:?}", self.max_duration)?;
        writeln!(f, "│  ├─ StdDev:  {:?}", self.std_deviation)?;
        writeln!(f, "│  ├─ P95:     {:?}", self.p95)?;
        writeln!(f, "│  ├─ P99:     {:?}", self.p99)?;
        writeln!(f, "│  └─ Recent(100): {:?}", self.recent_avg_100)?;

        if !self.phase_reports.is_empty() {
            writeln!(f, "├─ Phase Reports (top 5):")?;
            for (i, phase) in self.phase_reports.iter().take(5).enumerate() {
                let prefix = if i == self.phase_reports.len() - 1 { "└" } else { "├" };
                writeln!(f, "│  {} {}", prefix, phase)?;
            }
        }

        write!(f, "└─────────────────────────────")
    }
}
