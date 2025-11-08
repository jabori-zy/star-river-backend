use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;

use crate::StarRiver;
use async_stream::stream;
use axum::extract::State;
use event_center_new::event::Channel;
use event_center_new::EventCenterSingleton;


#[utoipa::path(
    get,
    path = "/api/v1/sse/account",
    tag = "账户管理",
    summary = "账户信息实时推送",
    description = "建立 Server-Sent Events (SSE) 连接，实时接收账户相关事件推送，包括账户配置变更、状态更新等信息",
    responses(
        (
            status = 200,
            description = "SSE连接建立成功，开始推送账户事件数据",
            content_type = "text/event-stream",
            headers(
                ("Cache-Control" = String, description = "缓存控制: no-cache"),
                ("Connection" = String, description = "连接类型: keep-alive"),
                ("Content-Type" = String, description = "内容类型: text/event-stream")
            )
        ),
        (
            status = 500,
            description = "服务器内部错误，无法建立SSE连接",
            content_type = "application/json"
        )
    ),
    params(),
    security()
)]
pub async fn account_sse_handler(State(star_river): State<StarRiver>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("Account SSE连接成功");

    // let event_center = star_river.event_center.lock().await;
    let account_event_receiver = EventCenterSingleton::subscribe(&Channel::Account)
        .await
        .expect("订阅Account通道失败");

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
        let _guard = Guard { channel_name: "Account" };
        let mut stream = tokio_stream::wrappers::BroadcastStream::new(account_event_receiver);

        while let Some(result) = stream.next().await {
            let event = result.map(|event| {
                let json = serde_json::to_string(&event).unwrap();
                Event::default().data(json)
            })
            .unwrap_or_else(|e| {
                Event::default().data(format!("Error: {}", e))
            });

            yield Ok(event);
        }
        // _guard 在这里被丢弃，触发断开连接日志
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("account-channel-keep-alive"),
    )
}
