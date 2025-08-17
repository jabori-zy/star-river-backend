pub mod event;

use crate::custom_type::{Balance, Equity};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;



/// 资产快照 - 用于保存历史资产数据并生成图表
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StatsSnapshot {
    /// 时间戳（毫秒）
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    
    /// 播放索引（回测进度）
    #[serde(rename = "playIndex")]
    pub play_index: i32,

    /// 当前可用余额
    #[serde(rename = "balance")]
    pub balance: Balance,
    
    /// 未实现盈亏
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: f64,
    
    /// 总资产价值（余额 + 未实现盈亏）
    #[serde(rename = "totalEquity")]
    pub total_equity: Equity,
    
    /// 累计收益率（百分比）
    #[serde(rename = "cumulativeReturn")]
    pub cumulative_return: f64,
    
    /// 已实现盈亏
    #[serde(rename = "realizedPnl")]
    pub realized_pnl: f64,
    
    /// 当前持仓数量
    #[serde(rename = "positionCount")]
    pub position_count: u32,
}

impl StatsSnapshot {
    /// 创建新的资产快照
    pub fn new(
        timestamp: i64,
        play_index: i32,
        initial_balance: Balance,
        current_balance: Balance,
        unrealized_pnl: f64,
        position_count: u32,
    ) -> Self {
        // 总权益 = 当前余额 + 未实现盈亏
        let total_equity = current_balance + unrealized_pnl;

        // 已实现盈亏 = 当前余额 - 初始余额
        let realized_pnl = current_balance - initial_balance;

        // 累计收益率 = (总权益 - 初始资金) / 初始资金 * 100%
        let cumulative_return = if initial_balance != 0.0 {
            (total_equity - initial_balance) / initial_balance
        } else {
            0.0
        };
        
        Self {
            timestamp,
            play_index,
            balance: current_balance,
            unrealized_pnl,
            total_equity,
            cumulative_return,
            realized_pnl,
            position_count,
        }
    }
    
    /// 获取净值（相对于初始资金的比例）
    pub fn get_net_value(&self) -> f64 {
        if self.balance != 0.0 {
            self.total_equity / self.balance
        } else {
            1.0
        }
    }
}

/// 资产快照历史记录 - 用于存储时间序列数据并生成图表
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StatsSnapshotHistory {
    
    /// 快照列表，按时间顺序排列
    pub snapshots: Vec<StatsSnapshot>,
    
    /// 最大保存数量（避免内存无限增长） 如果为None，则不限制保存数量
    pub max_snapshots: Option<usize>,
}

impl StatsSnapshotHistory {
    /// 创建新的资产快照历史记录
    pub fn new(max_snapshots: Option<usize>) -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots,
        }
    }
    
    /// 添加新的快照
    pub fn add_snapshot(&mut self, snapshot: StatsSnapshot) {
        // 按时间戳顺序插入
        let insert_pos = self.snapshots
            .binary_search_by_key(&snapshot.timestamp, |s| s.timestamp)
            .unwrap_or_else(|pos| pos);
        
        self.snapshots.insert(insert_pos, snapshot);
        
        // 保持最大数量限制, 如果max_snapshots为None，则不限制保存数量
        if let Some(max_snapshots) = self.max_snapshots {
            if self.snapshots.len() > max_snapshots {
                let excess = self.snapshots.len() - max_snapshots;
                self.snapshots.drain(0..excess);
            }
        }
    }
    
    /// 获取最新快照
    pub fn get_latest_snapshot(&self) -> Option<&StatsSnapshot> {
        self.snapshots.last()
    }
    
    /// 获取快照数量
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }
    
    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }
    
    /// 获取时间范围内的快照
    pub fn get_snapshots_in_range(&self, start_time: i64, end_time: i64) -> Vec<&StatsSnapshot> {
        self.snapshots.iter()
            .filter(|snapshot| snapshot.timestamp >= start_time && snapshot.timestamp <= end_time)
            .collect()
    }
    
    
    
    /// 计算最大回撤
    pub fn calculate_max_drawdown(&self) -> f64 {
        if self.snapshots.is_empty() {
            return 0.0;
        }
        
        let mut max_equity = self.snapshots[0].total_equity;
        let mut max_drawdown = 0.0;
        
        for snapshot in &self.snapshots {
            if snapshot.total_equity > max_equity {
                max_equity = snapshot.total_equity;
            }
            
            let drawdown = if max_equity != 0.0 {
                (max_equity - snapshot.total_equity) / max_equity * 100.0
            } else {
                0.0
            };
            
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
        
        max_drawdown
    }
    
    /// 清空历史记录
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }
    
    /// 获取指定播放索引之前的所有快照
    pub fn get_snapshots_before_play_index(&self, play_index: i32) -> Vec<StatsSnapshot> {
        self.snapshots.iter()
            .filter(|snapshot| snapshot.play_index <= play_index)
            .cloned()
            .collect()
    }
}