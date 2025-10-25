use snafu::OptionExt;
use crate::{
    custom_type::{NodeId, PlayIndex, StrategyId}, 
    error::engine_error::strategy_error::backtest_strategy_error::*
};
use tokio::time::{Instant, Duration};
use std::collections::{HashMap, VecDeque};

use super::node_benchmark::{CompletedCycleTracker, NodeBenchmark};

// ============================================================
// 第一部分：StrategyCycleTracker - 策略单周期追踪器
// ============================================================

/// 策略单周期追踪器（支持 phase 追踪）
#[derive(Debug)]
pub struct StrategyCycleTracker {
    play_index: PlayIndex,
    start_time: Instant,
    phase_durations: Vec<(String, Duration)>, // 使用 String 支持动态内容
}

impl StrategyCycleTracker {
    pub fn new(play_index: PlayIndex) -> Self {
        Self {
            play_index,
            start_time: Instant::now(),
            phase_durations: Vec::new(),
        }
    }

    /// 开始追踪一个阶段
    #[inline]
    pub fn start_phase(&mut self, phase_name: impl Into<String>) {
        // 直接记录当前时间作为阶段开始
        self.phase_durations.push((
            phase_name.into(),
            Instant::now().duration_since(self.start_time)
        ));
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
    pub fn end(&self) -> CompletedStrategyCycleTracker {
        let total_duration = self.start_time.elapsed();
        CompletedStrategyCycleTracker {
            play_index: self.play_index,
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
pub struct CompletedStrategyCycleTracker {
    play_index: PlayIndex,
    total_duration: Duration,
    phase_durations: Vec<(String, Duration)>,
}

impl CompletedStrategyCycleTracker {
    pub fn get_total_duration(&self) -> Duration {
        self.total_duration
    }

    pub fn get_play_index(&self) -> PlayIndex {
        self.play_index
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

    /// 生成单个周期的详细报告
    pub fn get_cycle_report(&self) -> StrategyCycleReport {
        StrategyCycleReport {
            play_index: self.play_index,
            total_duration: self.total_duration,
            phase_durations: self.phase_durations.clone(),
        }
    }
}

// ============================================================
// 第三部分：StrategyCycleReport - 单周期报告
// ============================================================

/// 策略单个周期的详细报告
#[derive(Debug, Clone)]
pub struct StrategyCycleReport {
    pub play_index: PlayIndex,
    pub total_duration: Duration,
    pub phase_durations: Vec<(String, Duration)>,
}

impl StrategyCycleReport {
    pub fn get_total_duration(&self) -> Duration {
        self.total_duration
    }

    pub fn get_play_index(&self) -> PlayIndex {
        self.play_index
    }

    /// 获取某个阶段的耗时占比
    pub fn get_phase_percentage(&self, phase_name: &str) -> f64 {
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
    pub fn get_slowest_phase(&self) -> Option<(&String, Duration)> {
        self.phase_durations
            .iter()
            .max_by_key(|(_, duration)| duration)
            .map(|(name, duration)| (name, *duration))
    }
}

impl std::fmt::Display for StrategyCycleReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─ Strategy Cycle Report [Play Index: {}]", self.play_index)?;
        writeln!(f, "│  Total Duration: {:?}", self.total_duration)?;
        
        if !self.phase_durations.is_empty() {
            writeln!(f, "├─ Phase Details:")?;
            
            for (i, (phase_name, duration)) in self.phase_durations.iter().enumerate() {
                let percentage = if !self.total_duration.is_zero() {
                    (duration.as_nanos() as f64 / self.total_duration.as_nanos() as f64) * 100.0
                } else {
                    0.0
                };
                
                let prefix = if i == self.phase_durations.len() - 1 { "└" } else { "├" };
                writeln!(
                    f,
                    "│  {} {}: {:?} ({:.2}%)",
                    prefix, phase_name, duration, percentage
                )?;
            }
        }
        
        if let Some((slowest_name, slowest_duration)) = self.get_slowest_phase() {
            writeln!(f, "├─ Slowest Phase: {} ({:?})", slowest_name, slowest_duration)?;
        }
        
        write!(f, "└─────────────────────────────")
    }
}

// ============================================================
// 详细策略周期报告（包含节点信息）
// ============================================================

/// 单个节点在该周期的性能详情
#[derive(Debug, Clone)]
pub struct NodeCycleDetail {
    pub node_id: NodeId,
    pub node_name: String,
    pub node_type: String,
    pub total_duration: Duration,
    pub phase_durations: Vec<(String, Duration)>,
    pub percentage: f64, // 占策略总时间的百分比
}

impl std::fmt::Display for NodeCycleDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "    ├─ Node: {} [{}] (ID: {})", self.node_name, self.node_type, self.node_id)?;
        writeln!(f, "    │  Total Duration: {:?} ({:.2}%)", self.total_duration, self.percentage)?;
        
        if !self.phase_durations.is_empty() {
            writeln!(f, "    │  Phase Details:")?;
            for (i, (phase_name, phase_duration)) in self.phase_durations.iter().enumerate() {
                let phase_percentage = if !self.total_duration.is_zero() {
                    (phase_duration.as_nanos() as f64 / self.total_duration.as_nanos() as f64) * 100.0
                } else {
                    0.0
                };
                
                let prefix = if i == self.phase_durations.len() - 1 { "└" } else { "├" };
                writeln!(
                    f,
                    "    │    {} {}: {:?} ({:.2}%)",
                    prefix, phase_name, phase_duration, phase_percentage
                )?;
            }
        }
        
        Ok(())
    }
}

/// 详细的策略周期报告（包含所有节点的详细信息）
#[derive(Debug, Clone)]
pub struct DetailedStrategyCycleReport {
    pub play_index: PlayIndex,
    pub strategy_total_duration: Duration,
    pub strategy_phase_durations: Vec<(String, Duration)>, // 策略级别的阶段耗时
    pub node_details: Vec<NodeCycleDetail>,
}

impl std::fmt::Display for DetailedStrategyCycleReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "╔═══════════════════════════════════════════════════════════╗")?;
        writeln!(f, "║ Detailed Strategy Cycle Report [Play Index: {}]", self.play_index)?;
        writeln!(f, "╠═══════════════════════════════════════════════════════════╣")?;
        writeln!(f, "║ Strategy Total Duration: {:?}", self.strategy_total_duration)?;
        
        // 显示策略级别的阶段信息
        if !self.strategy_phase_durations.is_empty() {
            writeln!(f, "╟───────────────────────────────────────────────────────────╢")?;
            writeln!(f, "║ Strategy Phase Breakdown:")?;
            for (phase_name, phase_duration) in &self.strategy_phase_durations {
                let phase_percentage = if !self.strategy_total_duration.is_zero() {
                    (phase_duration.as_nanos() as f64 / self.strategy_total_duration.as_nanos() as f64) * 100.0
                } else {
                    0.0
                };
                writeln!(
                    f,
                    "║   • {}: {:?} ({:.2}%)",
                    phase_name, phase_duration, phase_percentage
                )?;
            }
        }
        
        writeln!(f, "╟───────────────────────────────────────────────────────────╢")?;
        writeln!(f, "║ Node Performance Breakdown:")?;
        
        if self.node_details.is_empty() {
            writeln!(f, "║   (No node data available)")?;
        } else {
            // 按执行时间占比降序排序
            let mut sorted_nodes = self.node_details.clone();
            sorted_nodes.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap_or(std::cmp::Ordering::Equal));
            
            for (i, node_detail) in sorted_nodes.iter().enumerate() {
                writeln!(f, "║")?;
                writeln!(f, "║  {}. {} [{}]", i + 1, node_detail.node_name, node_detail.node_type)?;
                writeln!(f, "║     Duration: {:?} ({:.2}%)", node_detail.total_duration, node_detail.percentage)?;
                
                if !node_detail.phase_durations.is_empty() {
                    writeln!(f, "║     Phases:")?;
                    for (phase_name, phase_duration) in &node_detail.phase_durations {
                        let phase_percentage = if !node_detail.total_duration.is_zero() {
                            (phase_duration.as_nanos() as f64 / node_detail.total_duration.as_nanos() as f64) * 100.0
                        } else {
                            0.0
                        };
                        writeln!(
                            f,
                            "║       • {}: {:?} ({:.2}% of node)",
                            phase_name, phase_duration, phase_percentage
                        )?;
                    }
                }
            }
        }
        
        writeln!(f, "╟───────────────────────────────────────────────────────────╢")?;
        
        // 统计信息
        let total_node_time: Duration = self.node_details.iter().map(|n| n.total_duration).sum();
        let coverage_percentage = if !self.strategy_total_duration.is_zero() {
            (total_node_time.as_nanos() as f64 / self.strategy_total_duration.as_nanos() as f64) * 100.0
        } else {
            0.0
        };
        
        writeln!(f, "║ Summary:")?;
        writeln!(f, "║   Total Node Time: {:?}", total_node_time)?;
        writeln!(f, "║   Coverage: {:.2}%", coverage_percentage)?;
        write!(f, "╚═══════════════════════════════════════════════════════════╝")
    }
}

