use zeromq::{Socket, SocketRecv};


#[tokio::main]
async fn main() {
    let mut socket = zeromq::SubSocket::new();
    socket.connect("tcp://127.0.0.1:5555").await.unwrap();
    socket.subscribe("market").await.unwrap();
    println!("Subscriber is connected to tcp://127.0.0.1:5555 and subscribed to market");

    loop {
        let msg = socket.recv().await.unwrap();
        let msg_str = String::from_utf8_lossy(&msg.get(0).unwrap());
        let parts = msg_str.split(" ").collect::<Vec<&str>>();
        let topic = parts[0];
        let message = parts[1..].join(" ");
        println!("Received: {:?}", message);
    }
}