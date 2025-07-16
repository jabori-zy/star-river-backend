use super::backtest_strategy_context::BacktestStrategyContext;
use crate::strategy_engine::node::backtest_strategy_node::start_node::StartNode;
use crate::strategy_engine::node::BacktestNodeTrait;
use tokio_util::sync::CancellationToken;
use types::strategy::strategy_inner_event::{StrategyInnerEvent, PlayIndexUpdateEvent,StrategyInnerEventPublisher};
use utils::get_utc8_timestamp_millis;
use virtual_trading::VirtualTradingSystem;
use std::sync::Arc;
use tokio::sync::{RwLock, Notify, Mutex};


#[derive(Debug)]
struct PlayContext {
    node: Box<dyn BacktestNodeTrait + 'static>,
    play_index: Arc<RwLock<i32>>,
    signal_count: Arc<RwLock<i32>>,
    is_playing: Arc<RwLock<bool>>,
    initial_play_speed: Arc<RwLock<u32>>,
    child_cancel_play_token: CancellationToken,
    virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    strategy_inner_event_publisher: StrategyInnerEventPublisher,
    updated_play_index_notify: Arc<Notify>,
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

        PlayContext {
            node,
            play_index: self.play_index.clone(),
            signal_count: self.total_signal_count.clone(),
            is_playing: self.is_playing.clone(),
            initial_play_speed: self.initial_play_speed.clone(),
            child_cancel_play_token: self.cancel_play_token.child_token(),
            virtual_trading_system: self.virtual_trading_system.clone(),
            strategy_inner_event_publisher: self.strategy_inner_event_publisher.clone(),
            updated_play_index_notify: self.updated_play_index_notify.clone(),
        }

    }

    async fn run_play_loop(context: PlayContext, strategy_name: String) {
        loop {

            // 检查取消状态
            if context.child_cancel_play_token.is_cancelled() {
                tracing::info!("{}: 收到取消信号，优雅退出播放任务", strategy_name);
                *context.is_playing.write().await = false;
                break;
            }

            // 检查暂停状态
            if let Some(should_break) = Self::handle_pause_state(&context, &strategy_name).await {
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
                Self::send_play_signal(&context, new_play_index, total_signal_count).await;
                
            }

            // 检查播放完毕
            if play_index == total_signal_count - 1 {
                Self::handle_play_finished(&context, &strategy_name, play_index).await;
                break;
            }

            // 播放延迟
            if Self::handle_play_delay(&context, &strategy_name, play_speed).await {
                break;
            }

            
        }
    }

    // 处理暂停状态
    async fn handle_pause_state(context: &PlayContext, strategy_name: &str) -> Option<bool> {
        if !*context.is_playing.read().await {
            tracing::info!("{}: 暂停播放, signal_count: {}, played_signal_count: {}", 
                context.node.get_node_id().await, 
                *context.signal_count.read().await, 
                *context.play_index.read().await);
            
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
        } else if speed > 100 {
            tracing::warn!("播放速度大于100，已调整为100");
            100
        } else {
            speed
        }
    }

    // 获取信号计数
    async fn get_context_play_index(context: &PlayContext) -> (i32, i32) {
        let total_signal_count = *context.signal_count.read().await;
        let play_index = *context.play_index.read().await;
        (total_signal_count, play_index)
    }

    // 发送播放信号
    async fn send_play_signal(context: &PlayContext, play_index: i32, total_signal_count: i32) {
        tracing::info!("=========== 发送信号 ===========");

        Self::send_play_index_update_event(
            play_index, 
            total_signal_count, 
            context.strategy_inner_event_publisher.clone()
        ).await;

        let node_clone = context.node.clone();
        let virtual_trading_system_clone = context.virtual_trading_system.clone();
        let updated_play_index_notify = context.updated_play_index_notify.clone();
        tracing::info!("等待节点索引更新完毕");
        let start_node = node_clone.as_any().downcast_ref::<StartNode>().unwrap();
        updated_play_index_notify.notified().await;
        
        let mut virtual_trading_system_guard = virtual_trading_system_clone.lock().await;
        virtual_trading_system_guard.set_play_index(play_index).await;
        
        start_node.send_play_signal(play_index).await;
        tracing::info!("节点索引更新完毕");
        
        // tokio::spawn(async move {
        //     tracing::info!("等待节点索引更新完毕");
        //     let start_node = node_clone.as_any().downcast_ref::<StartNode>().unwrap();
        //     updated_play_index_notify.notified().await;
            
        //     let mut virtual_trading_system_guard = virtual_trading_system_clone.lock().await;
        //     virtual_trading_system_guard.set_kline_cache_index(played_signal_count).await;
            
        //     start_node.send_kline_tick_signal(played_signal_count).await;
        //     tracing::info!("节点索引更新完毕");
        // });
    }

    // 增加已播放信号计数
    async fn increment_played_signal_count(context: &PlayContext) -> i32 {
        let mut play_index = context.play_index.write().await;
        *play_index += 1;
        tracing::debug!("{}: 增加已播放信号计数, play_index: {}", context.node.get_node_id().await, *play_index);
        *play_index
    }

    // 处理播放完毕
    async fn handle_play_finished(context: &PlayContext, strategy_name: &str, play_index: i32) {
        let start_node = context.node.as_any().downcast_ref::<StartNode>().unwrap();
        start_node.send_finish_signal(play_index).await;
        
        let mut virtual_trading_system_guard = context.virtual_trading_system.lock().await;
        virtual_trading_system_guard.set_play_index(play_index).await;
        
        tracing::info!("{}: k线播放完毕，正常退出播放任务", strategy_name);
        *context.is_playing.write().await = false;
    }

    // 处理播放延迟
    // true 退出循环
    // false 继续循环
    async fn handle_play_delay(context: &PlayContext, strategy_name: &str, play_speed: u32) -> bool {
        let delay_millis = 1000 / play_speed as u64;
        tracing::info!("{}: 播放速度: {}, 播放延迟: {}ms", strategy_name, play_speed, delay_millis);
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
    // pub async fn play(&self) {
    //     // 判断播放状态是否为true
    //     if !self.check_and_set_playing_state().await {
    //         return;
    //     }

    //     // 获取开始节点的索引
    //     let start_node_index = self.node_indices.get("start_node").unwrap();
    //     let node: Box<dyn BacktestNodeTrait + 'static> = self.graph.node_weight(*start_node_index).unwrap().clone();

    //     let played_signal_index = self.played_signal_index.clone();
    //     let signal_count = self.signal_count.clone();
    //     let is_playing = self.is_playing.clone();
    //     let initial_play_speed = self.initial_play_speed.clone();
        
    //     let strategy_name = self.strategy_name.clone();
    //     let child_cancel_play_token = self.cancel_play_token.child_token();

    //     let virtual_trading_system = self.virtual_trading_system.clone();
    //     let strategy_inner_event_publisher = self.strategy_inner_event_publisher.clone();
        
    //     let updated_play_index_notify = self.updated_play_index_notify.clone();

    //     tokio::spawn(async move {
    //         // let start_node = node.as_any().downcast_ref::<StartNode>().unwrap();
            

    //         loop {
    //             let updated_play_index_notify = updated_play_index_notify.clone();
    //             let virtual_trading_system = virtual_trading_system.clone();
    //             // 首先检查取消令牌状态
    //             if child_cancel_play_token.is_cancelled() {
    //                 tracing::info!("{}: 收到取消信号，优雅退出播放任务", strategy_name);
    //                 *is_playing.write().await = false;
    //                 break;
    //             }
                
    //             // tracing::info!("{}: 播放k线，signal_count: {}, played_signal_count: {}", start_node.get_node_id().await, *signal_count.read().await, *played_signal_count.read().await);
                
    //             // 1. 判断是否为播放状态
    //             // 如果不是播放状态，则continue
    //             if !*is_playing.read().await {
    //                 tracing::info!("{}: 暂停播放, signal_count: {}, played_signal_count: {}", node.get_node_id().await, *signal_count.read().await, *played_signal_index.read().await);
                    
    //                 // 使用 tokio::select! 同时等待睡眠和取消信号
    //                 tokio::select! {
    //                     _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
    //                         continue;
    //                     }
    //                     _ = child_cancel_play_token.cancelled() => {
    //                         tracing::info!("{}: 在暂停状态收到取消信号，优雅退出播放任务", strategy_name);
    //                         *is_playing.write().await = false;
    //                         break;
    //                     }
    //                 }
    //             }

    //             // 2. 获取当前播放速度
    //             let play_speed = {
    //                 let speed = *initial_play_speed.read().await;
                    
    //                 // 确保 play_speed 在合理范围内（1-100）
    //                 if speed < 1 {
    //                     tracing::warn!("播放速度小于1，已调整为1");
    //                     1
    //                 } else if speed > 100 {
    //                     tracing::warn!("播放速度大于100，已调整为100");
    //                     100
    //                 } else {
    //                     speed
    //                 }
    //             };

    //             // 3. 获取信号计数和已发送的信号计数
    //             let signal_count = {
    //                 let signal_count = signal_count.read().await;
    //                 *signal_count
    //             };

    //             let played_signal_count = {
    //                 let played_signal_count = played_signal_index.read().await;
    //                 *played_signal_count
    //             };
                
    //             // 4. 如果已发送的信号计数小于等于信号计数，则发送信号
    //             if played_signal_count <= signal_count {
    //                 tracing::info!("=========== 发送信号 ===========");

    //                 Self::send_play_index_update_event(played_signal_count, signal_count, strategy_inner_event_publisher.clone()).await;
    //                 let node_clone = node.clone();
    //                 let virtual_trading_system_clone = virtual_trading_system.clone();
    //                 tokio::spawn(async move {
    //                     tracing::info!("等待节点索引更新完毕");
    //                     let start_node = node_clone.as_any().downcast_ref::<StartNode>().unwrap();
    //                     updated_play_index_notify.clone().notified().await;
    //                     // 更新虚拟交易系统中的k线缓存索引
    //                     let mut virtual_trading_system_guard = virtual_trading_system_clone.lock().await;
    //                     virtual_trading_system_guard.set_kline_cache_index(played_signal_count).await;
    //                     // 发送信号
    //                     start_node.send_kline_tick_signal(played_signal_count).await;
    //                     tracing::info!("节点索引更新完毕");

    //                 });
    //                 // // 更新虚拟交易系统中的k线缓存索引
    //                 // let mut virtual_trading_system_guard = virtual_trading_system.lock().await;
    //                 // virtual_trading_system_guard.set_kline_cache_index(played_signal_count).await;

    //                 // // 发送信号
    //                 // start_node.send_kline_tick_signal(played_signal_count).await;


    //                 // 更新已发送的信号计数
    //                 {
    //                     let mut played_signal_count = played_signal_index.write().await;
    //                     *played_signal_count += 1;
    //                 }
    //             }

    //             // 5. 如果已发送的信号计数大于信号计数，则停止播放
    //             if played_signal_count > signal_count {
    //                 let node_clone = node.clone();
    //                 let start_node = node_clone.as_any().downcast_ref::<StartNode>().unwrap();
    //                 // 发送k线播放完毕的信号
    //                 start_node.send_finish_signal(played_signal_count).await;
    //                 // 更新虚拟交易系统中的k线缓存索引
    //                 let mut virtual_trading_system_guard = virtual_trading_system.lock().await;
    //                 virtual_trading_system_guard.set_kline_cache_index(played_signal_count).await;
    //                 tracing::info!("{}: k线播放完毕，正常退出播放任务", strategy_name);
    //                 *is_playing.write().await = false;
    //                 break;
    //             }

    //             // 根据播放速度计算延迟时间（毫秒）
    //             let delay_millis = 1000 / play_speed as u64;
                
    //             // 使用 tokio::select! 同时等待睡眠和取消信号
    //             tokio::select! {
    //                 _ = tokio::time::sleep(tokio::time::Duration::from_millis(delay_millis)) => {
    //                     // 正常继续下一次循环
    //                 }
    //                 _ = child_cancel_play_token.cancelled() => {
    //                     tracing::info!("{}: 在播放过程中收到取消信号，优雅退出播放任务", strategy_name);
    //                     *is_playing.write().await = false;
    //                     break;
    //                 }
    //             }
    //         }
            
    //         tracing::info!("{}: 播放任务已完全退出", strategy_name);
    //     });
    // }

    // 播放k线
    pub async fn play(&self) {

        // 判断是否已播放完毕
        if *self.play_index.read().await == *self.total_signal_count.read().await as i32 {
            tracing::warn!("{}: 已播放完毕，无法继续播放", self.strategy_name.clone());
            return;
        }

        // 判断播放状态是否为true
        if !self.check_and_set_playing_state().await {
            return;
        }

        

        let play_context = self.create_play_context().await;
        tracing::info!("创建播放上下文完毕: played_signal_index: {}, signal_count: {}",play_context.play_index.read().await, play_context.signal_count.read().await);
        let strategy_name = self.strategy_name.clone();
        
        tokio::spawn(async move {
            Self::run_play_loop(play_context, strategy_name).await;
        });
    }

    // 暂停播放
    pub async fn pause(&mut self) {
        // 判断播放状态是否为true
        if !*self.is_playing.read().await {
            tracing::warn!("{}: 正在暂停，无需重复暂停", self.strategy_name.clone());
            return;
        }
        tracing::info!("{}: 请求暂停播放", self.strategy_name);
        self.cancel_play_token.cancel();
        // 替换已经取消的令牌
        self.cancel_play_token = CancellationToken::new();
    }

    // 重置播放
    pub async fn reset(&mut self) {
        tracing::info!("{}: 重置播放", self.strategy_name.clone());
        self.cancel_play_token.cancel();
        // 重置信号计数
        *self.play_index.write().await = 0;
        // 重置播放状态
        *self.is_playing.write().await = false;
        // 替换已经取消的令牌
        self.cancel_play_token = CancellationToken::new();

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



    // 执行单根K线播放
    async fn execute_single_kline_play(&self, play_index: i32, signal_count: i32) {
        tracing::info!("{}: 播放单根k线，signal_count: {}, played_signal_count: {}", 
            self.strategy_name, signal_count, play_index);

        Self::send_play_index_update_event(
            play_index, 
            signal_count, 
            self.strategy_inner_event_publisher.clone()
        ).await;

        let start_node_index = self.node_indices.get("start_node").unwrap();
        let node = self.graph.node_weight(*start_node_index).unwrap().clone();
        let virtual_trading_system = self.virtual_trading_system.clone();
        let updated_play_index_notify = self.updated_play_index_notify.clone();

        tokio::spawn(async move {
            tracing::info!("等待节点索引更新完毕");
            let start_node = node.as_any().downcast_ref::<StartNode>().unwrap();
            updated_play_index_notify.notified().await;
            
            let mut virtual_trading_system_guard = virtual_trading_system.lock().await;
            virtual_trading_system_guard.set_play_index(play_index).await;
            
            start_node.send_play_signal(play_index).await;
        });
    }

    // 增加单次播放计数
    async fn increment_single_play_count(&self) -> i32 {
        let mut play_index = self.play_index.write().await;
        *play_index += 1;
        *play_index
    }

    // 播放单根k线
    pub async fn play_one_kline(&self) -> Result<i32, String> {

        if *self.play_index.read().await == *self.total_signal_count.read().await{
            tracing::warn!("{}: 已播放完毕，无法继续播放", self.strategy_name.clone());
            return Err("已播放完毕，无法继续播放".to_string());
        }

        if !self.can_play_one_kline().await {
            return Err("无法播放单根k线".to_string());
        }

        let (total_signal_count, play_index) = self.get_current_play_index().await;
        
        if play_index < total_signal_count {
            // 先增加单次播放计数
            let play_index = self.increment_single_play_count().await;
            // 再执行单根k线播放
            self.execute_single_kline_play(play_index, total_signal_count).await;
        }

        if play_index == total_signal_count - 1 {
            let start_node_index = self.node_indices.get("start_node").unwrap();
            let node = self.graph.node_weight(*start_node_index).unwrap().clone();
            let start_node = node.as_any().downcast_ref::<StartNode>().unwrap();
            start_node.send_finish_signal(play_index).await;
            
            let mut virtual_trading_system_guard = self.virtual_trading_system.lock().await;
            virtual_trading_system_guard.set_play_index(play_index).await;
            
            tracing::info!("{}: k线播放完毕，正常退出播放任务", self.strategy_name.clone());
            *self.is_playing.write().await = false;
            return Ok(play_index);
        }
        
        Ok(play_index)
    }

    // 播放单根k线
    // pub async fn play_one_kline(&self) {
    //     // 判断播放状态是否为true
    //     if *self.is_playing.read().await {
    //         tracing::warn!("{}: 正在播放，无法播放单根k线", self.strategy_name.clone());
    //         return;
    //     }

    //     // 检查是否已经播放完毕
    //     if *self.played_signal_index.read().await > *self.signal_count.read().await {
    //         tracing::warn!("{}: 已播放完毕，无法播放更多K线", self.strategy_name);
    //         return;
    //     }
    //     // 获取开始节点的索引
    //     let start_node_index = self.node_indices.get("start_node").unwrap();
    //     let node = self.graph.node_weight(*start_node_index).unwrap().clone();

    //     let signal_count = {
    //         let signal_count = self.signal_count.read().await;
    //         *signal_count
    //     };
    //     let played_signal_count = {
    //         let played_signal_count = self.played_signal_index.read().await;
    //         *played_signal_count
    //     };

    //     let updated_play_index_notify = self.updated_play_index_notify.clone();
        
    //     // 3. 如果已发送的信号计数小于等于信号计数，则发送信号
    //     if played_signal_count <= signal_count {
    //         // 发送信号
    //         tracing::info!("{}: 播放单根k线，signal_count: {}, played_signal_count: {}", self.strategy_name.clone(), signal_count, played_signal_count);

    //         Self::send_play_index_update_event(played_signal_count, signal_count, self.strategy_inner_event_publisher.clone()).await;

    //         let virtual_trading_system = self.virtual_trading_system.clone();
    //         tokio::spawn(async move {
    //             tracing::info!("等待节点索引更新完毕");
    //             let start_node = node.as_any().downcast_ref::<StartNode>().unwrap();
    //             updated_play_index_notify.notified().await;
    //             // 更新虚拟交易系统中的k线缓存索引
    //             let mut virtual_trading_system_guard = virtual_trading_system.lock().await;
    //             virtual_trading_system_guard.set_kline_cache_index(played_signal_count).await;
    //             // 发送信号
    //             start_node.send_kline_tick_signal(played_signal_count).await;
    //             tracing::info!("节点索引更新完毕");
    //         });
            

                
            
    //         // 更新已发送的信号计数
    //         {
    //             let mut played_signal_count = self.played_signal_index.write().await;
    //             *played_signal_count += 1;
    //         }
            

    //     }

    //     // 4. 如果已发送的信号计数大于信号计数，则停止播放
    //     if played_signal_count > signal_count {
    //         // 发送k线播放完毕的信号
    //         return;
    //     }
    // }

    async fn send_play_index_update_event(
        play_index: i32, 
        total_signal_count: i32, 
        strategy_inner_event_publisher: StrategyInnerEventPublisher,
    ){
        let event = StrategyInnerEvent::PlayIndexUpdate(PlayIndexUpdateEvent {
            play_index,
            total_signal_count,
            timestamp: get_utc8_timestamp_millis(),
        });
        strategy_inner_event_publisher.send(event).unwrap();
    }

}