// ============================================================
// 第四部分：StrategyBenchmark - 策略性能统计
// ============================================================

/// 策略性能统计（聚合多个周期的数据）
#[derive(Debug, Clone)]
pub struct StrategyBenchmark {
    pub strategy_id: StrategyId,
    pub strategy_name: String,
    
    // 使用 VecDeque 作为环形缓冲区，限制内存使用
    cycle_trackers: VecDeque<CompletedStrategyCycleTracker>,
    node_benchmarks: HashMap<NodeId, NodeBenchmark>,
    max_history: usize,
    
    // 实时统计数据
    total_cycles: usize,
    total_duration_ns: u128,
    min_duration: Duration,
    max_duration: Duration,
    
    // 用于计算标准差的在线统计
    sum_squared_diff_ns: f64,
}

impl StrategyBenchmark {
    pub fn new(strategy_id: StrategyId, strategy_name: String) -> Self {
        Self {
            strategy_id,
            strategy_name,
            cycle_trackers: VecDeque::with_capacity(1000),
            node_benchmarks: HashMap::new(),
            max_history: 1000,
            total_cycles: 0,
            total_duration_ns: 0,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            sum_squared_diff_ns: 0.0,
        }
    }


    pub fn add_node_benchmark(&mut self, node_id: NodeId, node_name: String, node_type: String) {
        let node_benchmark = NodeBenchmark::new(node_type, node_id.clone(), node_name);
        self.node_benchmarks.insert(node_id, node_benchmark);
    }


