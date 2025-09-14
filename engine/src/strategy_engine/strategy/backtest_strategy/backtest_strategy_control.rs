use super::backtest_strategy_context::BacktestStrategyContext;
use super::backtest_strategy_state_machine::BacktestStrategyRunState;
use crate::strategy_engine::node::backtest_strategy_node::start_node::StartNode;
use crate::strategy_engine::node::BacktestNodeTrait;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify, RwLock};
use tokio_util::sync::CancellationToken;
use star_river_core::custom_type::PlayIndex;
use star_river_core::error::engine_error::strategy_engine_error::strategy_error::backtest_strategy_error::*;
use star_river_core::strategy::strategy_inner_event::StrategyInnerEventPublisher;
use event_center::communication::strategy::backtest_strategy::command::NodeResetParams;
use uuid::Uuid;
use virtual_trading::VirtualTradingSystem;
use tokio::sync::oneshot;

#[derive(Debug)]
struct PlayContext {
    strategy_name: String,
    node: Box<dyn BacktestNodeTrait + 'static>,
    play_index: Arc<RwLock<PlayIndex>>,
    signal_count: Arc<RwLock<i32>>,
    is_playing: Arc<RwLock<bool>>,
    initial_play_speed: Arc<RwLock<u32>>,
    child_cancel_play_token: CancellationToken,
    virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    strategy_inner_event_publisher: StrategyInnerEventPublisher,
    execute_over_notify: Arc<Notify>,
    play_index_watch_tx: tokio::sync::watch::Sender<PlayIndex>,
}

impl BacktestStrategyContext {
    // 检查并设置播放状态
    async fn check_and_set_playing_state(&self) -> bool {
        if *self.is_playing.read().await {
            tracing::warn!("{}: 正在播放，无需重复播放", self.strategy_name.clone());
            return false;
        }
        *self.is_playing.write().await = true;
        true
    }

    async fn create_play_context(&self) -> PlayContext {
        let start_node_index = self.node_indices.get("start_node").unwrap();
        let node = self.graph.node_weight(*start_node_index).unwrap().clone();
        let strategy_inner_event_publisher = self.strategy_inner_event_publisher.clone().unwrap();

        PlayContext {
            strategy_name: self.strategy_name.clone(),
            node,
            play_index: self.play_index.clone(),
            signal_count: self.total_signal_count.clone(),
            is_playing: self.is_playing.clone(),
            initial_play_speed: self.initial_play_speed.clone(),
            child_cancel_play_token: self.cancel_play_token.child_token(),
            virtual_trading_system: self.virtual_trading_system.clone(),
            strategy_inner_event_publisher: strategy_inner_event_publisher,
            execute_over_notify: self.execute_over_notify.clone(),
            play_index_watch_tx: self.play_index_watch_tx.clone(),
        }
    }

    async fn run_play_loop(context: PlayContext) {
        loop {
            // 检查取消状态
            if context.child_cancel_play_token.is_cancelled() {
                tracing::info!("{}: 收到取消信号，优雅退出播放任务", context.strategy_name);
                *context.is_playing.write().await = false;
                break;
            }

            // 检查暂停状态
            if let Some(should_break) =
                Self::handle_pause_state(&context, &context.strategy_name).await
            {
                if should_break {
                    break;
                }
                continue;
            }

            // 获取播放速度
            let play_speed = Self::get_play_speed(&context).await;
            let (total_signal_count, play_index) = Self::get_context_play_index(&context).await;

            // 处理信号发送
            if play_index < total_signal_count {
                // 因为从-1开始，所以先+1，再发送信号
                let new_play_index = Self::increment_played_signal_count(&context).await;
                context.play_index_watch_tx.send(new_play_index).unwrap();
                // 发送后，等待所有叶子节点执行完毕
                context.execute_over_notify.notified().await;

                // Self::send_play_signal(&context, new_play_index).await;
            }

            // 检查播放完毕
            if play_index == total_signal_count - 1 {
                Self::handle_play_finished(&context, &context.strategy_name, play_index).await;
                break;
            }

            // 播放延迟
            if Self::handle_play_delay(&context, &context.strategy_name, play_speed).await {
                break;
            }
        }
    }

