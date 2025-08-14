# 策略统计模块 (Strategy Stats)

## 概述

策略统计模块是 Star River 量化交易系统中负责回测策略性能评估的核心组件。该模块通过监听虚拟交易系统事件，实时收集交易数据，并计算各种策略绩效指标。

## 核心功能

### 1. 回测策略统计 (BacktestStrategyStats)

#### 主要职责
- **策略配置管理**：管理策略ID、初始余额、杠杆等配置参数
- **事件监听**：异步监听虚拟交易系统产生的交易事件
- **数据收集**：收集订单、成交、仓位变化等交易数据
- **统计计算**：基于收集的数据计算策略绩效指标

#### 核心组件
```rust
pub struct BacktestStrategyStats {
    pub strategy_id: StrategyId,           // 策略唯一标识
    pub initial_balance: Balance,          // 初始资金
    pub leverage: Leverage,                // 杠杆倍数
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>, // 虚拟交易系统
    pub virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver, // 事件接收器
    pub strategy_stats_event_sender: StrategyStatsEventSender, // 统计事件发送器
    cancel_token: CancellationToken,       // 任务取消令牌
    play_index: i32,                      // 播放索引
}
```

### 2. 事件驱动架构

#### 事件监听机制
- 使用 `tokio::spawn` 创建异步任务监听虚拟交易系统事件
- 支持优雅关闭，通过 `CancellationToken` 实现任务取消
- 使用 `BroadcastStream` 处理多播事件流

#### 事件处理流程
1. **事件接收**：从虚拟交易系统接收交易事件
2. **事件解析**：解析不同类型的交易事件
3. **数据更新**：更新内部统计数据
4. **指标计算**：实时计算策略绩效指标
5. **结果推送**：向其他模块推送统计结果

### 3. 配置管理

#### 初始化配置
```rust
// 设置初始余额
pub fn set_initial_balance(&mut self, initial_balance: Balance)

// 设置杠杆倍数  
pub fn set_leverage(&mut self, leverage: Leverage)
```

#### 动态配置
- 支持运行时修改统计参数
- 支持策略配置的热更新

## 技术特性

### 异步并发
- 基于 Tokio 异步运行时
- 使用 Arc<RwLock<T>> 实现线程安全的数据共享
- 支持高并发事件处理

### 错误处理
- 完善的错误传播机制
- 详细的日志记录
- 优雅的错误恢复

### 性能优化
- 事件流式处理，避免数据积压
- 内存高效的数据结构设计
- 支持大规模回测数据处理

## 使用示例

```rust
// 创建策略统计实例
let stats = BacktestStrategyStats::new(
    strategy_id,
    virtual_trading_system,
    event_receiver,
    event_sender
);

// 配置初始参数
stats.set_initial_balance(10000.0);
stats.set_leverage(10);

// 启动事件监听
let stats_arc = Arc::new(RwLock::new(stats));
BacktestStrategyStats::start_listening(stats_arc).await?;
```

## 统计指标计算

基于 Python QuantStats 库的指标体系，该模块可以计算以下综合性能指标：

### 核心收益指标
- **总收益率 (Total Return)**：策略期间的累计收益率
- **年化收益率 (CAGR)**：复合年增长率，标准化的年度收益指标
- **累计收益 (Cumulative Returns)**：按时间序列的累计收益曲线
- **期间收益率 (Period Returns)**：各个时间段的收益率分布

### 风险调整收益指标
- **夏普比率 (Sharpe Ratio)**：超额收益与总风险的比值
- **索提诺比率 (Sortino Ratio)**：超额收益与下行风险的比值
- **卡尔马比率 (Calmar Ratio)**：年化收益率与最大回撤的比值
- **信息比率 (Information Ratio)**：超额收益与跟踪误差的比值
- **ω比率 (Omega Ratio)**：收益概率加权与损失概率加权的比值

### 风险指标
- **波动率 (Volatility)**：收益率的年化标准差
- **最大回撤 (Max Drawdown)**：策略期间的最大亏损幅度
- **回撤持续期 (Drawdown Duration)**：回撤恢复到前期高点的时间
- **VaR (Value at Risk)**：在给定置信水平下的最大可能损失
- **CVaR (Conditional VaR)**：超过VaR阈值的期望损失
- **下行波动率 (Downside Volatility)**：负收益的标准差
- **偏度 (Skewness)**：收益分布的偏斜程度
- **峰度 (Kurtosis)**：收益分布的尖峰程度

### 交易行为指标
- **总交易次数 (Total Trades)**：策略执行的总交易笔数
- **胜率 (Win Rate)**：盈利交易占总交易的比例
- **平均盈利 (Average Win)**：单笔盈利交易的平均收益
- **平均亏损 (Average Loss)**：单笔亏损交易的平均损失
- **盈亏比 (Profit Factor)**：总盈利与总亏损的比值
- **最大连续盈利 (Max Consecutive Wins)**：最大连续盈利交易数
- **最大连续亏损 (Max Consecutive Losses)**：最大连续亏损交易数
- **平均持仓时间 (Average Hold Time)**：单笔交易的平均持有时间