    pub fn add_node_cycle_tracker(&mut self, node_id: NodeId, cycle_tracker: CompletedCycleTracker) -> Result<(), BacktestStrategyError> {

        self.node_benchmarks
            .get_mut(&node_id)
            .context(NodeBenchmarkNotFoundSnafu { node_id: node_id.clone() })?
            .add_cycle_tracker(cycle_tracker);
        Ok(())
    }

    /// 添加一个完成的周期追踪器
    pub fn add_cycle_tracker(&mut self, cycle_tracker: CompletedStrategyCycleTracker) {
        let duration = cycle_tracker.get_total_duration();
        let duration_ns = duration.as_nanos();
        
        // 更新统计数据
        self.total_cycles += 1;
        self.total_duration_ns += duration_ns;
        
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
        
        // 保存详细数据（使用环形缓冲区）
        if self.cycle_trackers.len() >= self.max_history {
            self.cycle_trackers.pop_front();
        }
        self.cycle_trackers.push_back(cycle_tracker);
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
        if self.cycle_trackers.is_empty() {
            return Duration::ZERO;
        }
        
        let mut durations: Vec<Duration> = self.cycle_trackers
            .iter()
            .map(|t| t.get_total_duration())
            .collect();
        
        durations.sort_unstable();
        durations[durations.len() / 2]
    }

