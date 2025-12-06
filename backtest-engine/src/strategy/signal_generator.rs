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
    total_signal_count: u64, // Total signal count
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
        // Calculate total signal count first (before moving min_interval)
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

    /// Calculate total signal count
    ///
    /// # Arguments
    /// * `start_time` - Start time
    /// * `end_time` - End time
    /// * `min_interval` - Minimum time interval
    ///
    /// # Returns
    /// Total signal count (including start and end time points)
    fn calculate_total_signals(start_time: DateTime<Utc>, end_time: DateTime<Utc>, min_interval: &KlineInterval) -> u64 {
        if start_time > end_time {
            return 0;
        }

        if start_time == end_time {
            return 1;
        }

        // Calculate time difference
        let time_diff = end_time.signed_duration_since(start_time);
        let interval_duration = Duration::from_std(min_interval.to_duration()).unwrap();

        // Calculate signal count: (time_diff / interval) + 1
        // +1 because it includes the starting point
        let count = time_diff.num_milliseconds() / interval_duration.num_milliseconds();
        (count + 1) as u64
    }

    /// Get total signal count
    pub fn total_signal_count(&self) -> u64 {
        self.total_signal_count
    }

    /// Get current index
    pub fn current_index(&self) -> u64 {
        self.current_index
    }

    /// Get playback progress (0.0 ~ 1.0)
    ///
    /// # Returns
    /// * `0.0` - Not started
    /// * `0.0 ~ 1.0` - Playing
    /// * `1.0` - Completed
    pub fn progress(&self) -> f64 {
        if self.total_signal_count == 0 {
            return 0.0;
        }

        if self.finished {
            return 1.0;
        }

        // current_index / total_count
        // Note: current_index starts from 0, so progress calculation needs adjustment
        // Example: total=10, current_index=0 means played the 1st one, progress should be 1/10 = 0.1
        let current = self.current_index as f64 + 1.0;
        let total = self.total_signal_count as f64;

        (current / total).min(1.0)
    }

    /// Get playback progress percentage (0 ~ 100)
    pub fn progress_percentage(&self) -> f64 {
        self.progress() * 100.0
    }

    /// Get remaining signal count
    pub fn remaining_signals(&self) -> u64 {
        if self.finished {
            return 0;
        }

        // total_signal_count is the total count
        // current_index is the last returned index
        // remaining = total - (current_index + 1)
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
