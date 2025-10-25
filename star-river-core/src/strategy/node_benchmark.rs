use crate::custom_type::{NodeId, PlayIndex};
use tokio::time::{Instant, Duration};
use std::collections::{HashMap, VecDeque};
use serde::Serialize;
use utoipa::ToSchema;

// ============================================================
// 第一部分：CycleTracker - 单周期追踪器
// ============================================================

#[derive(Debug, Clone)]
pub struct CycleTracker {
    play_index: PlayIndex,
    start_time: Instant,
    phase_durations: Vec<(String, Duration)>, // 使用 String 支持动态内容
}

impl CycleTracker {
    pub fn new(play_index: PlayIndex) -> Self {
        Self {
            play_index,
            start_time: Instant::now(),
            phase_durations: Vec::new(),
        }
    }

    #[inline]
    pub fn start_phase(&mut self, phase_name: impl Into<String>) {
        // 直接记录当前时间作为阶段开始
        self.phase_durations.push((
            phase_name.into(),
            Instant::now().duration_since(self.start_time)
        ));
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
    pub fn end(self) -> CompletedCycleTracker {
        let total_duration = self.start_time.elapsed();
        CompletedCycleTracker {
            play_index: self.play_index,
            total_duration,
            phase_durations: self.phase_durations,
        }
    }
}

/// 已完成的周期追踪器（不可变）
#[derive(Debug, Clone)]
pub struct CompletedCycleTracker {
    play_index: PlayIndex,
    total_duration: Duration,
    phase_durations: Vec<(String, Duration)>,
}

impl CompletedCycleTracker {
    pub fn get_total_duration(&self) -> Duration {
        self.total_duration
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

    pub fn get_play_index(&self) -> PlayIndex {
        self.play_index
    }

    /// 生成单个周期的详细报告
    pub fn get_cycle_report(&self, node_id: NodeId, node_name: String) -> NodeCycleReport {
        NodeCycleReport {
            node_id,
            node_name,
            play_index: self.play_index,
            total_duration: self.total_duration,
            phase_durations: self.phase_durations.clone(),
        }
    }
}

/// 单个周期的详细报告
#[derive(Debug, Clone)]
pub struct NodeCycleReport {
    pub node_id: NodeId,
    pub node_name: String,
    pub play_index: PlayIndex,
    pub total_duration: Duration,
    pub phase_durations: Vec<(String, Duration)>,
}

impl NodeCycleReport {
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

impl std::fmt::Display for NodeCycleReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─ Cycle Report [Play Index: {}]", self.play_index)?;
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
// 第二部分：NodeBenchmark - 节点性能统计
// ============================================================

#[derive(Debug, Clone)]
pub struct NodeBenchmark {
    pub node_id: NodeId,
    pub node_name: String,
    pub node_type: String,
    
    // 使用 VecDeque 作为环形缓冲区，限制内存使用
    cycle_trackers: VecDeque<CompletedCycleTracker>,
    max_history: usize, // 最多保留多少个历史周期
    
    // 实时统计数据
    total_cycles: usize,
    total_duration_ns: u128, // 使用纳秒累加，避免精度损失
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
            cycle_trackers: VecDeque::with_capacity(1000),
            max_history: 1000, // 只保留最近1000个周期的详细数据
            total_cycles: 0,
            total_duration_ns: 0,
            min_duration: Duration::MAX, // ✅ 修复：初始化为最大值
            max_duration: Duration::ZERO,
            sum_squared_diff_ns: 0.0,
        }
    }

