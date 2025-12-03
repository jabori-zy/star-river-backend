#![allow(unused)]
use chrono::{DateTime, Duration, Utc};
use star_river_core::kline::KlineInterval;

pub struct SignalGenerator {
    pub current_index: u64,
    pub current_time: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub min_interval: KlineInterval,
    pub finished: bool,
    total_signal_count: u64, // 总信号数量
}

impl SignalGenerator {
    pub fn new() -> Self {
        Self {
            current_index: 0,
            current_time: DateTime::<Utc>::MIN_UTC,
            start_time: DateTime::<Utc>::MIN_UTC,
            end_time: DateTime::<Utc>::MIN_UTC,
            min_interval: KlineInterval::Minutes1,
            finished: false,
            total_signal_count: 0,
        }
    }

    pub fn init(&mut self, start_time: DateTime<Utc>, end_time: DateTime<Utc>, min_interval: KlineInterval) {
        // 先计算总信号数量（在移动 min_interval 之前）
        let total_signal_count = Self::calculate_total_signals(start_time, end_time, &min_interval);

        self.start_time = start_time;
        self.end_time = end_time;
        self.min_interval = min_interval;
        self.current_index = 0;
        self.current_time = start_time;
        self.finished = false;
        self.total_signal_count = total_signal_count;
    }

    pub fn next(&mut self) -> Option<(u64, DateTime<Utc>)> {
        // If already finished, return None
        if self.finished {
            return None;
        }

        // Check if current time has exceeded end time
        if self.current_time > self.end_time {
            self.finished = true;
            return None;
        }

        // Save current values to return
        let result = (self.current_index, self.current_time);

        // Check if this is the last item (current_time == end_time)
        if self.current_time == self.end_time {
            self.finished = true;
            return Some(result);
        }

        // Update state for next iteration
        self.current_index += 1;
        let duration = Duration::from_std(self.min_interval.to_duration()).unwrap();
        self.current_time = self.current_time + duration;

        Some(result)
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.current_time = self.start_time;
        self.finished = false;
    }

    /// 计算总信号数量
    ///
    /// # Arguments
    /// * `start_time` - 开始时间
    /// * `end_time` - 结束时间
    /// * `min_interval` - 最小时间间隔
    ///
    /// # Returns
    /// 总信号数量（包含起始和结束时间点）
    fn calculate_total_signals(start_time: DateTime<Utc>, end_time: DateTime<Utc>, min_interval: &KlineInterval) -> u64 {
        if start_time > end_time {
            return 0;
        }

        if start_time == end_time {
            return 1;
        }

        // 计算时间差
        let time_diff = end_time.signed_duration_since(start_time);
        let interval_duration = Duration::from_std(min_interval.to_duration()).unwrap();

        // 计算信号数量: (时间差 / 间隔) + 1
        // +1 是因为包含起始点
        let count = time_diff.num_milliseconds() / interval_duration.num_milliseconds();
        (count + 1) as u64
    }

    /// 获取总信号数量
    pub fn total_signal_count(&self) -> u64 {
        self.total_signal_count
    }

    /// 获取当前索引
    pub fn current_index(&self) -> u64 {
        self.current_index
    }

    /// 获取播放进度 (0.0 ~ 1.0)
    ///
    /// # Returns
    /// * `0.0` - 未开始
    /// * `0.0 ~ 1.0` - 播放中
    /// * `1.0` - 已完成
    pub fn progress(&self) -> f64 {
        if self.total_signal_count == 0 {
            return 0.0;
        }

        if self.finished {
            return 1.0;
        }

        // 当前索引 / 总数量
        // 注意: current_index 从 0 开始，所以进度计算需要调整
        // 例如: total=10, current_index=0 表示播放了第1个，进度应该是 1/10 = 0.1
        let current = self.current_index as f64 + 1.0;
        let total = self.total_signal_count as f64;

        (current / total).min(1.0)
    }

    /// 获取播放进度百分比 (0 ~ 100)
    pub fn progress_percentage(&self) -> f64 {
        self.progress() * 100.0
    }

    /// 获取剩余信号数量
    pub fn remaining_signals(&self) -> u64 {
        if self.finished {
            return 0;
        }

        // total_signal_count 是总数
        // current_index 是已经返回的最后一个索引
        // 剩余 = 总数 - (当前索引 + 1)
        self.total_signal_count.saturating_sub(self.current_index + 1)
    }
}

impl std::fmt::Debug for SignalGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SignalGenerator(current_index: {}, current_time: {}, start_time: {}, end_time: {}, min_interval: {}, finished: {}, total_signal_count: {}, progress: {})",
            self.current_index,
            self.current_time,
            self.start_time,
            self.end_time,
            self.min_interval,
            self.finished,
            self.total_signal_count,
            self.progress()
        )
    }
}
