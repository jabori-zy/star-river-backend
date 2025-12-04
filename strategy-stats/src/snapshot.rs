use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::{Balance, Equity};
use utoipa::ToSchema;

/// 资产快照 - 用于保存历史资产数据并生成图表
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsSnapshot {
    /// 时间戳（毫秒）
    pub datetime: DateTime<Utc>,

    /// 账户余额（初始资金 + 已实现盈亏）
    pub balance: Balance,

    /// 可用余额（净值 - 已用保证金 - 冻结保证金）
    pub available_balance: Balance,

    /// 未实现盈亏
    pub unrealized_pnl: f64,

    /// 净值（账户余额 + 未实现盈亏）
    pub equity: Equity,

    /// 累计收益率（百分比）
    pub cumulative_return: f64,

    /// 已实现盈亏
    pub realized_pnl: f64,
}

impl StatsSnapshot {
    /// 创建新的资产快照
    pub fn new(
        datetime: DateTime<Utc>,
        initial_balance: Balance,
        balance: Balance,
        available_balance: Balance,
        unrealized_pnl: f64,
        equity: Equity,
        realized_pnl: f64,
    ) -> Self {
        // 累计收益率 = (净值 - 初始资金) / 初始资金 * 100%
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

    /// 获取净值（相对于初始资金的比例）
    pub fn get_net_value(&self) -> f64 {
        if self.balance != 0.0 { self.equity / self.balance } else { 1.0 }
    }
}

/// 资产快照历史记录 - 用于存储时间序列数据并生成图表
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsSnapshotHistory {
    /// 快照列表，按时间顺序排列
    pub snapshots: Vec<StatsSnapshot>,

    /// 最大保存数量（避免内存无限增长） 如果为None，则不限制保存数量
    pub max_snapshots: Option<usize>,

    /// 缓存的历史最高净值（用于增量计算最大回撤）
    #[serde(skip)]
    max_equity: f64,

    /// 缓存的最大回撤（百分比）
    #[serde(skip)]
    max_drawdown: f64,
}

impl StatsSnapshotHistory {
    /// 创建新的资产快照历史记录
    pub fn new(max_snapshots: Option<usize>) -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots,
            max_equity: 0.0,
            max_drawdown: 0.0,
        }
    }

    /// 添加新的快照
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

                // 检查是否删除了包含最高净值的快照
                // 如果删除的快照中有最高净值，需要重新计算
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

    /// 是否为空
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

    /// recalculate max drawdown (only when necessary, such as removing snapshots containing the highest equity)
    ///
    /// time complexity: O(n)
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
