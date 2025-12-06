# 量化交易系统计算公式

## 核心财务指标

### 1. Total Equity
```
Total Equity = Current Balance + Unrealized PnL
```

### 2. Available Balance
```
Available Balance = Current Balance - Used Margin - Frozen Funds
```

### 3. Net Value
```
Net Value = Total Equity / Initial Balance
```

### 4. Cumulative Return
```
Cumulative Return = (Total Equity - Initial Balance) / Initial Balance × 100%
```

## PnL Calculation

### 5. Realized PnL
```
Realized PnL = Current Balance - Initial Balance

Realized PnL = Σ(Close Price - Open Price) × Trade Quantity
```

### 6. Unrealized PnL
```
Unrealized PnL = Σ(Current Price - Entry Price) × Position Size

Long: (Current Price - Entry Price) × Position Size
Short: (Entry Price - Current Price) × Position Size
```

### 7. Total PnL
```
Total PnL = Realized PnL + Unrealized PnL
```

## Margin Calculation

### 8. Used Margin
```
Used Margin = Position Value / Leverage
```

### 9. Margin Ratio
```
Margin Ratio = Used Margin / Total Equity × 100%
```

### 10. Available Margin Ratio
```
Available Margin Ratio = Available Balance / Total Equity × 100%
```

## Risk Indicators

### 11. Drawdown
```
Current Drawdown = (Peak Net Value - Current Net Value) / Peak Net Value × 100%

Max Drawdown = max(Drawdown Series)
```

### 12. Underwater Duration
```
Underwater Duration = Current Time - Last Peak Time
```

## Trading Statistics

### 13. Win Rate
```
Win Rate = Profitable Trades / Total Trades × 100%
```

### 14. Profit Factor
```
Profit Factor = Total Profit / Total Loss
```

### 15. Average Win/Loss Ratio
```
Average Win/Loss Ratio = Average Profit / Average Loss
```

### 16. Sharpe Ratio
```
Sharpe Ratio = (Strategy Return - Risk-free Rate) / Strategy Return Std Dev
```

### 17. Max Consecutive Losses
```
Max Consecutive Losses = max(Consecutive Loss Count)
```

## Position Calculation

### 18. Position Value
```
Spot Position Value = Position Size × Current Price
Futures Position Value = Position Size × Contract Size × Current Price
```

### 19. Position Cost
```
Position Cost = Position Size × Average Entry Price
```

### 20. Position Return
```
Long Position Return = (Current Price - Entry Price) / Entry Price × 100%
Short Position Return = (Entry Price - Current Price) / Entry Price × 100%
```

## Capital Utilization

### 21. Capital Utilization
```
Capital Utilization = Used Margin / Total Equity × 100%
```

### 22. Leverage Utilization
```
Actual Leverage = Total Position Value / Total Equity
```

## Time Related Indicators

### 23. Annualized Return
```
Annualized Return = (Total Equity / Initial Balance)^(365/Days) - 1
```

### 24. Annualized Volatility
```
Annualized Volatility = Daily Return Std Dev × √252
```






