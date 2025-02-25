use zeromq::{Socket, SocketSend};



#[tokio::main]
async fn main() {
    let mut socket = zeromq::PubSocket::new();
    socket.connect("tcp://127.0.0.1:5555").await.unwrap();
    println!("Publisher is running on tcp://127.0.0.1:5555");

    

    tokio::spawn(async move {
        loop {
            let topic = "topic2"; // 主题
            let message = "服务器2";
            let msg = format!("{} {}", topic, message).to_string();
            socket.send(msg.into()).await.unwrap();
            println!("Published2: {} {}", topic, message);
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
    tokio::signal::ctrl_c().await.unwrap();
}