    /// 获取百分位数
    pub fn percentile(&self, p: f64) -> Duration {
        if self.cycle_trackers.is_empty() || !(0.0..=1.0).contains(&p) {
            return Duration::ZERO;
        }
        
        let mut durations: Vec<Duration> = self.cycle_trackers
            .iter()
            .map(|t| t.get_total_duration())
            .collect();
        
        durations.sort_unstable();
        let index = ((durations.len() as f64 - 1.0) * p) as usize;
        durations[index]
    }

    /// 获取最近N个周期的平均值
    pub fn recent_avg_duration(&self, n: usize) -> Duration {
        if self.cycle_trackers.is_empty() {
            return Duration::ZERO;
        }
        
        let count = n.min(self.cycle_trackers.len());
        let sum: Duration = self.cycle_trackers
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
        let overall_avg = self.avg_duration();
        
        if overall_avg.is_zero() {
            return false;
        }
        
        let ratio = recent_avg.as_nanos() as f64 / overall_avg.as_nanos() as f64;
        ratio > (1.0 + threshold)
    }

    /// 分析策略级别的阶段性能瓶颈
    pub fn analyze_phase_bottlenecks(&self) -> Vec<StrategyPhaseStatistics> {
        if self.cycle_trackers.is_empty() {
            return Vec::new();
        }
        
        // 收集所有阶段的统计数据
        let mut phase_stats: HashMap<String, Vec<Duration>> = HashMap::new();
        
        for tracker in &self.cycle_trackers {
            for (phase_name, duration) in tracker.get_all_phase_durations() {
                phase_stats
                    .entry(phase_name.clone())
                    .or_insert_with(Vec::new)
                    .push(*duration);
            }
        }
        
        // 计算每个阶段的统计信息
        let mut results: Vec<StrategyPhaseStatistics> = phase_stats
            .into_iter()
            .map(|(name, durations)| {
                let count = durations.len();
                let total: Duration = durations.iter().sum();
                let avg = total / count as u32;
                let max = *durations.iter().max().unwrap();
                let min = *durations.iter().min().unwrap();
                
                StrategyPhaseStatistics {
                    phase_name: name,
                    count,
                    avg_duration: avg,
                    min_duration: min,
                    max_duration: max,
                    total_duration: total,
                }
            })
            .collect();
        
        // 按平均耗时降序排序
        results.sort_by(|a, b| b.avg_duration.cmp(&a.avg_duration));
        results
    }

    /// 生成性能报告
    pub fn report(&self) -> StrategyPerformanceReport {
        StrategyPerformanceReport {
            strategy_id: self.strategy_id,
            strategy_name: self.strategy_name.clone(),
            total_cycles: self.total_cycles,
            avg_duration: self.avg_duration(),
            min_duration: self.min_duration,
            max_duration: self.max_duration,
            median_duration: self.median_duration(),
            std_deviation: self.std_deviation(),
            p95: self.percentile(0.95),
            p99: self.percentile(0.99),
            recent_avg_100: self.recent_avg_duration(100),
            phase_bottlenecks: self.analyze_phase_bottlenecks(),
        }
    }

    /// 重置统计数据
    pub fn reset(&mut self) {
        self.cycle_trackers.clear();
        self.total_cycles = 0;
        self.total_duration_ns = 0;
        self.min_duration = Duration::MAX;
        self.max_duration = Duration::ZERO;
        self.sum_squared_diff_ns = 0.0;
    }

    /// 获取最近N个周期的详细报告
    pub fn get_recent_cycle_reports(&self, n: usize) -> Vec<StrategyCycleReport> {
        self.cycle_trackers
            .iter()
            .rev()
            .take(n)
            .map(|tracker| tracker.get_cycle_report())
            .collect()
    }

