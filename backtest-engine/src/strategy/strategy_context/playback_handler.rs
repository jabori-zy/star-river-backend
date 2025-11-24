// std
use std::sync::Arc;

use event_center::EventCenterSingleton;
use star_river_core::custom_type::StrategyId;
use star_river_event::backtest_strategy::strategy_event::{BacktestStrategyEvent, PlayFinishedEvent};
use strategy_core::{
    benchmark::{StrategyBenchmark, strategy_benchmark::StrategyCycleTracker},
    node::NodeTrait,
    strategy::context_trait::{
        StrategyBenchmarkExt, StrategyCommunicationExt, StrategyIdentityExt, StrategyVariableExt, StrategyWorkflowExt,
    },
};
// third-party
use tokio::sync::{Mutex, Notify, RwLock, oneshot};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

// current crate
use super::BacktestStrategyContext;
// workspace crate
use crate::{
    node::BacktestNode,
    strategy::{PlayIndex, signal_generator::SignalGenerator},
};
use crate::{
    node::node_command::{NodeResetCmdPayload, NodeResetCommand},
    strategy::{
        strategy_error::{AlreadyPausingSnafu, AlreadyPlayingSnafu, BacktestStrategyError, PlayFinishedSnafu},
        strategy_state_machine::BacktestStrategyRunState,
    },
};

#[derive(Debug)]
struct PlayContext {
    strategy_id: StrategyId,
    strategy_name: String,
    node: BacktestNode,
    play_index: Arc<RwLock<PlayIndex>>,
    signal_count: Arc<RwLock<i32>>,
    is_playing: Arc<RwLock<bool>>,
    initial_play_speed: Arc<RwLock<u32>>,
    child_cancel_play_token: CancellationToken,
    execute_over_notify: Arc<Notify>,
    play_index_watch_tx: tokio::sync::watch::Sender<PlayIndex>,
    strategy_benchmark: Arc<RwLock<StrategyBenchmark>>,
    cycle_tracker: Arc<RwLock<Option<StrategyCycleTracker>>>,
    signal_generator: Arc<Mutex<SignalGenerator>>,
}

impl BacktestStrategyContext {
    // 检查并设置播放状态
    async fn check_and_set_playing_state(&self) -> bool {
        if *self.is_playing.read().await {
            tracing::warn!("{}: 正在播放，无需重复播放", self.strategy_name());
            return false;
        }
        *self.is_playing.write().await = true;
        true
    }

    async fn create_play_context(&self) -> PlayContext {
        let node = self.get_node("start_node").unwrap();

        PlayContext {
            strategy_id: self.strategy_id(),
            strategy_name: self.strategy_name().clone(),
            node: node.clone(),
            play_index: self.play_index.clone(),
            signal_count: self.total_signal_count.clone(),
            is_playing: self.is_playing.clone(),
            initial_play_speed: self.initial_play_speed.clone(),
            child_cancel_play_token: self.cancel_play_token.child_token(),
            execute_over_notify: self.execute_over_notify.clone(),
            play_index_watch_tx: self.play_index_watch_tx.clone(),
            strategy_benchmark: self.benchmark().clone(),
            cycle_tracker: self.cycle_tracker().clone(),
            signal_generator: self.signal_generator.clone(),
        }
    }