    // 处理暂停状态
    async fn handle_pause_state(context: &PlayContext, strategy_name: &str) -> Option<bool> {
        if !*context.is_playing.read().await {
            tracing::info!(
                "{}: 暂停播放, signal_count: {}, played_signal_count: {}",
                context.node.get_node_id().await,
                *context.signal_count.read().await,
                *context.play_index.read().await
            );

            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                    return Some(false); // continue
                }
                _ = context.child_cancel_play_token.cancelled() => {
                    tracing::info!("{}: 在暂停状态收到取消信号，优雅退出播放任务", strategy_name);
                    *context.is_playing.write().await = false;
                    return Some(true); // break
                }
            }
        }
        None
    }

    // 获取播放速度
    async fn get_play_speed(context: &PlayContext) -> u32 {
        let speed = *context.initial_play_speed.read().await;

        if speed < 1 {
            tracing::warn!("播放速度小于1，已调整为1");
            1
        } else {
            speed
        }
    }

    // 获取信号计数
    async fn get_context_play_index(context: &PlayContext) -> (i32, PlayIndex) {
        let total_signal_count = *context.signal_count.read().await;
        let play_index = *context.play_index.read().await;
        (total_signal_count, play_index)
    }

    // 发送播放信号
    // async fn send_play_signal(context: &PlayContext, play_index: PlayIndex) {
    // tracing::info!("=========== 发送信号 ===========");

    // Self::send_play_index_update_event(
    //     play_index,
    //     total_signal_count,
    //     context.strategy_inner_event_publisher.clone()
    // ).await;
    // 通过watch发送play_index
    // context.play_index_watch_tx.send(play_index).unwrap();

    // let node_clone = context.node.clone();
    // let virtual_trading_system_clone = context.virtual_trading_system.clone();
    // let updated_play_index_notify = context.updated_play_index_notify.clone();
    // tracing::info!("等待节点索引更新完毕");
    // let start_node = node_clone.as_any().downcast_ref::<StartNode>().unwrap();
    // updated_play_index_notify.notified().await;

    // let mut virtual_trading_system_guard = virtual_trading_system_clone.lock().await;
    // virtual_trading_system_guard.set_play_index(play_index).await;

    // start_node.send_play_signal().await;
    // tracing::info!("节点索引更新完毕");
    // }

    // 增加已播放信号计数
    async fn increment_played_signal_count(context: &PlayContext) -> i32 {
        let mut play_index = context.play_index.write().await;
        *play_index += 1;
        *play_index
    }

    // 处理播放完毕
    async fn handle_play_finished(
        context: &PlayContext,
        strategy_name: &str,
        play_index: PlayIndex,
    ) {
        let start_node = context.node.as_any().downcast_ref::<StartNode>().unwrap();
        start_node.send_finish_signal(play_index).await;

        // let mut virtual_trading_system_guard = context.virtual_trading_system.lock().await;
        // virtual_trading_system_guard.set_play_index(play_index).await;

        tracing::info!("{}: k线播放完毕，正常退出播放任务", strategy_name);
        *context.is_playing.write().await = false;
    }

    // 处理播放延迟
    // true 退出循环
    // false 继续循环
    async fn handle_play_delay(
        context: &PlayContext,
        strategy_name: &str,
        play_speed: u32,
    ) -> bool {
        // play_speed代表1秒播放多少根k线， 100代表1秒播放100根k线
        // 1000 / 100 = 10ms
        let delay_millis = 1000 / play_speed as u64;
        // tracing::info!("{}: 播放速度: {}, 播放延迟: {}ms", strategy_name, play_speed, delay_millis);
        tokio::select! {

            _ = tokio::time::sleep(tokio::time::Duration::from_millis(delay_millis)) => {
                false // 继续循环
            }
            _ = context.child_cancel_play_token.cancelled() => {
                tracing::info!("{}: 在播放过程中收到取消信号，优雅退出播放任务", strategy_name);
                *context.is_playing.write().await = false;
                true // 退出循环
            }
        }
    }

    // 播放k线
    pub async fn play(&mut self) -> Result<(), BacktestStrategyError> {
        // 判断是否已播放完毕
        if *self.play_index.read().await == *self.total_signal_count.read().await as i32 {
            tracing::warn!("{}: 已播放完毕，无法继续播放", self.strategy_name.clone());
            return Err(PlayFinishedSnafu {}.build());
        }

        // 判断播放状态是否为true
        if !self.check_and_set_playing_state().await {
            return Err(AlreadyPlayingSnafu {}.build());
        }

        let play_context = self.create_play_context().await;
        let before_reset_batch_id = self.batch_id;
        // 说明策略刚启动，重置batch_id
        if *play_context.play_index.read().await == -1 {
            self.batch_id = Uuid::new_v4();
            tracing::info!(
                "{}: 策略刚启动，重置batch_id: {}, 之前batch_id: {}",
                self.strategy_name.clone(),
                self.batch_id,
                before_reset_batch_id
            );
        }

        // 更新策略状态为playing
        self.update_strategy_status(BacktestStrategyRunState::Playing.to_string().to_lowercase())
            .await?;

        tokio::spawn(async move {
            Self::run_play_loop(play_context).await;
        });
        Ok(())
    }

    // 暂停播放
    pub async fn pause(&mut self) -> Result<(), BacktestStrategyError> {
        // 判断播放状态是否为true
        if !*self.is_playing.read().await {
            tracing::error!("{}: 正在暂停，无需重复暂停", self.strategy_name.clone());
            return Err(AlreadyPausingSnafu {}.build());
        }
        tracing::info!("{}: 请求暂停播放", self.strategy_name);
        // 更新策略状态为pausing
        self.update_strategy_status(BacktestStrategyRunState::Pausing.to_string().to_lowercase())
            .await?;

        self.cancel_play_token.cancel();
        // 替换已经取消的令牌
        self.cancel_play_token = CancellationToken::new();
        Ok(())
    }

    // 重置播放
    pub async fn reset(&mut self) -> Result<(), BacktestStrategyError> {
        tracing::info!("{}: 重置播放", self.strategy_name.clone());

        // 更新策略状态为ready
        self.update_strategy_status(BacktestStrategyRunState::Ready.to_string().to_lowercase())
            .await?;

        // 清空日志
        self.running_log.write().await.clear();

        self.cancel_play_token.cancel();
        // 重置信号计数
        *self.play_index.write().await = -1; // 重置为-1，表示未播放
                                             // 重置播放状态
        *self.is_playing.write().await = false;
        // 替换已经取消的令牌
        self.cancel_play_token = CancellationToken::new();
        Ok(())
    }

    // 检查是否可以播放单根K线
    async fn can_play_one_kline(&self) -> bool {
        if *self.is_playing.read().await {
            tracing::warn!("{}: 正在播放，无法播放单根k线", self.strategy_name);
            return false;
        }

        if *self.play_index.read().await > *self.total_signal_count.read().await {
            tracing::warn!("{}: 已播放完毕，无法播放更多K线", self.strategy_name);
            return false;
        }

        true
    }

    // 获取当前信号计数
    async fn get_current_play_index(&self) -> (i32, i32) {
        let total_signal_count = *self.total_signal_count.read().await;
        let play_index = *self.play_index.read().await;
        (total_signal_count, play_index)
    }

    // 增加单次播放计数
    async fn increment_single_play_count(&self) -> i32 {
        let mut play_index = self.play_index.write().await;
        *play_index += 1;
        *play_index
    }

    // 播放单根k线
    pub async fn play_one_kline(&mut self) -> Result<i32, BacktestStrategyError> {
        if *self.play_index.read().await == *self.total_signal_count.read().await {
            tracing::warn!("{}: 已播放完毕，无法继续播放", self.strategy_name.clone());
            return Err(PlayFinishedSnafu {}.build());
        }

        if !self.can_play_one_kline().await {
            return Err(AlreadyPlayingSnafu {}.build());
        }

        let before_reset_batch_id = self.batch_id;
        // 说明策略刚启动，重置batch_id
        if *self.play_index.read().await == -1 {
            self.batch_id = Uuid::new_v4();
            tracing::info!(
                "{}: 策略刚启动，重置batch_id: {}, 之前batch_id: {}",
                self.strategy_name.clone(),
                self.batch_id,
                before_reset_batch_id
            );
        }

        let (total_signal_count, play_index) = self.get_current_play_index().await;

        if play_index < total_signal_count {
            // 先增加单次播放计数
            let play_index = self.increment_single_play_count().await;
            // 再执行单根k线播放
            // self.execute_single_kline_play(play_index, total_signal_count).await;
            self.play_index_watch_tx.send(play_index).unwrap();
        }

        if play_index == total_signal_count - 1 {
            let start_node_index = self.node_indices.get("start_node").unwrap();
            let node = self.graph.node_weight(*start_node_index).unwrap().clone();
            let start_node = node.as_any().downcast_ref::<StartNode>().unwrap();
            start_node.send_finish_signal(play_index).await;

            // let mut virtual_trading_system_guard = self.virtual_trading_system.lock().await;
            // virtual_trading_system_guard.set_play_index(play_index).await;

            tracing::info!(
                "{}: k线播放完毕，正常退出播放任务",
                self.strategy_name.clone()
            );
            *self.is_playing.write().await = false;
            return Ok(play_index);
        }

        Ok(play_index)
    }

    // 发送播放索引更新事件
    // async fn send_play_index_update_event(
    //     play_index: i32,
    //     total_signal_count: i32,
    //     strategy_inner_event_publisher: StrategyInnerEventPublisher,
    // ){
    //     let event = StrategyInnerEvent::PlayIndexUpdate(PlayIndexUpdateEvent {
    //         play_index,
    //         total_signal_count,
    //         timestamp: get_utc8_timestamp_millis(),
    //     });
    //     strategy_inner_event_publisher.send(event).unwrap();
    // }

    pub(crate) async fn send_reset_node_event(&self) {
        let nodes = self.topological_sort();
        for node in nodes {
            let (resp_tx, resp_rx) = oneshot::channel();
            let node_reset_params = NodeResetParams::new(node.get_node_id().await, resp_tx);
            self.strategy_command_publisher
                .send(node_reset_params.into())
                .await
                .unwrap();
            let response = resp_rx.await.unwrap();
            tracing::info!("{}: 收到节点重置响应", response.node_id());
        }
    }
}
