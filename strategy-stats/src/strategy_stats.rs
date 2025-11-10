use std::sync::Arc;

use chrono::Utc;
use star_river_core::{
    custom_type::{Balance, StrategyId},
    system::DateTimeUtc,
};
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tokio_util::sync::CancellationToken;
use virtual_trading::{VirtualTradingSystem, event::VirtualTradingSystemEvent};

use crate::{
    event::{StrategyStatsEvent, StrategyStatsUpdatedEvent},
    snapshot::{StatsSnapshot, StatsSnapshotHistory},
};

// T: VirtualTradingSystem

#[derive(Debug)]
pub struct StrategyStats {
    mode: &'static str,
    strategy_id: StrategyId,
    initial_balance: Balance,
    virtual_trading_system: Option<Arc<Mutex<VirtualTradingSystem>>>,
    strategy_stats_event_transceiver: (broadcast::Sender<StrategyStatsEvent>, broadcast::Receiver<StrategyStatsEvent>),
    cancel_token: CancellationToken,
    asset_snapshot_history: Arc<RwLock<StatsSnapshotHistory>>, // 资产快照历史
    datetime: DateTimeUtc,
}

impl StrategyStats {
    pub fn new(mode: &'static str, strategy_id: StrategyId) -> Self {
        let (strategy_stats_event_tx, strategy_stats_event_rx) = broadcast::channel::<StrategyStatsEvent>(100);

        Self {
            mode,
            strategy_id,
            initial_balance: 0.0,
            virtual_trading_system: None,
            strategy_stats_event_transceiver: (strategy_stats_event_tx, strategy_stats_event_rx),
            cancel_token: CancellationToken::new(),
            asset_snapshot_history: Arc::new(RwLock::new(StatsSnapshotHistory::new(None))),
            datetime: Utc::now(),
            // play_index_watch_rx,
        }
    }

    pub fn set_initial_balance(&mut self, initial_balance: Balance) {
        self.initial_balance = initial_balance;
    }

    pub fn set_datetime(&mut self, datetime: DateTimeUtc) {
        self.datetime = datetime;
    }

    pub async fn handle_virtual_trading_system_events(stats: Arc<RwLock<Self>>) -> Result<(), String> {
        let (receiver, cancel_token) = {
            let guard = stats.read().await;
            let virtual_trading_system = guard.virtual_trading_system.as_ref().unwrap().lock().await;
            let receiver = virtual_trading_system.get_virtual_trading_system_event_receiver();
            let cancel_token = guard.cancel_token.clone();
            (receiver, cancel_token)
        };

        // 创建一个流，用于接收节点传递过来的message
        let mut stream = BroadcastStream::new(receiver);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 如果取消信号被触发，则中止任务
                    _ = cancel_token.cancelled() => {
                        tracing::info!("策略统计模块虚拟交易系统事件监听任务已中止");
                        break;
                    }
                    // 接收消息
                    receive_result = stream.next() => {
                        match receive_result {
                            Some(Ok(event)) => {
                                let stats_clone = Arc::clone(&stats);
                                let mut guard = stats_clone.write().await;
                                if let Err(e) = guard.handle_virtual_trading_system_event(event).await {
                                    tracing::error!("处理虚拟交易系统事件失败: {}", e);
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("策略统计模块接收消息错误: {}", e);
                            }
                            None => {
                                tracing::warn!("策略统计模块所有消息流已关闭");
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn handle_virtual_trading_system_event(&mut self, event: VirtualTradingSystemEvent) -> Result<(), String> {
        // 处理事件并更新资产快照
        match event {
            VirtualTradingSystemEvent::UpdateFinished => {
                self.create_backtest_snapshot().await?;
            }
            _ => {}
        }

        Ok(())
    }

    /// 手动创建资产快照
    pub async fn create_backtest_snapshot(&mut self) -> Result<(), String> {
        let trading_system = self.virtual_trading_system.as_ref().unwrap().lock().await;

        // let play_index = *self.play_index_watch_rx.borrow_and_update();
        // let trading_system_play_index = trading_system.get_play_index();

        let datetime = trading_system.get_datetime(); // 时间戳
        let balance = trading_system.get_balance(); // 账户余额
        let available_balance = trading_system.get_available_balance(); // 可用余额
        let positions = trading_system.get_current_positions(); // 当前持仓
        let unrealized_pnl = trading_system.get_unrealized_pnl(); // 未实现盈亏
        let equity = trading_system.get_equity(); // 净值
        let realized_pnl = trading_system.get_realized_pnl(); // 已实现盈亏
        let position_count = positions.len() as u32; // 持仓数量

        drop(trading_system);

        // 创建资产快照
        let snapshot = StatsSnapshot::new(
            datetime,
            self.initial_balance,
            balance,
            available_balance,
            unrealized_pnl,
            equity,
            realized_pnl,
            position_count,
        );

        let strategy_stats_updated_event = StrategyStatsUpdatedEvent {
            strategy_id: self.strategy_id,
            stats_snapshot: snapshot.clone(),
            datetime,
        };

        let _ = self
            .strategy_stats_event_transceiver
            .0
            .send(StrategyStatsEvent::StrategyStatsUpdated(strategy_stats_updated_event));

        // 添加到历史记录
        let mut asset_snapshot_history_guard = self.asset_snapshot_history.write().await;
        asset_snapshot_history_guard.add_snapshot(snapshot);

        // tracing::debug!("策略统计模块创建资产快照: 净值={:.2}, 收益率={:.2}%, 仓位数={}",
        //     asset_snapshot_history_guard.get_latest_snapshot().unwrap().equity,
        //     asset_snapshot_history_guard.get_latest_snapshot().unwrap().cumulative_return,
        //     asset_snapshot_history_guard.get_latest_snapshot().unwrap().position_count);

        Ok(())
    }

    /// 获取资产快照历史记录
    // pub async fn get_stats_history(&self, play_index: i32) -> Vec<StatsSnapshot> {
    //     let asset_snapshot_history_guard = self.asset_snapshot_history.read().await;
    //     asset_snapshot_history_guard.get_snapshots_before_play_index(play_index)
    // }

    /// 计算最大回撤
    pub async fn calculate_max_drawdown(&self) -> f64 {
        let asset_snapshot_history_guard = self.asset_snapshot_history.read().await;
        asset_snapshot_history_guard.calculate_max_drawdown()
    }

    /// 清空资产快照历史
    pub async fn clear_asset_snapshots(&mut self) {
        let mut asset_snapshot_history_guard = self.asset_snapshot_history.write().await;
        asset_snapshot_history_guard.clear();
    }
}
