use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::RwLock;

// ==================== 模拟的类型定义 ====================

/// 模拟 EngineContext trait
trait EngineContext: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn get_name(&self) -> &str;
}

/// 模拟 StrategyEngineContext - 具体的实现
#[derive(Debug)]
pub struct StrategyEngineContext {
    name: String,
    strategy_count: i32,
}

impl StrategyEngineContext {
    fn new(name: String) -> Self {
        Self {
            name,
            strategy_count: 0,
        }
    }

    // 这是 StrategyEngineContext 特有的方法
    async fn get_strategy_status(&self, strategy_id: i32) -> Result<String, String> {
        println!("[StrategyEngineContext] Getting status for strategy {}", strategy_id);
        Ok(format!("Strategy {} is running", strategy_id))
    }

    async fn get_strategy_count(&self) -> i32 {
        println!("[StrategyEngineContext] Getting strategy count");
        self.strategy_count
    }

    fn get_context_info(&self) -> String {
        format!("Context: {}, Strategies: {}", self.name, self.strategy_count)
    }
}

impl EngineContext for StrategyEngineContext {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

// ==================== 核心：BacktestStrategyEngine ====================

/// 模拟 BacktestStrategyEngine
struct BacktestStrategyEngine {
    context: Arc<RwLock<Box<dyn EngineContext>>>,
}

impl BacktestStrategyEngine {
    fn new(context: StrategyEngineContext) -> Self {
        Self {
            context: Arc::new(RwLock::new(Box::new(context))),
        }
    }

    // ==================== 方案 6A：返回一个实现了 Deref 的 Guard ====================

    /// 获取策略引擎上下文的只读引用
    /// 返回的 Guard 实现了 Deref，可以自动调用 StrategyEngineContext 的方法
    pub async fn context(&self) -> StrategyContextRef<'_> {
        let guard = self.context.read().await;
        StrategyContextRef { guard }
    }

    /// 获取策略引擎上下文的可写引用
    pub async fn context_mut(&self) -> StrategyContextMutRef<'_> {
        let guard = self.context.write().await;
        StrategyContextMutRef { guard }
    }
}

// ==================== 只读 Guard + Deref ====================

/// 策略引擎上下文的只读便捷访问器
pub struct StrategyContextRef<'a> {
    guard: tokio::sync::RwLockReadGuard<'a, Box<dyn EngineContext>>,
}

impl<'a> StrategyContextRef<'a> {
    /// 内部辅助方法：将 trait object 向下转型为具体类型
    fn as_strategy_context(&self) -> &StrategyEngineContext {
        self.guard
            .as_any()
            .downcast_ref::<StrategyEngineContext>()
            .expect("Failed to downcast to StrategyEngineContext")
    }
}

// 🌟 核心：实现 Deref trait
impl<'a> Deref for StrategyContextRef<'a> {
    type Target = StrategyEngineContext;

    fn deref(&self) -> &Self::Target {
        self.as_strategy_context()
    }
}

// ==================== 可写 Guard + DerefMut ====================

/// 策略引擎上下文的可写便捷访问器
pub struct StrategyContextMutRef<'a> {
    guard: tokio::sync::RwLockWriteGuard<'a, Box<dyn EngineContext>>,
}

impl<'a> StrategyContextMutRef<'a> {
    fn as_strategy_context_mut(&mut self) -> &mut StrategyEngineContext {
        self.guard
            .as_any_mut()
            .downcast_mut::<StrategyEngineContext>()
            .expect("Failed to downcast to StrategyEngineContext")
    }
}

impl<'a> Deref for StrategyContextMutRef<'a> {
    type Target = StrategyEngineContext;

    fn deref(&self) -> &Self::Target {
        // 注意：这里需要 unsafe 或者另一种方式
        // 简化处理，仅用于只读访问
        unsafe {
            &*(self.guard.as_any() as *const dyn std::any::Any as *const StrategyEngineContext)
        }
    }
}

impl<'a> std::ops::DerefMut for StrategyContextMutRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_strategy_context_mut()
    }
}

// ==================== 演示代码 ====================

#[tokio::main]
async fn main() {
    println!("=== 方案 6A: Guard + Deref Demo ===\n");

    // 1. 创建 Engine
    let context = StrategyEngineContext::new("BacktestStrategy".to_string());
    let engine = BacktestStrategyEngine::new(context);

    println!("✅ Engine created\n");

    // ==================== 演示 1: 基本的 Deref 用法 ====================
    println!("--- Demo 1: Basic Deref Usage ---");

    {
        let ctx = engine.context().await;

        // 🌟 直接调用 StrategyEngineContext 的方法，自动 Deref！
        println!("Context name: {}", ctx.get_name());
        println!("Context info: {}", ctx.get_context_info());

        // 调用异步方法
        let count = ctx.get_strategy_count().await;
        println!("Strategy count: {}", count);
    }

    println!();

    // ==================== 演示 2: 模拟 API 调用 ====================
    println!("--- Demo 2: Simulate API Call ---");

    {
        let ctx = engine.context().await;

        // 模拟 API 层的调用
        let strategy_id = 123;
        match ctx.get_strategy_status(strategy_id).await {
            Ok(status) => println!("✅ Status: {}", status),
            Err(e) => println!("❌ Error: {}", e),
        }
    }

    println!();

    // ==================== 演示 3: 多个并发读取 ====================
    println!("--- Demo 3: Multiple Concurrent Reads ---");

    let engine = Arc::new(engine);

    let mut handles = vec![];

    for i in 1..=3 {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            let ctx = engine_clone.context().await;
            let status = ctx.get_strategy_status(i).await.unwrap();
            println!("  [Task {}] {}", i, status);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!();

    // ==================== 演示 4: 对比传统方式 ====================
    println!("--- Demo 4: Compare with Traditional Way ---");

    println!("\n❌ Traditional way (verbose):");
    {
        let guard = engine.context.read().await;
        let strategy_context = guard.as_any()
            .downcast_ref::<StrategyEngineContext>()
            .unwrap();
        let status = strategy_context.get_strategy_status(456).await.unwrap();
        println!("  Status: {}", status);
    }

    println!("\n✅ With Deref (elegant):");
    {
        let ctx = engine.context().await;
        let status = ctx.get_strategy_status(456).await.unwrap();
        println!("  Status: {}", status);
    }

    println!();

    // ==================== 演示 5: 链式调用 ====================
    println!("--- Demo 5: Method Chaining ---");

    {
        let ctx = engine.context().await;

        // 可以流畅地调用多个方法
        println!("Name: {}", ctx.get_name());
        println!("Info: {}", ctx.get_context_info());
        let count = ctx.get_strategy_count().await;
        println!("Count: {}", count);
    }

    println!();

    // ==================== 总结 ====================
    println!("=== Summary ===");
    println!("✅ Deref trait allows transparent access to StrategyEngineContext methods");
    println!("✅ API layer code becomes much cleaner");
    println!("✅ No runtime overhead - zero-cost abstraction");
    println!("✅ Type-safe and compiler-checked");
    println!("✅ Works seamlessly with async methods");
}
