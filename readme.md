# 节点
1. 价格速度节点
2. 波动率节点
3. 因子节点（节点打包）

# 数据库刷新表
sea-orm-cli migrate refresh -d ./migration

# 数据库生成实体
sea-orm-cli entity -d ./database


# python打包
"pyinstaller -c -F --clean --name MetaTrader5-x86_64-pc-windows-msvc --distpath resources/bin/windows scripts/metatrader5/main.py"



todo
1. 每个策略添加一个消息黑洞，用于接收前端接收不到的消息，避免节点报错。
2. 终止策略
3. 策略引擎重新加载策略
4. 节点message的消息黑洞，看是用一个结束节点还是用别的方式

1. 进入节点页面
2. 点击开始策略
3. 加载策略
4. 设置策略
5. 启动策略

我的功能已经完成的差不多了，现在进入到补充优化阶段。读取整个文件，包括strategy和live_data_node（其他的节点也会按照这个节点的实现逻辑，所以只需要优化一个节点即可）。进行代码逻辑漏洞检查和功能的优化补充。先进行文字分析，不要给代码。

todo:
节点健康检查：监督者模式
全局异常恢复机制设计方案
针对策略引擎中缺少全局异常恢复机制的问题，我建议实现以下设计方案：
监督者模式（Supervisor Pattern）
监督者模式是一种常见的错误处理策略，特别适合分布式系统和有多个组件的应用程序。
设计要点：
策略健康监控器：
创建一个专门的组件监控所有节点的健康状态
定期检查节点是否响应
捕获并集中处理节点抛出的异常
节点故障隔离：
当节点发生异常时，将其隔离，防止影响整个策略
根据节点重要性决定是否需要停止整个策略
恢复策略：
定义不同级别的恢复策略：
尝试重启节点
降级运行（跳过故障节点）
完全停止策略
状态持久化：
定期保存策略和节点的关键状态
在恢复时可以从最近的有效状态恢复

### 1. 创建StrategyHealthMonitor组件

```rust
pub struct StrategyHealthMonitor {
    strategy_id: i32,
    strategy_name: String,
    node_health_checks: HashMap<String, NodeHealthStatus>,
    recovery_policy: RecoveryPolicy,
    event_publisher: EventPublisher,
}

#[derive(Debug, Clone)]
pub enum NodeHealthStatus {
    Healthy,
    Degraded(String), // 包含警告信息
    Failed(String),   // 包含错误信息
}

#[derive(Debug, Clone)]
pub enum RecoveryPolicy {
    RestartNode,      // 尝试重启单个节点
    SkipNode,         // 跳过故障节点继续运行
    StopStrategy,     // 停止整个策略
}
```
