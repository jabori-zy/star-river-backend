use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;

use async_stream::stream;
use event_center_new::event::Channel;
use event_center_new::EventCenterSingleton;
use event_center_new::event::Event as EventCenterEvent;
use star_river_event::backtest_strategy::strategy_event::BacktestStrategyEvent;

#[utoipa::path(
    get,
    path = "/api/v1/sse/strategy/backtest/state-log",
    tag = "Backtest Strategy",
    summary = "Backtest Strategy State Log SSE",
    responses(
        (status = 200, description = "Backtest Strategy State Log SSE connection successful")
    )
)]
pub async fn backtest_strategy_state_log_sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("Backtest Strategy State Log SSE connection successful");
    let strategy_event_receiver = EventCenterSingleton::subscribe(&Channel::Backtest)
        .await
        .expect("订阅Backtest通道失败");

    // 使用 Guard 在连接断开时记录日志
    struct Guard {
        channel_name: &'static str,
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            tracing::info!("{} SSE connection disconnected", self.channel_name);
        }
    }

    let stream = stream! {
        let _guard = Guard { channel_name: "Backtest Strategy State Log" };
        let mut stream = tokio_stream::wrappers::BroadcastStream::new(strategy_event_receiver);
        while let Some(result) = stream.next().await {
            // 过滤事件，只发送 NodeStateLog 和 StrategyStateLog 事件
            let event = match result {
                Ok(EventCenterEvent::Backtest(ref backtest_strategy_event)) => {
                    let event = result.as_ref().unwrap();
                    match backtest_strategy_event {
                        BacktestStrategyEvent::NodeStateLog(_) |
                        BacktestStrategyEvent::StrategyStateLog(_) => {
                            let json = serde_json::to_string(event).unwrap();
                            Some(Event::default().data(json))
                        }
                        _ => None
                    }
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
            .text("backtest-strategy-state-log-channel-keep-alive"),
    )
}