    /// 添加一个完成的周期追踪器
    pub fn add_cycle_tracker(&mut self, cycle_tracker: CompletedCycleTracker) {
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
            self.cycle_trackers.pop_front(); // 移除最旧的数据
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

    /// 获取中位数（只从保留的历史数据中计算）
    pub fn median_duration(&self) -> Duration {
        if self.cycle_trackers.is_empty() {
            return Duration::ZERO;
        }
        
        let mut durations: Vec<Duration> = self.cycle_trackers
            .iter()
            .map(|t| t.get_total_duration())
            .collect();
        
        durations.sort_unstable(); // 使用更快的不稳定排序
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

    /// 检测性能是否退化（最近N个周期 vs 总体平均）
    pub fn detect_performance_degradation(&self, recent_count: usize, threshold: f64) -> bool {
        if self.total_cycles < recent_count * 2 {
            return false; // 数据不足
        }
        
        let recent_avg = self.recent_avg_duration(recent_count);
        let overall_avg = self.avg_duration();
        
        if overall_avg.is_zero() {
            return false;
        }
        
        let ratio = recent_avg.as_nanos() as f64 / overall_avg.as_nanos() as f64;
        ratio > (1.0 + threshold) // 如果最近的平均值比总体高出 threshold 倍，则认为性能退化
    }

    /// 分析阶段性能瓶颈
    pub fn analyze_phase_bottlenecks(&self) -> Vec<PhaseStatistics> {
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
        let mut results: Vec<PhaseStatistics> = phase_stats
            .into_iter()
            .map(|(name, durations)| {
                let count = durations.len();
                let total: Duration = durations.iter().sum();
                let avg = total / count as u32;
                let max = *durations.iter().max().unwrap();
                let min = *durations.iter().min().unwrap();
                
                PhaseStatistics {
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
    pub fn report(&self) -> NodePerformanceReport {
        NodePerformanceReport {
            node_id: self.node_id.clone(),
            node_name: self.node_name.clone(),
            node_type: self.node_type.clone(),
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
    pub fn get_recent_cycle_reports(&self, n: usize) -> Vec<NodeCycleReport> {
        self.cycle_trackers
            .iter()
            .rev()
            .take(n)
            .map(|tracker| tracker.get_cycle_report(self.node_id.clone(), self.node_name.clone()))
            .collect()
    }

    /// 获取最近一个周期的报告
    pub fn get_last_cycle_report(&self) -> Option<NodeCycleReport> {
        self.cycle_trackers
            .back()
            .map(|tracker| tracker.get_cycle_report(self.node_id.clone(), self.node_name.clone()))
    }

    /// 获取总周期数
    pub fn get_total_cycles(&self) -> usize {
        self.total_cycles
    }

    /// 根据 play_index 查找对应的周期报告
    pub fn get_cycle_report_by_play_index(&self, play_index: PlayIndex) -> Option<NodeCycleReport> {
        self.cycle_trackers
            .iter()
            .find(|tracker| tracker.get_play_index() == play_index)
            .map(|tracker| tracker.get_cycle_report(self.node_id.clone(), self.node_name.clone()))
    }
}

// ============================================================
// 第三部分：报告类型
// ============================================================

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PhaseStatistics {
    pub phase_name: String,
    pub count: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub total_duration: Duration,
}

impl std::fmt::Display for PhaseStatistics {
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

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct NodePerformanceReport {
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub total_cycles: usize,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub median_duration: Duration,
    pub std_deviation: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub recent_avg_100: Duration,
    pub phase_bottlenecks: Vec<PhaseStatistics>,
}

impl std::fmt::Display for NodePerformanceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─ Performance Report: {} [{}]", self.node_name, self.node_type)?;
        writeln!(f, "│  Node ID: {}", self.node_id)?;
        writeln!(f, "│  Total Cycles: {}", self.total_cycles)?;
        writeln!(f, "├─ Duration Statistics:")?;
        writeln!(f, "│  ├─ Average: {:?}", self.avg_duration)?;
        writeln!(f, "│  ├─ Median:  {:?}", self.median_duration)?;
        writeln!(f, "│  ├─ Min:     {:?}", self.min_duration)?;
        writeln!(f, "│  ├─ Max:     {:?}", self.max_duration)?;
        writeln!(f, "│  ├─ StdDev:  {:?}", self.std_deviation)?;
        writeln!(f, "│  ├─ P95:     {:?}", self.p95)?;
        writeln!(f, "│  ├─ P99:     {:?}", self.p99)?;
        writeln!(f, "│  └─ Recent(100): {:?}", self.recent_avg_100)?;
        
        if !self.phase_bottlenecks.is_empty() {
            writeln!(f, "├─ Phase Bottlenecks (top 5):")?;
            for (i, phase) in self.phase_bottlenecks.iter().take(5).enumerate() {
                let prefix = if i == self.phase_bottlenecks.len() - 1 { "└" } else { "├" };
                writeln!(f, "│  {} {}", prefix, phase)?;
            }
        }
        
        write!(f, "└─────────────────────────────")
    }
}