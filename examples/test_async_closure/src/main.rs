#[tokio::main]
async fn main() {
    println!("=== AsyncFnOnce 示例 ===\n");

    // 示例 1: 基本的 AsyncFnOnce 闭包
    println!("1. 基本用法:");
    let value = String::from("Hello");
    let async_closure = async move |x: i32| {
        println!("  闭包内部执行，捕获的值: {}", value);
        x * 2
    };
    let result = async_closure(5).await;
    println!("  结果: {}\n", result);

    // 示例 2: 使用 AsyncFnOnce 作为函数参数
    println!("2. 作为函数参数:");
    async fn execute_once<F>(f: F) -> i32
    where
        F: AsyncFnOnce(i32) -> i32,
    {
        f(10).await
    }

    let data = vec![1, 2, 3];
    let result = execute_once(async move |x| {
        println!("  处理数据: {:?}", data);
        x + data.len() as i32
    })
    .await;
    println!("  结果: {}\n", result);

    // 示例 3: 展示只能调用一次的特性
    println!("3. 只能调用一次:");
    let owned_string = String::from("这个字符串会被移动");
    let once_closure = async move |prefix: &str| {
        println!("  {}：{}", prefix, owned_string);
        owned_string.len()
    };

    let result = once_closure("第一次调用").await;
    println!("  字符串长度: {}", result);
    // 注意：once_closure 已经被消耗，不能再次调用
    // once_closure("第二次调用").await; // 这会编译错误！
    println!("  （闭包已被消耗，无法再次调用）\n");

    // 示例 4: 与异步任务结合
    println!("4. 与异步任务结合:");
    let task_data = vec![1, 2, 3, 4, 5];
    let async_task = async move || {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("  异步任务完成");
        task_data.iter().sum::<i32>()
    };

    let sum = async_task().await;
    println!("  数组求和: {}\n", sum);

    // 示例 5: 错误处理
    println!("5. 错误处理:");
    let risky_closure =
        async move |x: i32| -> Result<i32, String> { if x > 0 { Ok(x * 2) } else { Err("数值必须大于0".to_string()) } };

    match risky_closure(5).await {
        Ok(result) => println!("  成功: {}", result),
        Err(e) => println!("  错误: {}", e),
    }

    println!("\n=== 示例完成 ===");
}
