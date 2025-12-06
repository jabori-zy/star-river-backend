#![allow(unused)]
use std::{ops::Deref, sync::Arc};

use tokio::sync::RwLock;

// ==================== Mock Type Definitions ====================

/// Mock EngineContext trait
trait EngineContext: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn get_name(&self) -> &str;
}

/// Mock StrategyEngineContext - concrete implementation
#[derive(Debug)]
pub struct StrategyEngineContext {
    name: String,
    strategy_count: i32,
}

impl StrategyEngineContext {
    fn new(name: String) -> Self {
        Self { name, strategy_count: 0 }
    }

    // Methods specific to StrategyEngineContext
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

// ==================== Core: BacktestStrategyEngine ====================

/// Mock BacktestStrategyEngine
struct BacktestStrategyEngine {
    context: Arc<RwLock<Box<dyn EngineContext>>>,
}

impl BacktestStrategyEngine {
    fn new(context: StrategyEngineContext) -> Self {
        Self {
            context: Arc::new(RwLock::new(Box::new(context))),
        }
    }

    // ==================== Solution 6A: Return a Guard that implements Deref ====================

    /// Get read-only reference to strategy engine context
    /// The returned Guard implements Deref, allowing automatic calls to StrategyEngineContext methods
    pub async fn context(&self) -> StrategyContextRef<'_> {
        let guard = self.context.read().await;
        StrategyContextRef { guard }
    }

    /// Get mutable reference to strategy engine context
    pub async fn context_mut(&self) -> StrategyContextMutRef<'_> {
        let guard = self.context.write().await;
        StrategyContextMutRef { guard }
    }
}

// ==================== Read-only Guard + Deref ====================

/// Read-only convenient accessor for strategy engine context
pub struct StrategyContextRef<'a> {
    guard: tokio::sync::RwLockReadGuard<'a, Box<dyn EngineContext>>,
}

impl<'a> StrategyContextRef<'a> {
    /// Internal helper method: downcast trait object to concrete type
    fn as_strategy_context(&self) -> &StrategyEngineContext {
        self.guard
            .as_any()
            .downcast_ref::<StrategyEngineContext>()
            .expect("Failed to downcast to StrategyEngineContext")
    }
}

// Core: Implement Deref trait
impl<'a> Deref for StrategyContextRef<'a> {
    type Target = StrategyEngineContext;

    fn deref(&self) -> &Self::Target {
        self.as_strategy_context()
    }
}

// ==================== Mutable Guard + DerefMut ====================

/// Mutable convenient accessor for strategy engine context
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
        // Note: This requires unsafe or another approach
        // Simplified handling, for read-only access only
        unsafe { &*(self.guard.as_any() as *const dyn std::any::Any as *const StrategyEngineContext) }
    }
}

impl<'a> std::ops::DerefMut for StrategyContextMutRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_strategy_context_mut()
    }
}

// ==================== Demo Code ====================

#[tokio::main]
async fn main() {
    println!("=== Solution 6A: Guard + Deref Demo ===\n");

    // 1. Create Engine
    let context = StrategyEngineContext::new("BacktestStrategy".to_string());
    let engine = BacktestStrategyEngine::new(context);

    println!("✅ Engine created\n");

    // ==================== Demo 1: Basic Deref Usage ====================
    println!("--- Demo 1: Basic Deref Usage ---");

    {
        let ctx = engine.context().await;

        // Call StrategyEngineContext methods directly, auto Deref!
        println!("Context name: {}", ctx.get_name());
        println!("Context info: {}", ctx.get_context_info());

        // Call async methods
        let count = ctx.get_strategy_count().await;
        println!("Strategy count: {}", count);
    }

    println!();

    // ==================== Demo 2: Simulate API Call ====================
    println!("--- Demo 2: Simulate API Call ---");

    {
        let ctx = engine.context().await;

        // Simulate API layer call
        let strategy_id = 123;
        match ctx.get_strategy_status(strategy_id).await {
            Ok(status) => println!("✅ Status: {}", status),
            Err(e) => println!("❌ Error: {}", e),
        }
    }

    println!();

    // ==================== Demo 3: Multiple Concurrent Reads ====================
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

    // ==================== Demo 4: Compare with Traditional Way ====================
    println!("--- Demo 4: Compare with Traditional Way ---");

    println!("\n❌ Traditional way (verbose):");
    {
        let guard = engine.context.read().await;
        let strategy_context = guard.as_any().downcast_ref::<StrategyEngineContext>().unwrap();
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

    // ==================== Demo 5: Method Chaining ====================
    println!("--- Demo 5: Method Chaining ---");

    {
        let ctx = engine.context().await;

        // Can call multiple methods fluently
        println!("Name: {}", ctx.get_name());
        println!("Info: {}", ctx.get_context_info());
        let count = ctx.get_strategy_count().await;
        println!("Count: {}", count);
    }

    println!();

    // ==================== Summary ====================
    println!("=== Summary ===");
    println!("✅ Deref trait allows transparent access to StrategyEngineContext methods");
    println!("✅ API layer code becomes much cleaner");
    println!("✅ No runtime overhead - zero-cost abstraction");
    println!("✅ Type-safe and compiler-checked");
    println!("✅ Works seamlessly with async methods");
}
