# TA-Lib 指标函数签名

## 指标函数列表
✅️
1. **TA_ACCBANDS**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outRealUpperBand: f64, outRealMiddleBand: f64, outRealLowerBand: f64) -> TA_RetCode
   // Acceleration Bands - 加速布林带

<!-- 2. **TA_ACOS**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
   // Vector Trigonometric ACos - 反余弦函数(不要) -->
✅️
3. **TA_AD**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, inVolume: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
   // Chaikin A/D Line - 蔡金累积/派发线

4. **TA_ADD**(startIdx: i32, endIdx: i32, inReal0: f64, inReal1: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
   // Vector Arithmetic Add - 向量加法(不要)
✅️
5. **TA_ADOSC**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, inVolume: f64, optInFastPeriod: i32, optInSlowPeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
   // Chaikin A/D Oscillator - 蔡金振荡器
✅️
6. **TA_ADX**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
   // Average Directional Movement Index - 平均趋向指数
✅️
7. **TA_ADXR**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
   // Average Directional Movement Index Rating - 平均趋向指数评级
✅️
8. **TA_APO**(startIdx: i32, endIdx: i32, inReal: f64, optInFastPeriod: i32, optInSlowPeriod: i32, optInMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
   // Absolute Price Oscillator - 绝对价格振荡器
✅️
9. **TA_AROON**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outAroonDown: f64, outAroonUp: f64) -> TA_RetCode
   // Aroon - 阿隆指标
✅️
10. **TA_AROONOSC**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Aroon Oscillator - 阿隆振荡器

<!-- 11. **TA_ASIN**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric ASin - 反正弦函数(不要) -->

<!-- 12. **TA_ATAN**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric ATan - 反正切函数(不要) -->
✅️
13. **TA_ATR**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Average True Range - 平均真实波幅

<!-- 14. **TA_AVGPRICE**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Average Price - 平均价格(不要) -->

<!-- 15. **TA_AVGDEV**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Average Deviation - 平均偏差(不要) -->
✅️
16. **TA_BBANDS**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, optInNbDevUp: f64, optInNbDevDn: f64, optInMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outRealUpperBand: f64, outRealMiddleBand: f64, outRealLowerBand: f64) -> TA_RetCode
    // Bollinger Bands - 布林带

<!-- 17. **TA_BETA**(startIdx: i32, endIdx: i32, inReal0: f64, inReal1: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Beta - 贝塔系数 -->
✅️
18. **TA_BOP**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Balance Of Power - 均势指标
✅️
19. **TA_CCI**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Commodity Channel Index - 商品通道指数

## 蜡烛图形态识别函数
✅️
20. **TA_CDL2CROWS**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Two Crows - 两只乌鸦
✅️
21. **TA_CDL3BLACKCROWS**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Black Crows - 三只黑乌鸦
✅️
22. **TA_CDL3INSIDE**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Inside Up/Down - 三内部上升/下降
✅️
23. **TA_CDL3LINESTRIKE**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three-Line Strike - 三线打击
✅️
24. **TA_CDL3OUTSIDE**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Outside Up/Down - 三外部上升/下降
✅️
25. **TA_CDL3STARSINSOUTH**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Stars In The South - 南方三星
✅️
26. **TA_CDL3WHITESOLDIERS**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Advancing White Soldiers - 三个白兵
✅️
27. **TA_CDLABANDONEDBABY**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, optInPenetration: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Abandoned Baby - 弃婴

28. **TA_CDLADVANCEBLOCK**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Advance Block - 大敌当前

29. **TA_CDLBELTHOLD**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Belt-hold - 捉腰带线

30. **TA_CDLBREAKAWAY**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Breakaway - 脱离

31. **TA_CDLCLOSINGMARUBOZU**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Closing Marubozu - 收盘缺影线

32. **TA_CDLCONCEALBABYSWALL**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Concealing Baby Swallow - 藏婴吞没

33. **TA_CDLCOUNTERATTACK**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Counterattack - 反击线

34. **TA_CDLDARKCLOUDCOVER**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, optInPenetration: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Dark Cloud Cover - 乌云盖顶

35. **TA_CDLDOJI**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Doji - 十字星

36. **TA_CDLDOJISTAR**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Doji Star - 十字星

37. **TA_CDLDRAGONFLYDOJI**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Dragonfly Doji - 蜻蜓十字星

38. **TA_CDLENGULFING**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Engulfing Pattern - 吞没形态

39. **TA_CDLEVENINGDOJISTAR**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, optInPenetration: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Evening Doji Star - 黄昏十字星

40. **TA_CDLEVENINGSTAR**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, optInPenetration: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Evening Star - 黄昏之星

41. **TA_CDLGAPSIDESIDEWHITE**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Up/Down-gap side-by-side white lines - 向上/向下跳空并列白线

42. **TA_CDLGRAVESTONEDOJI**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Gravestone Doji - 墓碑十字星

43. **TA_CDLHAMMER**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Hammer - 锤头线

44. **TA_CDLHANGINGMAN**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Hanging Man - 上吊线

45. **TA_CDLHARAMI**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Harami Pattern - 孕线形态

46. **TA_CDLHARAMICROSS**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Harami Cross Pattern - 十字孕线形态

47. **TA_CDLHIGHWAVE**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // High-Wave Candle - 长影线

48. **TA_CDLHIKKAKE**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Hikkake Pattern - 陷阱形态

49. **TA_CDLHIKKAKEMOD**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Modified Hikkake Pattern - 修正陷阱形态

50. **TA_CDLHOMINGPIGEON**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Homing Pigeon - 家鸽形态

## 数学运算函数

<!-- 51. **TA_CEIL**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Ceiling - 向上取整

52. **TA_COS**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric Cos - 余弦函数 -->

<!-- 53. **TA_COSH**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric Cosh - 双曲余弦函数

54. **TA_DIV**(startIdx: i32, endIdx: i32, inReal0: f64, inReal1: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Arithmetic Div - 向量除法

55. **TA_EXP**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Arithmetic Exp - 指数函数

56. **TA_FLOOR**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Floor - 向下取整

57. **TA_LN**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Log Natural - 自然对数

58. **TA_LOG10**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Log10 - 常用对数 -->

<!-- 59. **TA_MAX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Highest value over a specified period - 指定周期内最高值

60. **TA_MAXINDEX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Index of highest value over a specified period - 指定周期内最高值的索引

61. **TA_MIN**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Lowest value over a specified period - 指定周期内最低值

62. **TA_MININDEX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Index of lowest value over a specified period - 指定周期内最低值的索引

63. **TA_MINMAX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outMin: f64, outMax: f64) -> TA_RetCode
    // Lowest and highest values over a specified period - 指定周期内最低值和最高值

64. **TA_MINMAXINDEX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outMinIdx: i32, outMaxIdx: i32) -> TA_RetCode
    // Indexes of lowest and highest values over a specified period - 指定周期内最低值和最高值的索引

