#[tokio::main]
async fn main() {
    println!("=== AsyncFnOnce Example ===\n");

    // Example 1: Basic AsyncFnOnce closure
    println!("1. Basic Usage:");
    let value = String::from("Hello");
    let async_closure = async move |x: i32| {
        println!("  Closure execution, captured value: {}", value);
        x * 2
    };
    let result = async_closure(5).await;
    println!("  Result: {}\n", result);

    // Example 2: Using AsyncFnOnce as function parameter
    println!("2. As Function Parameter:");
    async fn execute_once<F>(f: F) -> i32
    where
        F: AsyncFnOnce(i32) -> i32,
    {
        f(10).await
    }

    let data = vec![1, 2, 3];
    let result = execute_once(async move |x| {
        println!("  Processing data: {:?}", data);
        x + data.len() as i32
    })
    .await;
    println!("  Result: {}\n", result);

    // Example 3: Demonstrate can only be called once
    println!("3. Can Only Be Called Once:");
    let owned_string = String::from("This string will be moved");
    let once_closure = async move |prefix: &str| {
        println!("  {}: {}", prefix, owned_string);
        owned_string.len()
    };

    let result = once_closure("First call").await;
    println!("  String length: {}", result);
    // Note: once_closure has been consumed, cannot be called again
    // once_closure("Second call").await; // This will cause a compile error!
    println!("  (Closure has been consumed, cannot be called again)\n");

    // Example 4: Combined with async tasks
    println!("4. Combined with Async Tasks:");
    let task_data = vec![1, 2, 3, 4, 5];
    let async_task = async move || {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("  Async task completed");
        task_data.iter().sum::<i32>()
    };

    let sum = async_task().await;
    println!("  Array sum: {}\n", sum);

    // Example 5: Error handling
    println!("5. Error Handling:");
    let risky_closure =
        async move |x: i32| -> Result<i32, String> { if x > 0 { Ok(x * 2) } else { Err("Value must be greater than 0".to_string()) } };

    match risky_closure(5).await {
        Ok(result) => println!("  Success: {}", result),
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n=== Example Completed ===");
}
