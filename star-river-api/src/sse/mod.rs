pub mod account_sse;
pub mod backtest_strategy_event_sse;
pub mod backtest_strategy_performance_sse;
pub mod backtest_strategy_running_log_sse;
pub mod backtest_strategy_state_log_sse;

use std::{convert::Infallible, time::Duration};

pub use account_sse::account_sse_handler;
use axum::response::sse::{Event, Sse};
pub use backtest_strategy_event_sse::backtest_strategy_event_sse_handler;
pub use backtest_strategy_performance_sse::backtest_strategy_performance_sse_handler;
pub use backtest_strategy_running_log_sse::backtest_strategy_running_log_sse_handler;
pub use backtest_strategy_state_log_sse::backtest_strategy_state_log_sse_handler;
use event_center::{EventCenterSingleton, event::Channel};
use futures::stream::Stream;
use serde::Deserialize;
use tokio_stream::StreamExt;

pub async fn market_sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("Market SSE connection successful");

    // let event_center = star_river.event_center.lock().await;

    // let market_event_receiver = event_center.subscribe(&Channel::Market).await.expect("Failed to subscribe to Market channel");
    let market_event_receiver = EventCenterSingleton::subscribe(&Channel::Market).await.expect("Failed to subscribe to Market channel");

    let stream = tokio_stream::wrappers::BroadcastStream::new(market_event_receiver)
        .map(|result| {
            result
                .map(|event| {
                    let json = serde_json::to_string(&event).unwrap();
                    Event::default().data(json)
                })
                .unwrap_or_else(|e| Event::default().data(format!("Error: {}", e)))
        })
        .map(Ok::<_, Infallible>);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("market-channel-keep-alive"),
    )
}

pub async fn indicator_sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("Indicator SSE connection successful");

    // let event_center = star_river.event_center.lock().await;
    let indicator_event_receiver = EventCenterSingleton::subscribe(&Channel::Indicator)
        .await
        .expect("Failed to subscribe to Indicator channel");

    // let indicator_event_receiver = event_center.subscribe(&Channel::Indicator).await.expect("Failed to subscribe to Indicator channel");

    let stream = tokio_stream::wrappers::BroadcastStream::new(indicator_event_receiver)
        .map(|result| {
            result
                .map(|event| {
                    let json = serde_json::to_string(&event).unwrap();
                    Event::default().data(json)
                })
                .unwrap_or_else(|e| Event::default().data(format!("Error: {}", e)))
        })
        .map(Ok::<_, Infallible>);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("indicator-channel-keep-alive"),
    )
}

#[derive(Debug, Deserialize)]
pub struct StrategySSEQuery {
    pub strategy_id: i32,
}

// pub async fn live_strategy_sse_handler(
//     State(star_river): State<StarRiver>,
//     Query(query): Query<StrategySSEQuery>,
// ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
//     tracing::info!("Strategy SSE connection successful: {}", query.strategy_id);
//     let strategy_id = query.strategy_id;

//     // let event_center = star_river.event_center.lock().await;
//     let strategy_event_receiver = EventCenterSingleton::subscribe(&Channel::Strategy)
//         .await
//         .expect("Failed to subscribe to Strategy channel");

//     // let strategy_event_receiver = event_center.subscribe(&Channel::Strategy).await.expect("Failed to subscribe to Strategy channel");
//     // Use Guard to log when connection is disconnected
//     struct Guard {
//         channel_name: &'static str,
//     }

//     impl Drop for Guard {
//         fn drop(&mut self) {
//             tracing::info!("{} SSE connection disconnected", self.channel_name);
//         }
//     }

//     let stream = stream! {
//         let _guard = Guard { channel_name: "Strategy" };
//         let mut stream = tokio_stream::wrappers::BroadcastStream::new(strategy_event_receiver);
//         while let Some(result) = stream.next().await {
//             let event = match result {
//                 Ok(EventCenterEvent::Strategy(StrategyEvent::LiveStrategyDataUpdate(strategy_data))) => {
//                     let json = serde_json::to_string(&strategy_data).unwrap();
//                     Some(Event::default().data(json))
//                 }
//                 Ok(_) => None,
//                 Err(e) => Some(Event::default().data(format!("Error: {}", e))),
//             };
//             if let Some(event) = event {
//                 yield Ok(event);
//             }
//         }

//     };
//     Sse::new(stream).keep_alive(
//         axum::response::sse::KeepAlive::new()
//             .interval(Duration::from_secs(1))
//             .text(&format!("strategy-{}-channel-keep-alive", strategy_id)),
//     )
// }
