# 回测系统订单记录展示后端方案

## 概述
本方案设计用于回测系统中订单记录的实时展示，当K线开始回放时，模拟订单会成交和更新，前端需要以表格形式实时展示订单状态变化。

## 1. 事件扩展方案

### 现有事件基础
系统已有的 `VirtualTradingSystemEvent` 包含：
- `FuturesOrderCreated(VirtualOrder)` - 订单已创建
- `FuturesOrderFilled(VirtualOrder)` - 订单已成交  
- `FuturesOrderCanceled(VirtualOrder)` - 订单已取消

### 需要扩展的事件
在 `types/src/virtual_trading_system/event.rs` 中添加：
- `FuturesOrderUpdated(VirtualOrder)` - 订单更新事件
- `FuturesOrderClosed(VirtualOrder)` - 订单平仓事件

## 2. 订单状态管理

### 订单生命周期
利用现有的 `OrderStatus` 枚举：
- `Created` - 订单创建，前端新增表格行
- `Placed` - 订单已挂单，前端更新状态
- `Partial` - 部分成交，前端更新成交信息
- `Filled` - 订单完全成交，前端移除该行
- `Canceled` - 订单取消，前端移除该行

### 前端表格显示策略
- **显示范围**：只显示 `Created`、`Placed`、`Partial` 状态的活跃订单
- **移除条件**：`Filled`、`Canceled` 状态的订单从表格中移除
- **平仓处理**：平仓操作触发订单删除事件

## 3. WebSocket实时推送方案

### 数据结构设计
```rust
// 订单表格更新事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum OrderTableEvent {
    #[serde(rename = "add")]
    OrderAdd {
        order: VirtualOrder,
        strategy_id: i32,
    },
    
    #[serde(rename = "update")]
    OrderUpdate {
        order: VirtualOrder,
        strategy_id: i32,
    },
    
    #[serde(rename = "remove")]
    OrderRemove {
        order_id: i32,
        strategy_id: i32,
    },
    
    #[serde(rename = "clear")]
    OrdersClear {
        strategy_id: i32,
    },
}
```

### WebSocket推送端点
- **连接路径**：`/ws/backtest/orders/{strategy_id}`
- **消息格式**：JSON格式的 `OrderTableEvent`
- **连接管理**：为每个策略维护独立的WebSocket连接池

### 推送逻辑流程
1. **订单创建**：`VirtualTradingSystemEvent::FuturesOrderCreated` → `OrderTableEvent::OrderAdd`
2. **订单更新**：`VirtualTradingSystemEvent::FuturesOrderUpdated` → `OrderTableEvent::OrderUpdate`
3. **订单成交**：`VirtualTradingSystemEvent::FuturesOrderFilled` → `OrderTableEvent::OrderRemove`
4. **订单取消**：`VirtualTradingSystemEvent::FuturesOrderCanceled` → `OrderTableEvent::OrderRemove`
5. **订单平仓**：`VirtualTradingSystemEvent::FuturesOrderClosed` → `OrderTableEvent::OrderRemove`
6. **回测重置**：策略重置时 → `OrderTableEvent::OrdersClear`

## 4. API接口设计

### RESTful接口
```rust
// 获取策略的所有活跃订单
GET /api/backtest/orders/{strategy_id}
Response: Vec<VirtualOrder>

// 获取历史订单记录
GET /api/backtest/orders/{strategy_id}/history?page=1&size=50
Response: {
    orders: Vec<VirtualOrder>,
    total: i64,
    page: i32,
    size: i32
}

// 清空策略订单（回测重置时使用）
DELETE /api/backtest/orders/{strategy_id}
Response: { success: bool, message: String }
```

### WebSocket接口
```rust
// WebSocket连接
WS /ws/backtest/orders/{strategy_id}

// 消息格式
{
    "action": "add|update|remove|clear",
    "order": { /* VirtualOrder对象 */ },
    "order_id": 123,
    "strategy_id": 456,
    "timestamp": 1640995200000
}
```

## 5. 数据过滤和管理

### 后端缓存策略
```rust
// 订单缓存管理器
pub struct OrderCacheManager {
    // 活跃订单缓存：StrategyId -> Vec<VirtualOrder>
    active_orders: HashMap<i32, Vec<VirtualOrder>>,
    
    // 历史订单缓存：StrategyId -> Vec<VirtualOrder>  
    history_orders: HashMap<i32, Vec<VirtualOrder>>,
    
    // WebSocket连接池：StrategyId -> Vec<WebSocketSender>
    connections: HashMap<i32, Vec<WebSocketSender>>,
}
```

### 缓存更新策略
1. **订单创建**：添加到 `active_orders`
2. **订单更新**：更新 `active_orders` 中的对应订单
3. **订单完成**：从 `active_orders` 移动到 `history_orders`
4. **策略重置**：清空对应策略的所有缓存

### 数据同步机制
- **内存缓存**：使用 HashMap 进行快速查找和更新
- **持久化**：定期将历史订单写入数据库
- **一致性保证**：通过事务确保缓存和数据库的一致性

## 6. 实现要点

### 事件监听器实现
```rust
// 在回测引擎中添加订单表格事件监听器
pub struct OrderTableEventListener {
    order_cache: Arc<RwLock<OrderCacheManager>>,
    websocket_manager: Arc<WebSocketManager>,
}

impl OrderTableEventListener {
    pub async fn handle_virtual_trading_event(
        &self, 
        event: VirtualTradingSystemEvent
    ) -> Result<(), String> {
        match event {
            VirtualTradingSystemEvent::FuturesOrderCreated(order) => {
                self.handle_order_created(order).await
            },
            VirtualTradingSystemEvent::FuturesOrderFilled(order) => {
                self.handle_order_filled(order).await
            },
            // ... 其他事件处理
        }
    }
}
```

### 状态映射规则
- `Created/Placed/Partial` → 前端表格显示
- `Filled/Canceled` → 前端表格删除
- 任何状态变更 → WebSocket推送更新

### 连接管理策略
1. **连接建立**：客户端连接时注册到对应策略的连接池
2. **连接维护**：定期发送心跳包检测连接状态
3. **连接清理**：自动清理断开的连接
4. **重连处理**：客户端重连时同步当前活跃订单状态

### 异常处理机制
- **网络异常**：WebSocket断开时缓存未发送的消息
- **数据同步**：客户端重连时发送完整的订单列表
- **内存管理**：限制缓存大小，超出限制时清理旧数据
- **错误恢复**：提供手动重置和数据修复接口

## 7. 性能优化

### 批量处理
- 将短时间内的多个订单更新合并为批量推送
- 使用防抖机制避免频繁的WebSocket消息发送

### 内存管理
- 定期清理历史订单缓存
- 使用LRU策略管理活跃订单缓存
- 限制每个策略的最大订单数量

### 网络优化
- 使用消息压缩减少带宽占用
- 实现增量更新减少数据传输量
- 支持断线重连和数据恢复

## 8. 部署和监控

### 监控指标
- WebSocket连接数量和状态
- 订单事件处理延迟
- 缓存命中率和内存使用量
- 消息发送成功率

### 日志记录
- 记录所有订单状态变更事件
- 记录WebSocket连接和断开事件
- 记录异常和错误信息用于调试

这个方案充分利用了现有的事件驱动架构，通过扩展事件类型和添加专门的WebSocket推送服务，实现了订单记录的实时展示需求，确保前端能够及时准确地反映回测过程中的订单状态变化。