65. **TA_MULT**(startIdx: i32, endIdx: i32, inReal0: f64, inReal1: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Arithmetic Mult - 向量乘法

66. **TA_SIN**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric Sin - 正弦函数

67. **TA_SINH**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric Sinh - 双曲正弦函数

68. **TA_SQRT**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Square Root - 平方根

69. **TA_SUB**(startIdx: i32, endIdx: i32, inReal0: f64, inReal1: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Arithmetic Subtraction - 向量减法

70. **TA_SUM**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Summation - 求和

71. **TA_TAN**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric Tan - 正切函数

72. **TA_TANH**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric Tanh - 双曲正切函数 -->

## 移动平均线函数

<!-- 73. **TA_DEMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Double Exponential Moving Average - 双指数移动平均线 -->

<!-- 74. **TA_EMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Exponential Moving Average - 指数移动平均线 -->

75. **TA_HT_TRENDLINE**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Hilbert Transform - Instantaneous Trendline - 希尔伯特变换瞬时趋势线

<!-- 76. **TA_KAMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Kaufman Adaptive Moving Average - 考夫曼自适应移动平均线 -->

77. **TA_MA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, optInMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Moving average - 移动平均线

<!-- 78. **TA_MAMA**(startIdx: i32, endIdx: i32, inReal: f64, optInFastLimit: f64, optInSlowLimit: f64, outBegIdx: i32, outNBElement: i32, outMAMA: f64, outFAMA: f64) -> TA_RetCode
    // MESA Adaptive Moving Average - MESA自适应移动平均线 -->

79. **TA_MAVP**(startIdx: i32, endIdx: i32, inReal: f64, inPeriods: f64, optInMinPeriod: i32, optInMaxPeriod: i32, optInMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Moving average with variable period - 可变周期移动平均线

<!-- 80. **TA_SMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Simple Moving Average - 简单移动平均线 -->

<!-- 81. **TA_T3**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, optInVFactor: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Triple Exponential Moving Average (T3) - 三重指数移动平均线 -->

<!-- 82. **TA_TEMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Triple Exponential Moving Average - 三重指数移动平均线 -->

<!-- 83. **TA_TRIMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Triangular Moving Average - 三角移动平均线 -->

<!-- 84. **TA_WMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Weighted Moving Average - 加权移动平均线 -->

## 动量指标函数

85. **TA_MACD**(startIdx: i32, endIdx: i32, inReal: f64, optInFastPeriod: i32, optInSlowPeriod: i32, optInSignalPeriod: i32, outBegIdx: i32, outNBElement: i32, outMACD: f64, outMACDSignal: f64, outMACDHist: f64) -> TA_RetCode
    // Moving Average Convergence/Divergence - MACD指标

86. **TA_MACDEXT**(startIdx: i32, endIdx: i32, inReal: f64, optInFastPeriod: i32, optInFastMAType: TA_MAType, optInSlowPeriod: i32, optInSlowMAType: TA_MAType, optInSignalPeriod: i32, optInSignalMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outMACD: f64, outMACDSignal: f64, outMACDHist: f64) -> TA_RetCode
    // MACD with controllable MA type - 可控制MA类型的MACD

87. **TA_MACDFIX**(startIdx: i32, endIdx: i32, inReal: f64, optInSignalPeriod: i32, outBegIdx: i32, outNBElement: i32, outMACD: f64, outMACDSignal: f64, outMACDHist: f64) -> TA_RetCode
    // Moving Average Convergence/Divergence Fix 12/26 - 固定12/26的MACD

88. **TA_MOM**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Momentum - 动量指标

89. **TA_PPO**(startIdx: i32, endIdx: i32, inReal: f64, optInFastPeriod: i32, optInSlowPeriod: i32, optInMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Percentage Price Oscillator - 价格百分比振荡器

90. **TA_ROC**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Rate of change : ((price/prevPrice)-1)*100 - 变化率

91. **TA_ROCP**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Rate of change Percentage: (price-prevPrice)/prevPrice - 变化率百分比

92. **TA_ROCR**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Rate of change ratio: (price/prevPrice) - 变化率比率

93. **TA_ROCR100**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Rate of change ratio 100 scale: (price/prevPrice)*100 - 变化率比率100倍

94. **TA_RSI**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Relative Strength Index - 相对强弱指数

95. **TA_TRIX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // 1-day Rate-Of-Change (ROC) of a Triple Smooth EMA - 三重平滑EMA的1日变化率

## 价格指标函数

96. **TA_MEDPRICE**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Median Price - 中位价

97. **TA_MIDPOINT**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // MidPoint over period - 周期中点

98. **TA_MIDPRICE**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Midpoint Price over period - 周期中点价格

99. **TA_TYPPRICE**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Typical Price - 典型价格

100. **TA_WCLPRICE**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Weighted Close Price - 加权收盘价

## 波动率指标函数

101. **TA_NATR**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Normalized Average True Range - 标准化平均真实波幅

102. **TA_STDDEV**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, optInNbDev: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Standard Deviation - 标准偏差

103. **TA_TRANGE**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // True Range - 真实波幅

104. **TA_VAR**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, optInNbDev: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Variance - 方差

## 成交量指标函数

105. **TA_MFI**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, inVolume: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Money Flow Index - 资金流量指数

106. **TA_OBV**(startIdx: i32, endIdx: i32, inReal: f64, inVolume: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // On Balance Volume - 能量潮

## 其他重要指标

107. **TA_SAR**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, optInAcceleration: f64, optInMaximum: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Parabolic SAR - 抛物线SAR

108. **TA_STOCH**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInFastK_Period: i32, optInSlowK_Period: i32, optInSlowK_MAType: TA_MAType, optInSlowD_Period: i32, optInSlowD_MAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outSlowK: f64, outSlowD: f64) -> TA_RetCode
     // Stochastic - 随机指标

109. **TA_STOCHF**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInFastK_Period: i32, optInFastD_Period: i32, optInFastD_MAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outFastK: f64, outFastD: f64) -> TA_RetCode
     // Stochastic Fast - 快速随机指标

110. **TA_STOCHRSI**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, optInFastK_Period: i32, optInFastD_Period: i32, optInFastD_MAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outFastK: f64, outFastD: f64) -> TA_RetCode
     // Stochastic Relative Strength Index - 随机相对强弱指数

111. **TA_ULTOSC**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInTimePeriod1: i32, optInTimePeriod2: i32, optInTimePeriod3: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Ultimate Oscillator - 终极振荡器

112. **TA_WILLR**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Williams' %R - 威廉指标

---

## 说明

- **参数类型映射**：
  - `::core::ffi::c_int` → `i32`
  - `*const f64` → `f64` (输入参数)
  - `*mut f64` → `f64` (输出参数)
  - `*mut ::core::ffi::c_int` → `i32` (输出参数)

- **函数命名规则**：
  - 只包含 `TA_` 开头的双精度版本
  - 已移除所有 `TA_S_` 开头的单精度版本
  - 忽略了所有 `_Lookback` 后缀的函数

- **返回值**：所有函数都返回 `TA_RetCode` 类型，表示操作结果状态

- **常用参数**：
  - `startIdx`, `endIdx`: 计算的起始和结束索引
  - `optInTimePeriod`: 时间周期参数
  - `outBegIdx`, `outNBElement`: 输出数据的起始索引和元素数量

- **函数分类**：
  - **基础技术指标**: ACCBANDS, ACOS, AD, ADD, ADOSC, ADX, ADXR, APO, AROON, AROONOSC, ASIN, ATAN, ATR, AVGPRICE, AVGDEV, BBANDS, BETA, BOP, CCI
  - **蜡烛图形态识别**: CDL系列函数，用于识别各种K线形态
  - **数学运算函数**: CEIL, COS, COSH, DIV, EXP, FLOOR, LN, LOG10, MAX, MIN, MULT, SIN, SINH, SQRT, SUB, SUM, TAN, TANH等
  - **移动平均线**: DEMA, EMA, HT_TRENDLINE, KAMA, MA, MAMA, MAVP, SMA, T3, TEMA, TRIMA, WMA
  - **动量指标**: MACD, MACDEXT, MACDFIX, MOM, PPO, ROC, ROCP, ROCR, ROCR100, RSI, TRIX
  - **价格指标**: MEDPRICE, MIDPOINT, MIDPRICE, TYPPRICE, WCLPRICE
  - **波动率指标**: NATR, STDDEV, TRANGE, VAR
  - **成交量指标**: MFI, OBV
  - **其他重要指标**: SAR, STOCH, STOCHF, STOCHRSI, ULTOSC, WILLR

## 蜡烛图形态识别函数

39. **TA_CDL2CROWS**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Two Crows - 两只乌鸦

40. **TA_S_CDL2CROWS**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Two Crows (Single Precision) - 两只乌鸦（单精度）

41. **TA_CDL3BLACKCROWS**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Black Crows - 三只黑乌鸦

42. **TA_S_CDL3BLACKCROWS**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Black Crows (Single Precision) - 三只黑乌鸦（单精度）

43. **TA_CDL3INSIDE**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Inside Up/Down - 三内部上升/下降

44. **TA_S_CDL3INSIDE**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Inside Up/Down (Single Precision) - 三内部上升/下降（单精度）

45. **TA_CDL3LINESTRIKE**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three-Line Strike - 三线打击

46. **TA_S_CDL3LINESTRIKE**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three-Line Strike (Single Precision) - 三线打击（单精度）

47. **TA_CDL3OUTSIDE**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Outside Up/Down - 三外部上升/下降

48. **TA_S_CDL3OUTSIDE**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Outside Up/Down (Single Precision) - 三外部上升/下降（单精度）

49. **TA_CDL3STARSINSOUTH**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Stars In The South - 南方三星

50. **TA_S_CDL3STARSINSOUTH**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Stars In The South (Single Precision) - 南方三星（单精度）

51. **TA_CDL3WHITESOLDIERS**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Advancing White Soldiers - 三个白兵

52. **TA_S_CDL3WHITESOLDIERS**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Three Advancing White Soldiers (Single Precision) - 三个白兵（单精度）

53. **TA_CDLABANDONEDBABY**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, optInPenetration: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Abandoned Baby - 弃婴

54. **TA_S_CDLABANDONEDBABY**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, optInPenetration: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Abandoned Baby (Single Precision) - 弃婴（单精度）

55. **TA_CDLADVANCEBLOCK**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Advance Block - 大敌当前

56. **TA_S_CDLADVANCEBLOCK**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Advance Block (Single Precision) - 大敌当前（单精度）

57. **TA_CDLBELTHOLD**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Belt-hold - 捉腰带线

58. **TA_S_CDLBELTHOLD**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Belt-hold (Single Precision) - 捉腰带线（单精度）

59. **TA_CDLBREAKAWAY**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Breakaway - 脱离

60. **TA_S_CDLBREAKAWAY**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Breakaway (Single Precision) - 脱离（单精度）

61. **TA_CDLCLOSINGMARUBOZU**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Closing Marubozu - 收盘缺影线

62. **TA_S_CDLCLOSINGMARUBOZU**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Closing Marubozu (Single Precision) - 收盘缺影线（单精度）

63. **TA_CDLCONCEALBABYSWALL**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Concealing Baby Swallow - 藏婴吞没

64. **TA_S_CDLCONCEALBABYSWALL**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Concealing Baby Swallow (Single Precision) - 藏婴吞没（单精度）

65. **TA_CDLCOUNTERATTACK**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Counterattack - 反击线

66. **TA_S_CDLCOUNTERATTACK**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Counterattack (Single Precision) - 反击线（单精度）

67. **TA_CDLDARKCLOUDCOVER**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, optInPenetration: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Dark Cloud Cover - 乌云盖顶

68. **TA_S_CDLDARKCLOUDCOVER**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, optInPenetration: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Dark Cloud Cover (Single Precision) - 乌云盖顶（单精度）

69. **TA_CDLDOJI**(startIdx: i32, endIdx: i32, inOpen: f64, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Doji - 十字星

70. **TA_S_CDLDOJI**(startIdx: i32, endIdx: i32, inOpen: f32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Doji (Single Precision) - 十字星（单精度）

## 数学运算函数

71. **TA_CEIL**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Ceiling - 向上取整

72. **TA_S_CEIL**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Ceiling (Single Precision) - 向上取整（单精度）

73. **TA_COS**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric Cos - 余弦函数

74. **TA_S_COS**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric Cos (Single Precision) - 余弦函数（单精度）

75. **TA_COSH**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric Cosh - 双曲余弦函数

76. **TA_S_COSH**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Trigonometric Cosh (Single Precision) - 双曲余弦函数（单精度）

77. **TA_DIV**(startIdx: i32, endIdx: i32, inReal0: f64, inReal1: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Arithmetic Div - 向量除法

78. **TA_S_DIV**(startIdx: i32, endIdx: i32, inReal0: f32, inReal1: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Arithmetic Div (Single Precision) - 向量除法（单精度）

79. **TA_EXP**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Arithmetic Exp - 指数函数

80. **TA_S_EXP**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Arithmetic Exp (Single Precision) - 指数函数（单精度）

81. **TA_FLOOR**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Floor - 向下取整

82. **TA_S_FLOOR**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Floor (Single Precision) - 向下取整（单精度）

83. **TA_LN**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Log Natural - 自然对数

84. **TA_S_LN**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Log Natural (Single Precision) - 自然对数（单精度）

85. **TA_LOG10**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Log10 - 常用对数

86. **TA_S_LOG10**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Log10 (Single Precision) - 常用对数（单精度）

87. **TA_MAX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Highest value over a specified period - 指定周期内最高值

88. **TA_S_MAX**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Highest value over a specified period (Single Precision) - 指定周期内最高值（单精度）

89. **TA_MAXINDEX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Index of highest value over a specified period - 指定周期内最高值的索引

90. **TA_S_MAXINDEX**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Index of highest value over a specified period (Single Precision) - 指定周期内最高值的索引（单精度）

91. **TA_MIN**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Lowest value over a specified period - 指定周期内最低值

92. **TA_S_MIN**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Lowest value over a specified period (Single Precision) - 指定周期内最低值（单精度）

93. **TA_MININDEX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Index of lowest value over a specified period - 指定周期内最低值的索引

94. **TA_S_MININDEX**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outInteger: i32) -> TA_RetCode
    // Index of lowest value over a specified period (Single Precision) - 指定周期内最低值的索引（单精度）

95. **TA_MINMAX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outMin: f64, outMax: f64) -> TA_RetCode
    // Lowest and highest values over a specified period - 指定周期内最低值和最高值

96. **TA_S_MINMAX**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outMin: f64, outMax: f64) -> TA_RetCode
    // Lowest and highest values over a specified period (Single Precision) - 指定周期内最低值和最高值（单精度）

97. **TA_MINMAXINDEX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outMinIdx: i32, outMaxIdx: i32) -> TA_RetCode
    // Indexes of lowest and highest values over a specified period - 指定周期内最低值和最高值的索引

98. **TA_S_MINMAXINDEX**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outMinIdx: i32, outMaxIdx: i32) -> TA_RetCode
    // Indexes of lowest and highest values over a specified period (Single Precision) - 指定周期内最低值和最高值的索引（单精度）

99. **TA_MULT**(startIdx: i32, endIdx: i32, inReal0: f64, inReal1: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
    // Vector Arithmetic Mult - 向量乘法

100. **TA_S_MULT**(startIdx: i32, endIdx: i32, inReal0: f32, inReal1: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Arithmetic Mult (Single Precision) - 向量乘法（单精度）

101. **TA_SIN**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Trigonometric Sin - 正弦函数

102. **TA_S_SIN**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Trigonometric Sin (Single Precision) - 正弦函数（单精度）

103. **TA_SINH**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Trigonometric Sinh - 双曲正弦函数

104. **TA_S_SINH**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Trigonometric Sinh (Single Precision) - 双曲正弦函数（单精度）

105. **TA_SQRT**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Square Root - 平方根

106. **TA_S_SQRT**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Square Root (Single Precision) - 平方根（单精度）

107. **TA_SUB**(startIdx: i32, endIdx: i32, inReal0: f64, inReal1: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Arithmetic Subtraction - 向量减法

108. **TA_S_SUB**(startIdx: i32, endIdx: i32, inReal0: f32, inReal1: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Arithmetic Subtraction (Single Precision) - 向量减法（单精度）

109. **TA_SUM**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Summation - 求和

110. **TA_S_SUM**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Summation (Single Precision) - 求和（单精度）

111. **TA_TAN**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Trigonometric Tan - 正切函数

112. **TA_S_TAN**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Trigonometric Tan (Single Precision) - 正切函数（单精度）

113. **TA_TANH**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Trigonometric Tanh - 双曲正切函数

114. **TA_S_TANH**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Vector Trigonometric Tanh (Single Precision) - 双曲正切函数（单精度）

## 移动平均线函数

115. **TA_DEMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Double Exponential Moving Average - 双指数移动平均线

116. **TA_S_DEMA**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Double Exponential Moving Average (Single Precision) - 双指数移动平均线（单精度）

117. **TA_EMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Exponential Moving Average - 指数移动平均线

118. **TA_S_EMA**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Exponential Moving Average (Single Precision) - 指数移动平均线（单精度）

119. **TA_HT_TRENDLINE**(startIdx: i32, endIdx: i32, inReal: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Hilbert Transform - Instantaneous Trendline - 希尔伯特变换瞬时趋势线

120. **TA_S_HT_TRENDLINE**(startIdx: i32, endIdx: i32, inReal: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Hilbert Transform - Instantaneous Trendline (Single Precision) - 希尔伯特变换瞬时趋势线（单精度）

121. **TA_KAMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Kaufman Adaptive Moving Average - 考夫曼自适应移动平均线

122. **TA_S_KAMA**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Kaufman Adaptive Moving Average (Single Precision) - 考夫曼自适应移动平均线（单精度）

123. **TA_MA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, optInMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Moving average - 移动平均线

124. **TA_S_MA**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, optInMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Moving average (Single Precision) - 移动平均线（单精度）

125. **TA_MAMA**(startIdx: i32, endIdx: i32, inReal: f64, optInFastLimit: f64, optInSlowLimit: f64, outBegIdx: i32, outNBElement: i32, outMAMA: f64, outFAMA: f64) -> TA_RetCode
     // MESA Adaptive Moving Average - MESA自适应移动平均线

126. **TA_S_MAMA**(startIdx: i32, endIdx: i32, inReal: f32, optInFastLimit: f64, optInSlowLimit: f64, outBegIdx: i32, outNBElement: i32, outMAMA: f64, outFAMA: f64) -> TA_RetCode
     // MESA Adaptive Moving Average (Single Precision) - MESA自适应移动平均线（单精度）

127. **TA_MAVP**(startIdx: i32, endIdx: i32, inReal: f64, inPeriods: f64, optInMinPeriod: i32, optInMaxPeriod: i32, optInMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Moving average with variable period - 可变周期移动平均线

128. **TA_S_MAVP**(startIdx: i32, endIdx: i32, inReal: f32, inPeriods: f32, optInMinPeriod: i32, optInMaxPeriod: i32, optInMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Moving average with variable period (Single Precision) - 可变周期移动平均线（单精度）

129. **TA_SMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Simple Moving Average - 简单移动平均线

130. **TA_S_SMA**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Simple Moving Average (Single Precision) - 简单移动平均线（单精度）

131. **TA_T3**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, optInVFactor: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Triple Exponential Moving Average (T3) - 三重指数移动平均线

132. **TA_S_T3**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, optInVFactor: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Triple Exponential Moving Average (T3) (Single Precision) - 三重指数移动平均线（单精度）

133. **TA_TEMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Triple Exponential Moving Average - 三重指数移动平均线

134. **TA_S_TEMA**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Triple Exponential Moving Average (Single Precision) - 三重指数移动平均线（单精度）

135. **TA_TRIMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Triangular Moving Average - 三角移动平均线

136. **TA_S_TRIMA**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Triangular Moving Average (Single Precision) - 三角移动平均线（单精度）

137. **TA_WMA**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Weighted Moving Average - 加权移动平均线

138. **TA_S_WMA**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Weighted Moving Average (Single Precision) - 加权移动平均线（单精度）

## 动量指标函数

139. **TA_MACD**(startIdx: i32, endIdx: i32, inReal: f64, optInFastPeriod: i32, optInSlowPeriod: i32, optInSignalPeriod: i32, outBegIdx: i32, outNBElement: i32, outMACD: f64, outMACDSignal: f64, outMACDHist: f64) -> TA_RetCode
     // Moving Average Convergence/Divergence - MACD指标

140. **TA_S_MACD**(startIdx: i32, endIdx: i32, inReal: f32, optInFastPeriod: i32, optInSlowPeriod: i32, optInSignalPeriod: i32, outBegIdx: i32, outNBElement: i32, outMACD: f64, outMACDSignal: f64, outMACDHist: f64) -> TA_RetCode
     // Moving Average Convergence/Divergence (Single Precision) - MACD指标（单精度）

141. **TA_MACDEXT**(startIdx: i32, endIdx: i32, inReal: f64, optInFastPeriod: i32, optInFastMAType: TA_MAType, optInSlowPeriod: i32, optInSlowMAType: TA_MAType, optInSignalPeriod: i32, optInSignalMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outMACD: f64, outMACDSignal: f64, outMACDHist: f64) -> TA_RetCode
     // MACD with controllable MA type - 可控制MA类型的MACD

142. **TA_S_MACDEXT**(startIdx: i32, endIdx: i32, inReal: f32, optInFastPeriod: i32, optInFastMAType: TA_MAType, optInSlowPeriod: i32, optInSlowMAType: TA_MAType, optInSignalPeriod: i32, optInSignalMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outMACD: f64, outMACDSignal: f64, outMACDHist: f64) -> TA_RetCode
     // MACD with controllable MA type (Single Precision) - 可控制MA类型的MACD（单精度）

143. **TA_MACDFIX**(startIdx: i32, endIdx: i32, inReal: f64, optInSignalPeriod: i32, outBegIdx: i32, outNBElement: i32, outMACD: f64, outMACDSignal: f64, outMACDHist: f64) -> TA_RetCode
     // Moving Average Convergence/Divergence Fix 12/26 - 固定12/26的MACD

144. **TA_S_MACDFIX**(startIdx: i32, endIdx: i32, inReal: f32, optInSignalPeriod: i32, outBegIdx: i32, outNBElement: i32, outMACD: f64, outMACDSignal: f64, outMACDHist: f64) -> TA_RetCode
     // Moving Average Convergence/Divergence Fix 12/26 (Single Precision) - 固定12/26的MACD（单精度）

145. **TA_MOM**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Momentum - 动量指标

146. **TA_S_MOM**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Momentum (Single Precision) - 动量指标（单精度）

147. **TA_PPO**(startIdx: i32, endIdx: i32, inReal: f64, optInFastPeriod: i32, optInSlowPeriod: i32, optInMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Percentage Price Oscillator - 价格百分比振荡器

148. **TA_S_PPO**(startIdx: i32, endIdx: i32, inReal: f32, optInFastPeriod: i32, optInSlowPeriod: i32, optInMAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Percentage Price Oscillator (Single Precision) - 价格百分比振荡器（单精度）

149. **TA_ROC**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Rate of change : ((price/prevPrice)-1)*100 - 变化率

150. **TA_S_ROC**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Rate of change : ((price/prevPrice)-1)*100 (Single Precision) - 变化率（单精度）

151. **TA_ROCP**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Rate of change Percentage: (price-prevPrice)/prevPrice - 变化率百分比

152. **TA_S_ROCP**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Rate of change Percentage: (price-prevPrice)/prevPrice (Single Precision) - 变化率百分比（单精度）

153. **TA_ROCR**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Rate of change ratio: (price/prevPrice) - 变化率比率

154. **TA_S_ROCR**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Rate of change ratio: (price/prevPrice) (Single Precision) - 变化率比率（单精度）

155. **TA_ROCR100**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Rate of change ratio 100 scale: (price/prevPrice)*100 - 变化率比率100倍

156. **TA_S_ROCR100**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Rate of change ratio 100 scale: (price/prevPrice)*100 (Single Precision) - 变化率比率100倍（单精度）

157. **TA_RSI**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Relative Strength Index - 相对强弱指数

158. **TA_S_RSI**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Relative Strength Index (Single Precision) - 相对强弱指数（单精度）

159. **TA_TRIX**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // 1-day Rate-Of-Change (ROC) of a Triple Smooth EMA - 三重平滑EMA的1日变化率

160. **TA_S_TRIX**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // 1-day Rate-Of-Change (ROC) of a Triple Smooth EMA (Single Precision) - 三重平滑EMA的1日变化率（单精度）

## 价格指标函数

161. **TA_MEDPRICE**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Median Price - 中位价

162. **TA_S_MEDPRICE**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Median Price (Single Precision) - 中位价（单精度）

163. **TA_MIDPOINT**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // MidPoint over period - 周期中点

164. **TA_S_MIDPOINT**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // MidPoint over period (Single Precision) - 周期中点（单精度）

165. **TA_MIDPRICE**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Midpoint Price over period - 周期中点价格

166. **TA_S_MIDPRICE**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Midpoint Price over period (Single Precision) - 周期中点价格（单精度）

167. **TA_TYPPRICE**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Typical Price - 典型价格

168. **TA_S_TYPPRICE**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Typical Price (Single Precision) - 典型价格（单精度）

169. **TA_WCLPRICE**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Weighted Close Price - 加权收盘价

170. **TA_S_WCLPRICE**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Weighted Close Price (Single Precision) - 加权收盘价（单精度）

## 波动率指标函数

171. **TA_NATR**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Normalized Average True Range - 标准化平均真实波幅

172. **TA_S_NATR**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, inClose: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Normalized Average True Range (Single Precision) - 标准化平均真实波幅（单精度）

173. **TA_STDDEV**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, optInNbDev: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Standard Deviation - 标准偏差

174. **TA_S_STDDEV**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, optInNbDev: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Standard Deviation (Single Precision) - 标准偏差（单精度）

175. **TA_TRANGE**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // True Range - 真实波幅

176. **TA_S_TRANGE**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, inClose: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // True Range (Single Precision) - 真实波幅（单精度）

177. **TA_VAR**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, optInNbDev: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Variance - 方差

178. **TA_S_VAR**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, optInNbDev: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Variance (Single Precision) - 方差（单精度）

## 成交量指标函数

179. **TA_MFI**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, inVolume: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Money Flow Index - 资金流量指数

180. **TA_S_MFI**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, inClose: f32, inVolume: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Money Flow Index (Single Precision) - 资金流量指数（单精度）

181. **TA_OBV**(startIdx: i32, endIdx: i32, inReal: f64, inVolume: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // On Balance Volume - 能量潮

182. **TA_S_OBV**(startIdx: i32, endIdx: i32, inReal: f32, inVolume: f32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // On Balance Volume (Single Precision) - 能量潮（单精度）

## 其他重要指标

183. **TA_SAR**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, optInAcceleration: f64, optInMaximum: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Parabolic SAR - 抛物线SAR

184. **TA_S_SAR**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, optInAcceleration: f64, optInMaximum: f64, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Parabolic SAR (Single Precision) - 抛物线SAR（单精度）

185. **TA_STOCH**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInFastK_Period: i32, optInSlowK_Period: i32, optInSlowK_MAType: TA_MAType, optInSlowD_Period: i32, optInSlowD_MAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outSlowK: f64, outSlowD: f64) -> TA_RetCode
     // Stochastic - 随机指标

186. **TA_S_STOCH**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, inClose: f32, optInFastK_Period: i32, optInSlowK_Period: i32, optInSlowK_MAType: TA_MAType, optInSlowD_Period: i32, optInSlowD_MAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outSlowK: f64, outSlowD: f64) -> TA_RetCode
     // Stochastic (Single Precision) - 随机指标（单精度）

187. **TA_STOCHF**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInFastK_Period: i32, optInFastD_Period: i32, optInFastD_MAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outFastK: f64, outFastD: f64) -> TA_RetCode
     // Stochastic Fast - 快速随机指标

188. **TA_S_STOCHF**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, inClose: f32, optInFastK_Period: i32, optInFastD_Period: i32, optInFastD_MAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outFastK: f64, outFastD: f64) -> TA_RetCode
     // Stochastic Fast (Single Precision) - 快速随机指标（单精度）

189. **TA_STOCHRSI**(startIdx: i32, endIdx: i32, inReal: f64, optInTimePeriod: i32, optInFastK_Period: i32, optInFastD_Period: i32, optInFastD_MAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outFastK: f64, outFastD: f64) -> TA_RetCode
     // Stochastic Relative Strength Index - 随机相对强弱指数

190. **TA_S_STOCHRSI**(startIdx: i32, endIdx: i32, inReal: f32, optInTimePeriod: i32, optInFastK_Period: i32, optInFastD_Period: i32, optInFastD_MAType: TA_MAType, outBegIdx: i32, outNBElement: i32, outFastK: f64, outFastD: f64) -> TA_RetCode
     // Stochastic Relative Strength Index (Single Precision) - 随机相对强弱指数（单精度）

191. **TA_ULTOSC**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInTimePeriod1: i32, optInTimePeriod2: i32, optInTimePeriod3: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Ultimate Oscillator - 终极振荡器

192. **TA_S_ULTOSC**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, inClose: f32, optInTimePeriod1: i32, optInTimePeriod2: i32, optInTimePeriod3: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Ultimate Oscillator (Single Precision) - 终极振荡器（单精度）

193. **TA_WILLR**(startIdx: i32, endIdx: i32, inHigh: f64, inLow: f64, inClose: f64, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Williams' %R - 威廉指标

194. **TA_S_WILLR**(startIdx: i32, endIdx: i32, inHigh: f32, inLow: f32, inClose: f32, optInTimePeriod: i32, outBegIdx: i32, outNBElement: i32, outReal: f64) -> TA_RetCode
     // Williams' %R (Single Precision) - 威廉指标（单精度）

---

## 说明

- **参数类型映射**：
  - `::core::ffi::c_int` → `i32`
  - `*const f64` → `f64` (输入参数)
  - `*const f32` → `f32` (单精度输入参数)
  - `*mut f64` → `f64` (输出参数)
  - `*mut ::core::ffi::c_int` → `i32` (输出参数)

- **函数命名规则**：
  - `TA_` 开头的是双精度版本
  - `TA_S_` 开头的是单精度版本
  - 忽略了所有 `_Lookback` 后缀的函数

- **返回值**：所有函数都返回 `TA_RetCode` 类型，表示操作结果状态

- **常用参数**：
  - `startIdx`, `endIdx`: 计算的起始和结束索引
  - `optInTimePeriod`: 时间周期参数
  - `outBegIdx`, `outNBElement`: 输出数据的起始索引和元素数量