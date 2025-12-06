use std::{convert::Infallible, time::Duration};

use async_stream::stream;
use axum::response::sse::{Event, Sse};
use event_center::{EventCenterSingleton, event::Channel};
use futures::stream::Stream;
use tokio_stream::StreamExt;

#[utoipa::path(
    get,
    path = "/api/v1/sse/account",
    tag = "Account Management",
    summary = "Real-time account information push",
    description = "Establish Server-Sent Events (SSE) connection to receive real-time account-related event pushes, including account configuration changes, status updates, etc.",
    responses(
        (
            status = 200,
            description = "SSE connection established successfully, start pushing account event data",
            content_type = "text/event-stream",
            headers(
                ("Cache-Control" = String, description = "Cache control: no-cache"),
                ("Connection" = String, description = "Connection type: keep-alive"),
                ("Content-Type" = String, description = "Content type: text/event-stream")
            )
        ),
        (
            status = 500,
            description = "Internal server error, unable to establish SSE connection",
            content_type = "application/json"
        )
    ),
    params(),
    security()
)]
pub async fn account_sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("Account SSE connection successful");

    // let event_center = star_river.event_center.lock().await;
    let account_event_receiver = EventCenterSingleton::subscribe(&Channel::Account)
        .await
        .expect("Failed to subscribe to Account channel");

    // Use Guard to log when connection is disconnected
    struct Guard {
        channel_name: &'static str,
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            tracing::info!("{} SSE connection disconnected", self.channel_name);
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
        // _guard is dropped here, triggering the disconnection log
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("account-channel-keep-alive"),
    )
}