    async fn run_play_loop(context: PlayContext) {
        loop {
            // 检查取消状态
            if context.child_cancel_play_token.is_cancelled() {
                tracing::info!("[{}]: received cancel signal, exit play task", context.strategy_name.clone());
                *context.is_playing.write().await = false;
                break;
            }

            // 检查暂停状态
            if let Some(should_break) = Self::handle_pause_state(&context, &context.strategy_name).await {
                if should_break {
                    break;
                }
                continue;
            }

            // 获取播放速度
            let play_speed = Self::get_play_speed(&context).await;

            // Check if signal generator has finished
            let mut signal_generator_guard = context.signal_generator.lock().await;
            let is_finished = signal_generator_guard.is_finished();

            if is_finished {
                drop(signal_generator_guard);
                tracing::info!("[{}]: all signals played, exit play task", context.strategy_name);
                *context.is_playing.write().await = false;
                break;
            }

            // Get next signal
            let (signal_index, signal_time) = signal_generator_guard.next().unwrap();
            let is_finished_after_next = signal_generator_guard.is_finished();
            let progress = signal_generator_guard.progress();
            drop(signal_generator_guard);

            tracing::debug!(
                "[{}]: playing signal_index: {}, signal_time: {}, progress: {}",
                context.strategy_name,
                signal_index,
                signal_time,
                progress
            );

            // 单次逻辑开始
            let mut strategy_cycle_tracker = StrategyCycleTracker::new(signal_index as u32);
            strategy_cycle_tracker.start_phase("increment play index");

            context.play_index_watch_tx.send(signal_index as i32).unwrap();

            // 显式释放 cycle_tracker 锁，避免在等待 notify 时持有锁
            strategy_cycle_tracker.end_phase("increment play index");
            {
                let mut cycle_tracker_guard = context.cycle_tracker.write().await;
                *cycle_tracker_guard = Some(strategy_cycle_tracker); // 共享到策略上下文中
            }
            // 发送后，等待所有叶子节点执行完毕
            context.execute_over_notify.notified().await;

            // 检查播放完毕
            if is_finished_after_next {
                Self::handle_play_finished(&context, &context.strategy_name, signal_index as i32).await;
                break;
            }

            // 播放延迟
            // play_speed代表1秒播放多少根k线， 100代表1秒播放100根k线
            // 1000 / 100 = 10ms
            let delay_millis = 1000 / play_speed as u64;
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_millis)).await;
        }
    }

    // 处理暂停状态
    async fn handle_pause_state(context: &PlayContext, strategy_name: &str) -> Option<bool> {
        if !*context.is_playing.read().await {
            let signal_generator_guard = context.signal_generator.lock().await;
            let total_count = signal_generator_guard.total_signal_count();
            let current_index = signal_generator_guard.current_index();
            drop(signal_generator_guard);

            tracing::info!(
                "[{}]: pause play, total_signal_count: {}, current_index: {}",
                context.node.node_id().await,
                total_count,
                current_index
            );

            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                    return Some(false); // continue
                }
                _ = context.child_cancel_play_token.cancelled() => {
                    tracing::info!("[{}]: received cancel signal, exit play task", strategy_name);
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
            tracing::warn!("[{}]: play speed less than 1, adjusted to 1", context.strategy_name.clone());
            1
        } else {
            speed
        }
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

    // 处理播放完毕, 发送播放完毕事件
    async fn handle_play_finished(context: &PlayContext, strategy_name: &str, play_index: PlayIndex) {
        let finish_event: BacktestStrategyEvent =
            PlayFinishedEvent::new(context.strategy_id, context.strategy_name.clone(), play_index).into();
        let _ = EventCenterSingleton::publish(finish_event.into()).await;

        tracing::info!("[{}]: k线播放完毕，正常退出播放任务", strategy_name);
        *context.is_playing.write().await = false;
    }

    // 播放k线
    pub async fn play(&mut self) -> Result<(), BacktestStrategyError> {
        // 判断是否已播放完毕
        let signal_generator_guard = self.signal_generator.lock().await;
        let is_finished = signal_generator_guard.is_finished();
        let current_index = signal_generator_guard.current_index();
        drop(signal_generator_guard);

        if is_finished {
            tracing::warn!("[{}]: already played finished, cannot play more kline", self.strategy_name());
            return Err(PlayFinishedSnafu {
                strategy_name: self.strategy_name().clone(),
            }
            .build());
        }

        // 判断播放状态是否为true
        if !self.check_and_set_playing_state().await {
            return Err(AlreadyPlayingSnafu {}.build());
        }

        let play_context = self.create_play_context().await;
        // 说明策略刚启动，重置batch_id
        if current_index == 0 {
            self.batch_id = Uuid::new_v4();
        }

        // 更新策略状态为playing
        self.store_strategy_status(BacktestStrategyRunState::Playing.to_string()).await?;

        tokio::spawn(async move {
            Self::run_play_loop(play_context).await;
        });
        Ok(())
    }

    // 暂停播放
    pub async fn pause(&mut self) -> Result<(), BacktestStrategyError> {
        // 判断播放状态是否为true
        if !*self.is_playing.read().await {
            tracing::error!("[{}]: is pausing, cannot pause again", self.strategy_name());
            return Err(AlreadyPausingSnafu {}.build());
        }
        tracing::info!("[{}]: request pause play", self.strategy_name());
        // 更新策略状态为pausing
        self.store_strategy_status(BacktestStrategyRunState::Pausing.to_string()).await?;

        self.cancel_play_token.cancel();
        // 替换已经取消的令牌
        self.cancel_play_token = CancellationToken::new();
        Ok(())
    }

    // 重置播放
    pub async fn reset(&mut self) -> Result<(), BacktestStrategyError> {
        // 更新策略状态为ready
        self.store_strategy_status(BacktestStrategyRunState::Ready.to_string()).await?;

        // 清空日志
        self.clear_running_log().await;

        // 清空数据
        self.clear_data().await;

        // 重置所有自定义变量
        self.reset_all_custom_variables().await;

        // 重置所有系统变量
        self.reset_all_sys_variables().await;

        // reset leaf node execution tracker
        self.reset_leaf_node_execution_info();

        // 重置策略性能报告
        {
            let mut benchmark_guard = self.benchmark().write().await;
            benchmark_guard.reset();
        }

        self.cancel_play_token.cancel();
        // 重置信号计数
        *self.play_index.write().await = -1; // 重置为-1，表示未播放
        // 重置播放状态
        *self.is_playing.write().await = false;

        let mut signal_generator_guard = self.signal_generator.lock().await;
        signal_generator_guard.reset();

        // 替换已经取消的令牌
        self.cancel_play_token = CancellationToken::new();
        Ok(())
    }

    // 检查是否可以播放单根K线
    async fn can_play_one_kline(&self) -> bool {
        if *self.is_playing.read().await {
            tracing::warn!("[{}] is playing, cannot play one kline", self.strategy_name());
            return false;
        }
        true
    }

    // 播放单根k线
    pub async fn play_one(&mut self) -> Result<PlayIndex, BacktestStrategyError> {
        if !self.can_play_one_kline().await {
            return Err(AlreadyPlayingSnafu {}.build());
        }

        let mut signal_generator_guard = self.signal_generator.lock().await;
        let is_finished = signal_generator_guard.is_finished();

        if is_finished {
            tracing::warn!("[{}] already played finished", self.strategy_name());
            return Err(PlayFinishedSnafu {
                strategy_name: self.strategy_name().clone(),
            }
            .build());
        }

        let (signal_index, signal_time) = signal_generator_guard.next().unwrap();
        let is_finished_after_next = signal_generator_guard.is_finished();
        let mut strategy_cycle_tracker = StrategyCycleTracker::new(signal_index as u32);
        strategy_cycle_tracker.start_phase("increment play index");
        drop(signal_generator_guard);
        tracing::debug!("signal_index: {}, signal_time: {}", signal_index, signal_time);

        let before_reset_batch_id = self.batch_id;
        // 说明策略刚启动，重置batch_id
        if signal_index == 0 {
            self.batch_id = Uuid::new_v4();
            tracing::info!(
                "[{}]: play started, reset batch_id: {}, before batch_id: {}",
                self.strategy_name(),
                self.batch_id,
                before_reset_batch_id
            );
        }

        // 单次逻辑开始
        self.play_index_watch_tx.send(signal_index as i32).unwrap();
        if is_finished_after_next {
            let finish_event: BacktestStrategyEvent =
                PlayFinishedEvent::new(self.strategy_id(), self.strategy_name().clone(), signal_index as i32).into();
            let _ = EventCenterSingleton::publish(finish_event.into()).await;

            tracing::info!("[{}]: kline played finished, exit play task", self.strategy_name());
            self.set_is_playing(false).await;
        }

        strategy_cycle_tracker.end_phase("increment play index");
        let mut cycle_tracker_guard = self.cycle_tracker().write().await;
        *cycle_tracker_guard = Some(strategy_cycle_tracker); // 共享到策略上下文中

        Ok(signal_index as i32)
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

    pub async fn send_reset_node_event(&self) {
        let nodes = self.topological_sort().unwrap();
        for node in nodes {
            let (resp_tx, resp_rx) = oneshot::channel();
            let node_id = node.node_id().await;
            let payload = NodeResetCmdPayload {};
            let cmd = NodeResetCommand::new(node_id, resp_tx, payload);

            self.send_node_command(cmd.into()).await;
            let response = resp_rx.await.unwrap();
            if response.is_success() {
                tracing::info!("{}: 收到节点重置响应", response.node_id());
            } else {
                tracing::error!("{}: 收到节点重置响应失败", response.node_id());
            }
        }
    }
}