    /// 获取最近一个周期的报告（简单版本，仅策略级别）
    pub fn get_last_cycle_report(&self) -> Option<StrategyCycleReport> {
        self.cycle_trackers
            .back()
            .map(|tracker| tracker.get_cycle_report())
    }

    
    pub fn get_last_detailed_cycle_report(&self) -> Option<DetailedStrategyCycleReport> {
        // 获取策略的最后一个周期
        let strategy_cycle = self.cycle_trackers.back()?;
        let strategy_total_duration = strategy_cycle.get_total_duration();
        let play_index = strategy_cycle.get_play_index();
        let strategy_phase_durations = strategy_cycle.get_all_phase_durations().to_vec();
        
        // 收集所有节点的最后一个周期数据
        let mut node_details = Vec::new();
        
        for (node_id, node_benchmark) in &self.node_benchmarks {
            if let Some(node_cycle_report) = node_benchmark.get_last_cycle_report() {
                let node_total_duration = node_cycle_report.total_duration;
                
                // 计算该节点占策略总时间的百分比
                let percentage = if !strategy_total_duration.is_zero() {
                    (node_total_duration.as_nanos() as f64 / strategy_total_duration.as_nanos() as f64) * 100.0
                } else {
                    0.0
                };
                
                node_details.push(NodeCycleDetail {
                    node_id: node_id.clone(),
                    node_name: node_benchmark.node_name.clone(),
                    node_type: node_benchmark.node_type.clone(),
                    total_duration: node_total_duration,
                    phase_durations: node_cycle_report.phase_durations.clone(),
                    percentage,
                });
            }
        }
        
        Some(DetailedStrategyCycleReport {
            play_index,
            strategy_total_duration,
            strategy_phase_durations,
            node_details,
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
#[derive(Debug, Clone)]
pub struct StrategyPhaseStatistics {
    pub phase_name: String,
    pub count: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub total_duration: Duration,
}

impl std::fmt::Display for StrategyPhaseStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "  ├─ {}: avg={:?}, min={:?}, max={:?}, count={}",
            self.phase_name,
            self.avg_duration,
            self.min_duration,
            self.max_duration,
            self.count
        )
    }
}

/// 策略整体性能报告
#[derive(Debug, Clone)]
pub struct StrategyPerformanceReport {
    pub strategy_id: StrategyId,
    pub strategy_name: String,
    pub total_cycles: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub median_duration: Duration,
    pub std_deviation: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub recent_avg_100: Duration,
    pub phase_bottlenecks: Vec<StrategyPhaseStatistics>,
}

impl std::fmt::Display for StrategyPerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n")?;
        writeln!(f, "╔═══════════════════════════════════════════════════════════╗")?;
        writeln!(f, "║ Strategy Performance Report")?;
        writeln!(f, "╠═══════════════════════════════════════════════════════════╣")?;
        writeln!(f, "║ Strategy: {} [ID: {}]", self.strategy_name, self.strategy_id)?;
        writeln!(f, "║ Total Cycles: {}", self.total_cycles)?;
        writeln!(f, "╟───────────────────────────────────────────────────────────╢")?;
        writeln!(f, "║ Duration Statistics:")?;
        writeln!(f, "║  ├─ Average:      {:>12?}", self.avg_duration)?;
        writeln!(f, "║  ├─ Median:       {:>12?}", self.median_duration)?;
        writeln!(f, "║  ├─ Min:          {:>12?}", self.min_duration)?;
        writeln!(f, "║  ├─ Max:          {:>12?}", self.max_duration)?;
        writeln!(f, "║  ├─ StdDev:       {:>12?}", self.std_deviation)?;
        writeln!(f, "║  ├─ P95:          {:>12?}", self.p95)?;
        writeln!(f, "║  ├─ P99:          {:>12?}", self.p99)?;
        writeln!(f, "║  └─ Recent(100):  {:>12?}", self.recent_avg_100)?;
        
        if !self.phase_bottlenecks.is_empty() {
            writeln!(f, "╟───────────────────────────────────────────────────────────╢")?;
            writeln!(f, "║ Phase Bottlenecks (top 5):")?;
            for (i, phase) in self.phase_bottlenecks.iter().take(5).enumerate() {
                let prefix = if i == self.phase_bottlenecks.len().min(5) - 1 { "└" } else { "├" };
                writeln!(f, "║  {} {}: avg={:?}, count={}", prefix, phase.phase_name, phase.avg_duration, phase.count)?;
            }
        }
        
        write!(f, "╚═══════════════════════════════════════════════════════════╝")
    }
}