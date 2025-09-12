use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;

use async_stream::stream;
use event_center::event::strategy_event::backtest_strategy_event::BacktestStrategyEvent;
use event_center::event::Event as EventCenterEvent;
use event_center::event::StrategyEvent;
use event_center::Channel;
use event_center::EventCenterSingleton;

#[utoipa::path(
    get,
    path = "/api/v1/sse/strategy/backtest/event",
    tag = "Backtest Strategy",
    summary = "Backtest Strategy Event SSE",
    responses(
        (status = 200, description = "Backtest Strategy Event SSE connection successful")
    )
)]
pub async fn backtest_strategy_event_sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("Backtest Strategy Event SSE connection successful");
    // let event_center = star_river.event_center.lock().await;
    let strategy_event_receiver = EventCenterSingleton::subscribe(&Channel::Strategy)
        .await
        .expect("订阅Strategy通道失败");
    // let strategy_event_receiver = event_center.subscribe(&Channel::Strategy).await.expect("订阅Strategy通道失败");
    // 使用 Guard 在连接断开时记录日志
    struct Guard {
        channel_name: &'static str,
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            tracing::info!("{} Event SSE connection disconnected", self.channel_name);
        }
    }

    let stream = stream! {
        let _guard = Guard { channel_name: "Strategy Event" };
        let mut stream = tokio_stream::wrappers::BroadcastStream::new(strategy_event_receiver);
        while let Some(result) = stream.next().await {
            // 过滤事件
            let event = match result {
                Ok(EventCenterEvent::Strategy(StrategyEvent::BacktestStrategy(_))) => {
                    let event = result.as_ref().unwrap();
                    match event {
                        EventCenterEvent::Strategy(StrategyEvent::BacktestStrategy(BacktestStrategyEvent::NodeStateLog(_)))  => None,
                        _ => {
                            let json = serde_json::to_string(event).unwrap();
                            // tracing::debug!("backtest-strategy-sse: {:?}", json);
                            // tracing::debug!("序列化后的 JSON: {}", json); // 添加这行
                            Some(Event::default().data(json))

                        }
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
            .text("backtest-strategy-event-channel-keep-alive"),
    )
}
