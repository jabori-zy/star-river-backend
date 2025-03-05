

#[tokio::main]
async fn main() {
    async fn takes_async_closure(f: impl AsyncFn(u64)) {
        f(0).await;
        f(1).await;
    }
    
    takes_async_closure(async |i| {
        core::future::ready(i).await;
        println!("done with {i}.");
    }).await;
    
}
