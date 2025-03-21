use tokio::{select, time::Duration};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    println!("程序开始执行");
    
    let token = CancellationToken::new();
    let cloned_token = token.clone();

    // 模拟一个长时间运行的任务
    let join_handle = tokio::spawn(async move {
        println!("长时间任务开始执行");
        select! {
            _ = cloned_token.cancelled() => {
                println!("任务收到取消信号");
                
            }
            _ = async {
                let mut i = 0;
                loop {
                    i += 1;
                    println!("i: {}", i);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            } => {
                println!("长时间任务完成（这行不应该被打印）");
            }
        };
    });

    // 模拟在短暂延迟后发送取消信号
    let cancel_handle = tokio::spawn(async move {
        println!("等待 2 秒后发送取消信号...");
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("发送取消信号");
        token.cancel();
        println!("取消信号已发送");
    });

    // 等待所有任务完成
    println!("等待任务完成...");
    join_handle.await.unwrap();
    cancel_handle.await.unwrap();
    println!("程序正常结束");
}