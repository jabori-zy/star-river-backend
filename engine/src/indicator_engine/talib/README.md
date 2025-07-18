# 技术指标计算系统优化

## 概述

本优化重构了原有的技术指标计算系统，通过引入元数据驱动的架构和宏系统，大大简化了新指标的添加过程，提高了代码的可维护性和扩展性。

## 优化前后对比

### 优化前
- 每个指标需要单独的实现文件
- 大量重复的unsafe代码
- 需要手动修改多个地方来添加新指标
- 错误处理分散且不统一
- lookback计算硬编码在match语句中

### 优化后
- 统一的指标元数据系统
- 自动代码生成减少重复
- 新指标只需一行宏调用
- 统一的错误处理机制
- 自动的指标注册和发现

## 核心组件

### 1. 指标元数据系统 (`indicator_meta.rs`)

```rust
pub struct IndicatorMeta {
    pub name: &'static str,
    pub lookback_fn: fn(&[IndicatorParam]) -> i32,
    pub calculate_fn: fn(&[f64], &[IndicatorParam]) -> Result<IndicatorOutput, TalibError>,
    pub output_format: OutputFormat,
}
```

### 2. 宏系统 (`indicator_macros.rs`)

提供了三个主要宏：
- `simple_indicator!`: 快速定义标准TA-Lib指标
- `define_indicator!`: 定义自定义指标
- `generate_single_output_calculator!` / `generate_triple_output_calculator!`: 生成计算函数

### 3. 指标注册表

自动管理所有指标的元数据，提供统一的查询和计算接口。

## 使用方法

### 添加标准TA-Lib指标

只需要一行代码：

```rust
simple_indicator!(RSI, single, RSI, CalculateRSIError, period);
```

这会自动生成：
- lookback计算函数
- 指标计算函数
- 错误处理
- 注册函数

### 添加自定义指标

```rust
pub fn my_indicator_calculate(data: &[f64], params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
    // 自定义计算逻辑
}

define_indicator!(MY_INDICATOR, {
    lookback: |params| { /* 计算lookback */ },
    calculate: my_indicator_calculate,
    output: Single,
});
```

### 使用指标

```rust
// 通用接口
let result = TALib::calculate_indicator("SMA", &data, &params)?;

// 或者使用便利函数
let sma_result = calculate_sma(&data, 20)?;
```

## 参数类型

支持的参数类型：
- `Period(i32)`: 周期参数
- `DevUp(f64)` / `DevDown(f64)`: 标准差倍数
- `MAType(i32)`: 移动平均类型
- `FastPeriod(i32)` / `SlowPeriod(i32)` / `SignalPeriod(i32)`: 多周期参数

## 输出格式

- `Single`: 单一数值序列 `Vec<f64>`
- `Triple`: 三重数值序列 `Vec<Vec<f64>>`（如MACD、布林带）

## 新增指标流程

1. 在 `indicators.rs` 中添加指标定义：
   ```rust
   simple_indicator!(NEW_INDICATOR, single, NEW_INDICATOR, GenericCalculationError, period);
   ```

2. 在 `init_indicators()` 函数中添加注册调用：
   ```rust
   register_new_indicator();
   ```

3. 如果需要，在 `IndicatorConfig` 枚举中添加配置类型

4. 完成！新指标自动支持所有通用接口

## 向后兼容性

系统保持了与现有代码的完全兼容：
- 原有的函数接口仍然可用
- 现有的调用代码无需修改
- 逐步迁移到新接口

## 性能优化

- 编译时代码生成，无运行时开销
- 统一的内存管理减少分配
- 类型安全的参数传递
- 优化的错误处理路径

## 扩展性

系统设计为高度可扩展：
- 支持任意数量的参数
- 支持自定义输出格式
- 支持组合指标
- 支持指标链式计算

## 示例

查看 `examples.rs` 文件了解更多使用示例，包括：
- 标准指标定义
- 自定义指标实现
- 组合指标创建
- 测试用例

## 测试

运行测试：
```bash
cargo test indicator_engine::talib
```

## 未来改进

- 支持多输入数据（OHLCV）
- 并行计算支持
- 缓存机制优化
- 更多标准指标支持
