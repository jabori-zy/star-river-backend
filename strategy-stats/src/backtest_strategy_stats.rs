use types::custom_type::{Balance, Leverage};
use types::virtual_trading_system::event::{VirtualTradingSystemEvent, VirtualTradingSystemEventReceiver};
use types::strategy_stats::event::StrategyStatsEventSender;
use types::strategy_stats::{AssetSnapshot, AssetSnapshotHistory};
use virtual_trading::VirtualTradingSystem;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use utils;


#[derive(Debug)]
pub struct BacktestStrategyStats {
    pub initial_balance: Balance,
    pub leverage: Leverage,
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    pub virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver,
    pub strategy_stats_event_sender: StrategyStatsEventSender,
    cancel_token: CancellationToken,
    asset_snapshot_history: Arc<RwLock<AssetSnapshotHistory>>, // 资产快照历史
    play_index: Arc<RwLock<i32>>,
    timestamp: Arc<RwLock<i64>>,

    

}

impl BacktestStrategyStats {
    pub fn new(
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver, 
        strategy_stats_event_sender: StrategyStatsEventSender
    ) -> Self {
        Self {
            initial_balance: 0.0,
            leverage: 0,
            virtual_trading_system,
            virtual_trading_system_event_receiver,
            strategy_stats_event_sender,
            cancel_token: CancellationToken::new(),
            asset_snapshot_history: Arc::new(RwLock::new(AssetSnapshotHistory::new(None))),
            play_index: Arc::new(RwLock::new(0)),
            timestamp: Arc::new(RwLock::new(0)),
        }
    }

    pub fn set_initial_balance(&mut self, initial_balance: Balance) {
        self.initial_balance = initial_balance;
    }

    pub fn set_leverage(&mut self, leverage: Leverage) {
        self.leverage = leverage;
    }


    pub async fn start_listening(stats: Arc<RwLock<Self>>) -> Result<(), String> {
        let (receiver, cancel_token) = {
            let guard = stats.read().await;
            let receiver = guard.virtual_trading_system_event_receiver.resubscribe();
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
                                let guard = stats_clone.read().await;
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

    async fn handle_virtual_trading_system_event(&self, event: VirtualTradingSystemEvent) -> Result<(), String> {
        tracing::info!("策略统计模块收到虚拟交易系统事件: {:?}", event);
        
        // 处理事件并更新资产快照
        match event {
            VirtualTradingSystemEvent::PlayIndexUpdated((play_index, timestamp)) => {
                let mut play_index_guard = self.play_index.write().await;
                *play_index_guard = play_index;
                let mut timestamp_guard = self.timestamp.write().await;
                *timestamp_guard = timestamp;
                tracing::info!("策略统计模块更新时间戳: {:?}, 播放索引: {:?}", timestamp, play_index);
            }
            VirtualTradingSystemEvent::FuturesOrderFilled(_) |
            VirtualTradingSystemEvent::PositionCreated(_) |
            VirtualTradingSystemEvent::PositionUpdated(_) => {
                // 订单成交或仓位变化时更新资产快照
                self.update_asset_snapshot().await?;
            }
            _ => {}
        }
        
        Ok(())
    }

    /// 更新资产快照
    async fn update_asset_snapshot(&self) -> Result<(), String> {
        let trading_system = self.virtual_trading_system.lock().await;
        
        let timestamp = utils::get_utc8_timestamp_millis();
        let play_index = trading_system.get_play_index();
        let current_balance = trading_system.get_current_balance();
        let positions = trading_system.get_positions();
        
        // 计算未实现盈亏
        let unrealized_pnl: f64 = positions.iter()
            .map(|pos| pos.unrealized_profit)
            .sum();
            
        let position_count = positions.len() as u32;
        
        drop(trading_system);
        
        // 创建资产快照
        let snapshot = AssetSnapshot::new(
            timestamp,
            play_index,
            self.initial_balance,
            current_balance,
            unrealized_pnl,
            position_count,
        );
        
        tracing::debug!("策略统计模块资产快照: 总资产={:.2}, 收益率={:.2}%, 仓位数={}", 
            snapshot.total_equity, snapshot.cumulative_return, snapshot.position_count);
        
        Ok(())
    }

    /// 手动创建资产快照
    pub async fn create_asset_snapshot(&mut self) -> Result<(), String> {
        let trading_system = self.virtual_trading_system.lock().await;
        
        let timestamp = utils::get_utc8_timestamp_millis();
        let play_index = trading_system.get_play_index();
        let current_balance = trading_system.get_current_balance();
        let positions = trading_system.get_positions();
        
        // 计算未实现盈亏
        let unrealized_pnl: f64 = positions.iter()
            .map(|pos| pos.unrealized_profit)
            .sum();
            
        let position_count = positions.len() as u32;
        
        drop(trading_system);
        
        // 创建资产快照
        let snapshot = types::strategy_stats::AssetSnapshot::new(
            timestamp,
            play_index,
            self.initial_balance,
            current_balance,
            unrealized_pnl,
            position_count,
        );
        
        // 添加到历史记录
        let mut asset_snapshot_history_guard = self.asset_snapshot_history.write().await;
        asset_snapshot_history_guard.add_snapshot(snapshot);
        
        tracing::debug!("策略统计模块创建资产快照: 总资产={:.2}, 收益率={:.2}%, 仓位数={}", 
            asset_snapshot_history_guard.get_latest_snapshot().unwrap().total_equity,
            asset_snapshot_history_guard.get_latest_snapshot().unwrap().cumulative_return,
            asset_snapshot_history_guard.get_latest_snapshot().unwrap().position_count);
        
        Ok(())
    }

    /// 获取资产快照历史记录
    pub async fn get_asset_snapshot_history(&self) -> Arc<RwLock<AssetSnapshotHistory>> {
        let asset_snapshot_history_guard = self.asset_snapshot_history.read().await;
        Arc::new(RwLock::new(asset_snapshot_history_guard.clone()))
    }

    /// 获取用于画图的数据
    pub async fn get_chart_data(&self) -> (Vec<i64>, Vec<f64>, Vec<f64>) {
        let asset_snapshot_history_guard = self.asset_snapshot_history.read().await;
        asset_snapshot_history_guard.get_chart_data()
    }

    /// 获取净值曲线数据
    pub async fn get_net_value_curve(&self) -> (Vec<i64>, Vec<f64>) {
        let asset_snapshot_history_guard = self.asset_snapshot_history.read().await;
        asset_snapshot_history_guard.get_net_value_curve()
    }

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