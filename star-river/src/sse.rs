use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;
use serde::Deserialize;

use crate::StarRiver;
use axum::extract::{State, Query};
use event_center::Channel;



pub async fn market_sse_handler(
    State(star_river): State<StarRiver>
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("Market SSE连接成功");

    let event_center = star_river.event_center.lock().await;

    let market_event_receiver = event_center.subscribe(&Channel::Market).expect("订阅Market通道失败");

    let stream = tokio_stream::wrappers::BroadcastStream::new(market_event_receiver)
    .map(|result| {
        result.map(|event| {
            let json = serde_json::to_string(&event).unwrap();
            Event::default().data(json)
        })
        .unwrap_or_else(|e| {
            Event::default().data(format!("Error: {}", e))
        })

    })
    .map(Ok::<_, Infallible>);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("market-channel-keep-alive"),
    )
}


pub async fn indicator_sse_handler(
    State(star_river): State<StarRiver>
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("Indicator SSE连接成功");

    let event_center = star_river.event_center.lock().await;

    let indicator_event_receiver = event_center.subscribe(&Channel::Indicator).expect("订阅Indicator通道失败");

    let stream = tokio_stream::wrappers::BroadcastStream::new(indicator_event_receiver)
    .map(|result| {
        result.map(|event| {
            let json = serde_json::to_string(&event).unwrap();
            Event::default().data(json)


        })
        .unwrap_or_else(|e| {
            Event::default().data(format!("Error: {}", e))
        })

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

pub async fn strategy_sse_handler(
    State(star_river): State<StarRiver>,
    Query(query): Query<StrategySSEQuery>
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let strategy_id = query.strategy_id;

    let event_center = star_river.event_center.lock().await;

    let strategy_event_receiver = event_center.subscribe(&Channel::Strategy).expect("订阅Strategy通道失败");

    let stream = tokio_stream::wrappers::BroadcastStream::new(strategy_event_receiver)
    .map(|result| {
        result.map(|event| {
            let json = serde_json::to_string(&event).unwrap();
            Event::default().data(json)


        })
        .unwrap_or_else(|e| {
            Event::default().data(format!("Error: {}", e))
        })

    })
    .map(Ok::<_, Infallible>);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("strategy-channel-keep-alive"),
    )
    
}

