use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{Balance, Equity};
use utoipa::ToSchema;

/// Asset snapshot - used to save historical asset data and generate charts
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsSnapshot {
    /// Timestamp (milliseconds)
    pub datetime: DateTime<Utc>,

    /// Account balance (initial capital + realized P&L)
    pub balance: Balance,

    /// Available balance (equity - used margin - frozen margin)
    pub available_balance: Balance,

    /// Unrealized P&L
    pub unrealized_pnl: f64,

    /// Equity (account balance + unrealized P&L)
    pub equity: Equity,

    /// Cumulative return (percentage)
    pub cumulative_return: f64,

    /// Realized P&L
    pub realized_pnl: f64,
}

impl StatsSnapshot {
    /// Create new asset snapshot
    pub fn new(
        datetime: DateTime<Utc>,
        initial_balance: Balance,
        balance: Balance,
        available_balance: Balance,
        unrealized_pnl: f64,
        equity: Equity,
        realized_pnl: f64,
    ) -> Self {
        // Cumulative return = (equity - initial capital) / initial capital * 100%
        let cumulative_return = if initial_balance != 0.0 {
            (equity - initial_balance) / initial_balance
        } else {
            0.0
        };

        Self {
            datetime,
            balance,
            available_balance,
            unrealized_pnl,
            equity,
            cumulative_return,
            realized_pnl,
        }
    }

    /// Get net value (ratio relative to initial capital)
    pub fn get_net_value(&self) -> f64 {
        if self.balance != 0.0 { self.equity / self.balance } else { 1.0 }
    }
}

/// Asset snapshot history - used to store time series data and generate charts
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsSnapshotHistory {
    /// Snapshot list, sorted by time
    pub snapshots: Vec<StatsSnapshot>,

    /// Maximum number of snapshots to keep (to prevent unlimited memory growth). If None, no limit is imposed
    pub max_snapshots: Option<usize>,

    /// Cached historical maximum equity (for incremental max drawdown calculation)
    #[serde(skip)]
    max_equity: f64,

    /// Cached maximum drawdown (percentage)
    #[serde(skip)]
    max_drawdown: f64,
}

impl StatsSnapshotHistory {
    /// Create new asset snapshot history
    pub fn new(max_snapshots: Option<usize>) -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots,
            max_equity: 0.0,
            max_drawdown: 0.0,
        }
    }

    /// Add new snapshot
    pub fn add_snapshot(&mut self, snapshot: StatsSnapshot) {
        let snapshot_equity = snapshot.equity;

        self.snapshots.push(snapshot);

        // incremental update max drawdown
        // 1. update max equity
        if snapshot_equity > self.max_equity {
            self.max_equity = snapshot_equity;
        }

        // 2. calculate current drawdown
        let current_drawdown = if self.max_equity != 0.0 {
            (self.max_equity - snapshot_equity) / self.max_equity * 100.0
        } else {
            0.0
        };

        // 3. update max drawdown
        if current_drawdown > self.max_drawdown {
            self.max_drawdown = current_drawdown;
        }

        // keep max snapshots limit, if max_snapshots is None, then no limit
        if let Some(max_snapshots) = self.max_snapshots {
            if self.snapshots.len() > max_snapshots {
                let excess = self.snapshots.len() - max_snapshots;
                let removed_snapshots = self.snapshots.drain(0..excess);

                // Check if removed snapshots contain the maximum equity
                // If so, need to recalculate
                let need_recalculate = removed_snapshots.into_iter().any(|s| s.equity >= self.max_equity);

                if need_recalculate {
                    self.recalculate_max_drawdown();
                }
            }
        }
    }

    /// get latest snapshot
    pub fn get_latest_snapshot(&self) -> Option<&StatsSnapshot> {
        self.snapshots.last()
    }

    /// get snapshots count
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// get snapshots in range
    pub fn get_snapshots_in_range(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Vec<&StatsSnapshot> {
        self.snapshots
            .iter()
            .filter(|snapshot| snapshot.datetime >= start_time && snapshot.datetime <= end_time)
            .collect()
    }

    /// get max drawdown (O(1) time complexity)
    ///
    /// return cached max drawdown percentage
    pub fn calculate_max_drawdown(&self) -> f64 {
        self.max_drawdown
    }

    /// get max equity
    pub fn get_max_equity(&self) -> f64 {
        self.max_equity
    }

    /// clear history
    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.max_equity = 0.0;
        self.max_drawdown = 0.0;
    }

    /// Recalculate max drawdown (only when necessary, such as removing snapshots containing the highest equity)
    ///
    /// Time complexity: O(n)
    fn recalculate_max_drawdown(&mut self) {
        if self.snapshots.is_empty() {
            self.max_equity = 0.0;
            self.max_drawdown = 0.0;
            return;
        }

        // recalculate max equity and max drawdown
        let mut max_equity = self.snapshots[0].equity;
        let mut max_drawdown = 0.0;

        for snapshot in &self.snapshots {
            if snapshot.equity > max_equity {
                max_equity = snapshot.equity;
            }

            let drawdown = if max_equity != 0.0 {
                (max_equity - snapshot.equity) / max_equity * 100.0
            } else {
                0.0
            };

            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        self.max_equity = max_equity;
        self.max_drawdown = max_drawdown;
    }

    /// get snapshots before datetime
    pub fn get_snapshots_before_datetime(&self, datetime: DateTime<Utc>) -> Vec<StatsSnapshot> {
        self.snapshots
            .iter()
            .filter(|snapshot| snapshot.datetime <= datetime)
            .cloned()
            .collect()
    }
}
