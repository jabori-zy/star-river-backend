# 量化交易系统计算公式

## 核心财务指标

### 1. 总权益 (Total Equity)
```
总权益 = 当前余额 + 未实现盈亏
Total Equity = Current Balance + Unrealized PnL
```

### 2. 可用余额 (Available Balance)
```
可用余额 = 当前余额 - 已用保证金 - 冻结资金
Available Balance = Current Balance - Used Margin - Frozen Funds
```

### 3. 净值 (Net Value)
```
净值 = 总权益 / 初始资金
Net Value = Total Equity / Initial Balance
```

### 4. 累计收益率 (Cumulative Return)
```
累计收益率 = (总权益 - 初始资金) / 初始资金 × 100%
Cumulative Return = (Total Equity - Initial Balance) / Initial Balance × 100%
```

## 盈亏计算

### 5. 已实现盈亏 (Realized PnL)
```
已实现盈亏 = 当前余额 - 初始资金
Realized PnL = Current Balance - Initial Balance

或者累计所有平仓交易的盈亏：
Realized PnL = Σ(平仓价格 - 开仓价格) × 交易数量
```

### 6. 未实现盈亏 (Unrealized PnL)
```
未实现盈亏 = Σ(当前价格 - 开仓价格) × 持仓数量
Unrealized PnL = Σ(Current Price - Entry Price) × Position Size

多头：(当前价格 - 开仓价格) × 持仓数量
空头：(开仓价格 - 当前价格) × 持仓数量
```

### 7. 总盈亏 (Total PnL)
```
总盈亏 = 已实现盈亏 + 未实现盈亏
Total PnL = Realized PnL + Unrealized PnL
```

## 保证金计算

### 8. 保证金占用 (Used Margin)
```
保证金占用 = 持仓价值 / 杠杆倍数
Used Margin = Position Value / Leverage

现货：保证金占用 = 持仓价值
期货：保证金占用 = 名义价值 / 杠杆倍数
```

### 9. 保证金率 (Margin Ratio)
```
保证金率 = 已用保证金 / 总权益 × 100%
Margin Ratio = Used Margin / Total Equity × 100%
```

### 10. 可用保证金率 (Available Margin Ratio)
```
可用保证金率 = 可用余额 / 总权益 × 100%
Available Margin Ratio = Available Balance / Total Equity × 100%
```

## 风险指标

### 11. 回撤 (Drawdown)
```
当前回撤 = (历史最高净值 - 当前净值) / 历史最高净值 × 100%
Current Drawdown = (Peak Net Value - Current Net Value) / Peak Net Value × 100%

最大回撤 = max(回撤序列)
Max Drawdown = max(Drawdown Series)
```

### 12. 水下时间 (Underwater Duration)
```
水下时间 = 当前时间 - 上次创新高时间
Underwater Duration = Current Time - Last Peak Time
```

## 交易统计

### 13. 胜率 (Win Rate)
```
胜率 = 盈利交易次数 / 总交易次数 × 100%
Win Rate = Profitable Trades / Total Trades × 100%
```

### 14. 盈利因子 (Profit Factor)
```
盈利因子 = 总盈利金额 / 总亏损金额
Profit Factor = Total Profit / Total Loss
```

### 15. 平均盈亏比 (Average Win/Loss Ratio)
```
平均盈亏比 = 平均盈利金额 / 平均亏损金额
Average Win/Loss Ratio = Average Profit / Average Loss
```

### 16. 夏普比率 (Sharpe Ratio)
```
夏普比率 = (策略收益率 - 无风险收益率) / 策略收益率标准差
Sharpe Ratio = (Strategy Return - Risk-free Rate) / Strategy Return Std Dev
```

### 17. 最大连续亏损 (Max Consecutive Losses)
```
最大连续亏损 = max(连续亏损交易次数)
Max Consecutive Losses = max(Consecutive Loss Count)
```

## 持仓计算

### 18. 持仓价值 (Position Value)
```
现货持仓价值 = 持仓数量 × 当前价格
Spot Position Value = Position Size × Current Price

期货持仓价值 = 持仓数量 × 合约面值 × 当前价格
Futures Position Value = Position Size × Contract Size × Current Price
```

### 19. 持仓成本 (Position Cost)
```
持仓成本 = 持仓数量 × 平均开仓价格
Position Cost = Position Size × Average Entry Price
```

### 20. 持仓盈亏率 (Position Return)
```
多头盈亏率 = (当前价格 - 开仓价格) / 开仓价格 × 100%
Long Position Return = (Current Price - Entry Price) / Entry Price × 100%

空头盈亏率 = (开仓价格 - 当前价格) / 开仓价格 × 100%
Short Position Return = (Entry Price - Current Price) / Entry Price × 100%
```

## 资金利用率

### 21. 资金利用率 (Capital Utilization)
```
资金利用率 = 已用保证金 / 总权益 × 100%
Capital Utilization = Used Margin / Total Equity × 100%
```

### 22. 杠杆利用率 (Leverage Utilization)
```
实际杠杆 = 持仓总价值 / 总权益
Actual Leverage = Total Position Value / Total Equity
```

## 时间相关指标

### 23. 年化收益率 (Annualized Return)
```
年化收益率 = (总权益 / 初始资金)^(365/天数) - 1
Annualized Return = (Total Equity / Initial Balance)^(365/Days) - 1
```

### 24. 年化波动率 (Annualized Volatility)
```
年化波动率 = 日收益率标准差 × √252
Annualized Volatility = Daily Return Std Dev × √252
```

## 注意事项

1. **时间戳一致性**：所有计算都应基于相同的时间点
2. **价格数据准确性**：确保使用最新的市场价格
3. **费用考虑**：实际交易中需要扣除手续费、滑点等成本
4. **杠杆风险**：高杠杆交易需要特别注意保证金维持率
5. **数据精度**：财务计算建议使用高精度数值类型


保证金： 持仓量 * 开仓价格 / 杠杆倍数
例如：0.1个btc * 10000usdt / 10 = 1000usdt  需要的保证金为1000usdt
意味着，杠杆倍数越大，需要的保证金越少


收益额 = （当前价格  - 开仓价格） * 持仓量
收益率 = 收益额 / 保证金 * 100%
强平价格 = 开仓价格 - 保证金/持仓量






