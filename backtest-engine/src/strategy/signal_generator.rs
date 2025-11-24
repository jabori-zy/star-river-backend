use chrono::{DateTime, Duration, Utc};
use star_river_core::kline::KlineInterval;



#[derive(Debug)]
pub struct SignalGenerator {
    pub current_index: i32,
    pub current_time: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub min_interval: KlineInterval,
    finished: bool,
}

impl SignalGenerator {
    pub fn new(start_time: DateTime<Utc>, end_time: DateTime<Utc>, min_interval: KlineInterval) -> Self {
        Self {
            current_index: 0,
            current_time: start_time,
            start_time,
            end_time,
            min_interval,
            finished: false,
        }
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.current_time = self.start_time;
        self.finished = false;
    }
}

impl Iterator for SignalGenerator {
    type Item = (i32, DateTime<Utc>);

    fn next(&mut self) -> Option<Self::Item> {
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
}