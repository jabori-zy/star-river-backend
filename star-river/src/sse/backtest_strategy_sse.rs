use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;

use crate::StarRiver;
use axum::extract::State;
use event_center::Channel;
use async_stream::stream;
use event_center::strategy_event::StrategyEvent;
use event_center::Event as EventCenterEvent;




#[utoipa::path(
    get,
    path = "/api/v1/strategy/backtest",
    tag = "策略管理",
    summary = "回测策略SSE",
    responses(
        (status = 200, description = "回测策略SSE连接成功")
    )
)]
pub async fn backtest_strategy_sse_handler(State(star_river): State<StarRiver>,) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("Backtest Strategy SSE连接成功");
    let event_center = star_river.event_center.lock().await;
    let strategy_event_receiver = event_center.subscribe(&Channel::Strategy).await.expect("订阅Strategy通道失败");
    // 使用 Guard 在连接断开时记录日志
    struct Guard {
        channel_name: &'static str,
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            tracing::info!("{} SSE连接已断开", self.channel_name);
        }
    }

    let stream = stream! {
        let _guard = Guard { channel_name: "Strategy" };
        let mut stream = tokio_stream::wrappers::BroadcastStream::new(strategy_event_receiver);
        while let Some(result) = stream.next().await {
            // 过滤事件
            let event = match result {
                Ok(EventCenterEvent::Strategy(StrategyEvent::BacktestStrategy(_))) => {
                    let json = serde_json::to_string(&result.as_ref().unwrap()).unwrap();
                    // tracing::debug!("backtest-strategy-sse: {:?}", json);
                    // tracing::debug!("序列化后的 JSON: {}", json); // 添加这行
                    Some(Event::default().data(json))
                }
                Ok(_) => None,
                Err(e) => Some(Event::default().data(format!("Error: {}", e))),
            };
            if let Some(event) = event {
                yield Ok(event);
            }
        }
    };
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("backtest-strategy-channel-keep-alive"),
    )
}