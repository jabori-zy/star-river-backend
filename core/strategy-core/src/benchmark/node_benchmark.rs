use std::collections::{BTreeMap, HashMap, VecDeque};

use serde::Serialize;
use star_river_core::custom_type::{CycleId, NodeId};
use tokio::time::{Duration, Instant};
use utoipa::ToSchema;

// ============================================================
// Part 1: CycleTracker - Single Cycle Tracker
// ============================================================

#[derive(Debug, Clone)]
pub struct CycleTracker {
    cycle_id: CycleId,
    start_time: Instant,
    phase_durations: Vec<(String, Duration)>, // Use String to support dynamic content
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
        // Record current time as phase start
        self.phase_durations
            .push((phase_name.into(), Instant::now().duration_since(self.start_time)));
    }

    #[inline]
    pub fn end_phase(&mut self, phase_name: impl AsRef<str>) {
        let phase_name = phase_name.as_ref();
        // Find corresponding start time and calculate duration
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

/// Completed cycle (immutable)
#[derive(Debug, Clone)]
pub struct CompletedCycle {
    cycle_id: CycleId,
    duration: Duration,                       // Total duration of single cycle
    phase_durations: Vec<(String, Duration)>, // Duration of each phase
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

    /// Generate single cycle report
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

/// Detailed report for single cycle
#[derive(Debug, Clone)]
pub struct NodeCycleReport {
    pub node_id: NodeId,
    pub node_name: String,
    pub cycle_id: CycleId,
    pub duration: Duration,
    pub phase_durations: Vec<(String, Duration)>,
}

impl NodeCycleReport {
    /// Get percentage of phase duration
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

    /// Get slowest phase
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
// Part 2: NodeBenchmark - Node Performance Statistics
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
    pub all_phase_durations: BTreeMap<u64, usize>, // Store nanosecond values (key: nanoseconds, value: cycle count)
}

impl NodePhaseBenchmark {
    /// Create new phase performance benchmark
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

    /// Add phase duration sample
    pub fn add_duration(&mut self, duration: Duration) {
        let duration_ns = duration.as_nanos();

        // Update count
        self.total_cycles += 1;
        self.total_duration += duration;
        self.total_duration_ns += duration_ns;

        // Update min/max values
        if duration < self.min_duration {
            self.min_duration = duration;
        }
        if duration > self.max_duration {
            self.max_duration = duration;
        }

        // Update average
        self.avg_duration = Duration::from_nanos((self.total_duration_ns / self.total_cycles as u128) as u64);

        // Update standard deviation calculation (online algorithm)
        let avg_ns = self.total_duration_ns as f64 / self.total_cycles as f64;
        let diff = duration_ns as f64 - avg_ns;
        self.sum_squared_diff_ns += diff * diff;

        // Insert into BTreeMap (store nanosecond values)
        let duration_ns_u64 = duration_ns as u64;
        *self.all_phase_durations.entry(duration_ns_u64).or_insert(0) += 1;
    }

    /// Get standard deviation
    pub fn std_deviation(&self) -> Duration {
        if self.total_cycles <= 1 {
            return Duration::ZERO;
        }
        let variance = self.sum_squared_diff_ns / (self.total_cycles - 1) as f64;
        Duration::from_nanos(variance.sqrt() as u64)
    }

    /// Get percentile (based on BTreeMap, O(log n) complexity)
    pub fn percentile(&self, p: f64) -> Duration {
        if self.all_phase_durations.is_empty() || !(0.0..=1.0).contains(&p) {
            return Duration::ZERO;
        }

        // Calculate target index (based on total sample count)
        let target_index = ((self.total_cycles as f64 - 1.0) * p) as usize;

        // Traverse BTreeMap (sorted), accumulate count until reaching target index
        let mut accumulated = 0;
        for (duration_ns, count) in &self.all_phase_durations {
            accumulated += count;
            if accumulated > target_index {
                return Duration::from_nanos(*duration_ns);
            }
        }

        // If not found (theoretically should not happen), return max value
        self.all_phase_durations
            .keys()
            .last()
            .copied()
            .map(Duration::from_nanos)
            .unwrap_or(Duration::ZERO)
    }

    /// Generate phase report
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
    pub avg_duration_percentage: f32, // Average duration percentage
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

    // Use VecDeque as ring buffer to limit memory usage
    all_cycles: VecDeque<CompletedCycle>,
    all_cycle_durations: BTreeMap<u64, usize>, // Store nanosecond values (key: nanoseconds, value: cycle count with this duration)
    all_phase_benchmarks: HashMap<String, NodePhaseBenchmark>, // All phase durations (key: phase name, value: phase performance stats)

    max_history: usize, // Maximum number of historical cycles to keep
    // Real-time statistics
    total_cycles: usize,
    total_duration_ns: u128, // Use nanosecond accumulation to avoid precision loss
    avg_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,

    // Online statistics for calculating standard deviation
    sum_squared_diff_ns: f64, // Sum of squared differences
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
            max_history: 1000, // Keep only recent 1000 cycles of detailed data
            total_cycles: 0,
            total_duration_ns: 0,
            avg_duration: Duration::ZERO,
            min_duration: Duration::MAX, // Initialize to max value
            max_duration: Duration::ZERO,
            sum_squared_diff_ns: 0.0,
        }
    }

    /// Add completed cycle tracker
    pub fn add_completed_cycle(&mut self, completed_cycle: CompletedCycle) {
        let duration = completed_cycle.get_duration();
        let duration_ns = duration.as_nanos();

        // Update statistics
        self.total_cycles += 1;
        self.total_duration_ns += duration_ns;

        // Update average duration
        self.avg_duration = Duration::from_nanos((self.total_duration_ns / self.total_cycles as u128) as u64);

        // Update min/max values
        if duration < self.min_duration {
            self.min_duration = duration;
        }
        if duration > self.max_duration {
            self.max_duration = duration;
        }

        // Update standard deviation calculation (online algorithm)
        let avg_ns = self.total_duration_ns as f64 / self.total_cycles as f64;
        let diff = duration_ns as f64 - avg_ns;
        self.sum_squared_diff_ns += diff * diff;

        // Insert into all_cycle_durations (BTreeMap) - store nanosecond values
        let duration_ns_u64 = duration_ns as u64;
        *self.all_cycle_durations.entry(duration_ns_u64).or_insert(0) += 1;

        // Update all_phase_benchmarks (HashMap<String, NodePhaseBenchmark>)
        for (phase_name, phase_duration) in completed_cycle.get_all_phase_durations() {
            self.all_phase_benchmarks
                .entry(phase_name.clone())
                .or_insert_with(|| NodePhaseBenchmark::new(phase_name.clone()))
                .add_duration(*phase_duration);
        }

        // Save detailed data (using ring buffer)
        if self.all_cycles.len() >= self.max_history {
            self.all_cycles.pop_front(); // Remove oldest data
        }
        self.all_cycles.push_back(completed_cycle);
    }

    /// Get average execution time
    pub fn avg_duration(&self) -> Duration {
        if self.total_cycles == 0 {
            return Duration::ZERO;
        }
        Duration::from_nanos((self.total_duration_ns / self.total_cycles as u128) as u64)
    }

    /// Get standard deviation
    pub fn std_deviation(&self) -> Duration {
        if self.total_cycles <= 1 {
            return Duration::ZERO;
        }
        let variance = self.sum_squared_diff_ns / (self.total_cycles - 1) as f64;
        Duration::from_nanos(variance.sqrt() as u64)
    }

    /// Get percentile (based on BTreeMap, O(log n) complexity)
    pub fn percentile(&self, p: f64) -> Duration {
        if self.all_cycle_durations.is_empty() || !(0.0..=1.0).contains(&p) {
            return Duration::ZERO;
        }

        // Calculate target index (based on total sample count)
        let target_index = ((self.total_cycles as f64 - 1.0) * p) as usize;

        // Traverse BTreeMap (sorted), accumulate count until reaching target index
        let mut accumulated = 0;
        for (duration_ns, count) in &self.all_cycle_durations {
            accumulated += count;
            if accumulated > target_index {
                return Duration::from_nanos(*duration_ns);
            }
        }

        // If not found (theoretically should not happen), return max value
        self.all_cycle_durations
            .keys()
            .last()
            .copied()
            .map(Duration::from_nanos)
            .unwrap_or(Duration::ZERO)
    }

    /// Get average of recent N cycles
    pub fn recent_avg_duration(&self, n: usize) -> Duration {
        if self.all_cycles.is_empty() {
            return Duration::ZERO;
        }

        let count = n.min(self.all_cycles.len());
        let sum: Duration = self.all_cycles.iter().rev().take(count).map(|t| t.get_duration()).sum();

        sum / count as u32
    }

    /// Detect performance degradation (recent N cycles vs overall average)
    pub fn detect_performance_degradation(&self, recent_count: usize, threshold: f64) -> bool {
        if self.total_cycles < recent_count * 2 {
            return false; // Insufficient data
        }

        let recent_avg = self.recent_avg_duration(recent_count);
        let overall_avg = self.avg_duration;

        if overall_avg.is_zero() {
            return false;
        }

        let ratio = recent_avg.as_nanos() as f64 / overall_avg.as_nanos() as f64;
        ratio > (1.0 + threshold) // If recent average exceeds overall by threshold, consider it degraded
    }

    // Get phase report
    pub fn get_phase_report(&self) -> Vec<NodePhaseReport> {
        if self.all_phase_benchmarks.is_empty() {
            return Vec::new();
        }

        // Calculate duration percentage for each phase, then generate report
        let mut results: Vec<NodePhaseReport> = self
            .all_phase_benchmarks
            .values()
            .map(|benchmark| benchmark.report(self.avg_duration))
            .collect();

        // Sort by average duration in descending order
        results.sort_by(|a, b| b.avg_duration.cmp(&a.avg_duration));
        results
    }

    /// Generate performance report
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

    /// Reset statistics
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

    /// Get detailed reports for recent N cycles
    pub fn recent_cycle_reports(&self, n: usize) -> Vec<NodeCycleReport> {
        self.all_cycles
            .iter()
            .rev()
            .take(n)
            .map(|tracker| tracker.get_cycle_report(self.node_id.clone(), self.node_name.clone()))
            .collect()
    }

    /// Get last cycle report
    pub fn last_cycle_report(&self) -> Option<NodeCycleReport> {
        self.all_cycles
            .back()
            .map(|tracker| tracker.get_cycle_report(self.node_id.clone(), self.node_name.clone()))
    }

    /// Get total cycle count
    pub fn total_cycles(&self) -> usize {
        self.total_cycles
    }

    /// Find cycle report by cycle_id
    pub fn cycle_report_by_cycle_id(&self, cycle_id: CycleId) -> Option<NodeCycleReport> {
        self.all_cycles
            .iter()
            .find(|tracker| tracker.get_cycle_id() == cycle_id)
            .map(|tracker| tracker.get_cycle_report(self.node_id.clone(), self.node_name.clone()))
    }
}

// ============================================================
// Part 3: Report Types
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
