use std::net::SocketAddr;
use std::ops::ControlFlow;
use axum::body::Bytes;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade, CloseFrame};
use axum::extract::connect_info::ConnectInfo;
use axum::response::IntoResponse;
use futures::{sink::SinkExt, stream::StreamExt};

pub async fn ws_handler(ws: WebSocketUpgrade, ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        handle_socket(socket, addr)
    })

}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr) {
    tracing::info!("Client connected: {}", addr);

    if socket.send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
    .await.is_ok() {
        tracing::info!("Ping {}", addr)
    }
    else {
        tracing::error!("Failed to send ping to {}", addr);
        return;
    }

    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if process_message(msg, addr).is_break() {
                return;
            }
        }else {
            println!("client {addr} abruptly disconnected");
            return;
        }
    }

    for i in 1..5 {
        if socket
        .send(Message::Text(format!("Hi {i} times!").into()))
        .await
        .is_err() {
            println!("client {addr} abruptly disconnected");
            return;

        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }


    let (mut sender, mut receiver) = socket.split();
    let mut send_task = tokio::spawn(async move {
        let n_msg = 20;
        for i in 0..n_msg {
            // In case of any websocket error, we exit.
            if sender
                .send(Message::Text(format!("Server message {i} ...").into()))
                .await
                .is_err()
            {
                return i;
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        println!("Sending close to {addr}...");
        if let Err(e) = sender
            .send(Message::Close(Some(CloseFrame {
                code: axum::extract::ws::close_code::NORMAL,
                reason: Utf8Bytes::from_static("Goodbye"),
            })))
            .await
        {
            println!("Could not send Close due to {e}, probably it is ok?");
        }
        n_msg
    });

    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            // print message and break if instructed to do so
            if process_message(msg, addr).is_break() {
                break;
            }
        }
        cnt
    });

    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => println!("{a} messages sent to {addr}"),
                Err(a) => println!("Error sending messages {a:?}")
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => println!("Received {b} messages"),
                Err(b) => println!("Error receiving messages {b:?}")
            }
            send_task.abort();
        }
    }

    println!("Websocket context {addr} destroyed");

}


fn process_message(msg: Message, addr: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {addr} sent str: {t:?}");
        }
        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", addr, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    addr, cf.code, cf.reason
                );
            } else {
                println!(">>> {addr} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }
        Message::Pong(v) => {
            println!(">>> {addr} sent pong with {v:?}");
        }
        Message::Ping(v) => {
            println!(">>> {addr} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}