### 一致性指标
- **收益一致性 (Consistency)**：正收益期间占总期间的比例
- **月度胜率 (Monthly Win Rate)**：月度正收益的比例
- **年度胜率 (Yearly Win Rate)**：年度正收益的比例
- **R平方 (R-Squared)**：收益与基准的相关性
- **贝塔系数 (Beta)**：相对于基准的系统性风险
- **阿尔法系数 (Alpha)**：相对于基准的超额收益

### 尾部风险指标
- **预期不足 (Expected Shortfall)**：超过VaR的条件期望损失
- **尾部比率 (Tail Ratio)**：右尾与左尾的比值
- **最坏月份 (Worst Month)**：单月最大亏损
- **最佳月份 (Best Month)**：单月最大盈利
- **最坏年份 (Worst Year)**：单年最大亏损
- **最佳年份 (Best Year)**：单年最大盈利

### 基准对比指标
- **跟踪误差 (Tracking Error)**：与基准收益的标准差
- **向上捕获率 (Up Capture)**：基准上涨时的捕获比例
- **向下捕获率 (Down Capture)**：基准下跌时的下跌比例
- **相关系数 (Correlation)**：与基准的相关性

### 回撤分析指标
- **平均回撤 (Average Drawdown)**：所有回撤的平均值
- **回撤频率 (Drawdown Frequency)**：回撤发生的频率
- **恢复因子 (Recovery Factor)**：总收益与最大回撤的比值
- **Ulcer指数 (Ulcer Index)**：回撤深度和持续时间的综合指标

### 收益分布指标
- **收益分位数 (Return Quantiles)**：收益分布的各分位点
- **VaR置信区间 (VaR Confidence Intervals)**：不同置信水平的VaR值
- **收益直方图统计 (Return Distribution Stats)**：收益分布的详细统计
- **滚动统计 (Rolling Statistics)**：滚动窗口的各项指标

### 季节性分析
- **月度收益模式 (Monthly Returns Pattern)**：各月份的平均收益
- **季度收益分析 (Quarterly Analysis)**：各季度的绩效表现
- **工作日效应 (Weekday Effect)**：不同工作日的收益模式

## 集成接口

### 输入接口
- 虚拟交易系统事件流
- 策略配置参数
- 市场数据

### 输出接口  
- 实时统计数据
- 绩效评估报告
- 风险预警信号

## 指标实现架构

### 数据收集层
```rust
// 核心数据结构
pub struct PerformanceData {
    pub timestamps: Vec<i64>,
    pub returns: Vec<f64>,
    pub cumulative_returns: Vec<f64>,
    pub portfolio_values: Vec<f64>,
    pub trades: Vec<TradeRecord>,
    pub positions: Vec<PositionRecord>,
}
```

### 指标计算引擎
```rust
// 指标计算接口
pub trait MetricCalculator {
    fn calculate_sharpe_ratio(&self, returns: &[f64], risk_free_rate: f64) -> f64;
    fn calculate_max_drawdown(&self, cumulative_returns: &[f64]) -> f64;
    fn calculate_volatility(&self, returns: &[f64]) -> f64;
    fn calculate_var(&self, returns: &[f64], confidence_level: f64) -> f64;
    // ... 更多指标计算方法
}
```

### 实时指标更新
- **增量计算**：只计算新增数据，避免重复计算
- **滚动窗口**：支持滚动时间窗口的指标计算
- **缓存机制**：缓存计算结果，提高性能

## QuantStats 集成方案

### 数据格式转换
```rust
// 将 Rust 数据转换为 Python 兼容格式
pub fn convert_to_quantstats_format(data: &PerformanceData) -> PyResult<()> {
    // 时间序列数据转换
    // 收益率序列格式化
    // 调用 Python QuantStats 函数
}
```

### Python 桥接
- 使用 PyO3 实现 Rust-Python 互操作
- 支持调用 QuantStats 的所有指标计算函数
- 异步执行避免阻塞主线程

### 指标同步机制
- 定期同步 Rust 计算结果与 Python 计算结果
- 确保指标计算的一致性和准确性

## 扩展性

### 自定义指标
```rust
// 用户自定义指标接口
pub trait CustomMetric {
    fn name(&self) -> &str;
    fn calculate(&self, data: &PerformanceData) -> Result<f64, String>;
    fn requires_benchmark(&self) -> bool;
}
```

### 插件化架构
- 支持动态加载指标计算插件
- 提供标准化的指标注册机制
- 支持第三方指标库集成

### 多策略支持
- 支持同时监控和统计多个策略的性能表现
- 提供策略间的对比分析功能
- 支持组合策略的整体绩效评估

### 数据导出
- **JSON 格式**：结构化的指标数据导出
- **CSV 格式**：兼容 Excel 和数据分析工具
- **HTML 报告**：完整的可视化绩效报告
- **PDF 报告**：专业的投资组合分析报告

### 可视化集成
- 集成图表生成功能
- 支持多种图表类型（收益曲线、回撤图、分布图等）
- 提供交互式 Web 界面

## 注意事项

1. **内存管理**：长时间运行的回测需要注意内存使用量
2. **数据一致性**：确保统计数据与虚拟交易系统数据的一致性
3. **性能监控**：在处理大量事件时需要监控CPU和内存使用情况
4. **错误处理**：网络异常或数据异常时的错误恢复机制