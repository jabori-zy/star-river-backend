use crate::define_indicator;

// K线形态识别指标 (Pattern Recognition Indicators)
// 1. CDL2CROWS            Two Crows #两只乌鸦
// 2. CDL3BLACKCROWS       Three Black Crows #三只乌鸦
// 3. CDL3INSIDE           Three Inside Up/Down #三内部上涨/下跌
// 4. CDL3LINESTRIKE       Three-Line Strike #三线打击
// 5. CDL3OUTSIDE          Three Outside Up/Down #三外部上涨/下跌
// 6. CDL3STARSINSOUTH     Three Stars In The South #三颗星在南方
// 7. CDL3WHITESOLDIERS    Three Advancing White Soldiers #三只白兵
// 8. CDLABANDONEDBABY     Abandoned Baby #弃婴
// 9. CDLADVANCEBLOCK      Advance Block #前进阻挡
// 10. CDLBELTHOLD         Belt-hold #带柄
// 11. CDLBREAKAWAY        Breakaway #突破
// 12. CDLCLOSINGMARUBOZU  Closing Marubozu #收盘十字星
// 13. CDLCONCEALBABYSWALL Concealing Baby Swallow #隐藏婴儿吞噬
// 14. CDLCOUNTERATTACK    Counterattack #反击
// 15. CDLDARKCLOUDCOVER   Dark Cloud Cover #乌云盖顶
// 16. CDLDOJI             Doji #十字星
// 17. CDLDOJISTAR         Doji Star #十字星
// 18. CDLDRAGONFLYDOJI    Dragonfly Doji #蜻蜓十字星
// 19. CDLENGULFING        Engulfing Pattern #吞噬模式
// 20. CDLEVENINGDOJISTAR  Evening Doji Star #黄昏十字星
// 21. CDLEVENINGSTAR      Evening Star #黄昏星
// 22. CDLGAPSIDESIDEWHITE Up/Down-gap side-by-side white lines #上/下缺口侧边白线
// 23. CDLGRAVESTONEDOJI   Gravestone Doji #墓碑十字星
// 24. CDLHAMMER           Hammer #锤子
// 25. CDLHANGINGMAN       Hanging Man #吊人
// 26. CDLHARAMI           Harami Pattern #孕线模式
// 27. CDLHARAMICROSS      Harami Cross Pattern #孕线交叉模式
// 28. CDLHIGHWAVE         High-Wave Candle #高浪烛
// 29. CDLHIKKAKE          Hikkake Pattern #跳空模式
// 30. CDLHIKKAKEMOD       Modified Hikkake Pattern #修改跳空模式
// 31. CDLHOMINGPIGEON     Homing Pigeon #归巢鸽
// 32. CDLIDENTICAL3CROWS  Identical Three Crows #相同三只乌鸦
// 33. CDLINNECK           In-Neck Pattern #颈线模式
// 34. CDLINVERTEDHAMMER   Inverted Hammer #倒锤子
// 35. CDLKICKING          Kicking #踢腿
// 36. CDLKICKINGBYLENGTH  Kicking - bull/bear determined by the longer marubozu #踢腿 - 牛/熊由更长的十字星决定
// 37. CDLLADDERBOTTOM     Ladder Bottom #梯底
// 38. CDLLONGLEGGEDDOJI   Long Legged Doji #长脚十字星
// 39. CDLLONGLINE         Long Line Candle #长蜡烛
// 40. CDLMARUBOZU         Marubozu #实体蜡烛
// 41. CDLMATCHINGLOW      Matching Low #匹配低点
// 42. CDLMATHOLD          Mat Hold #支撑
// 43. CDLMORNINGDOJISTAR  Morning Doji Star #早晨十字星
// 44. CDLMORNINGSTAR      Morning Star #早晨之星
// 45. CDLONNECK           On-Neck Pattern #颈线模式
// 46. CDLPIERCING         Piercing Pattern #刺透模式
// 47. CDLRICKSHAWMAN      Rickshaw Man #人力车夫
// 48. CDLRISEFALL3METHODS Rising/Falling Three Methods #上升/下降三法
// 49. CDLSEPARATINGLINES  Separating Lines #分离线
// 50. CDLSHOOTINGSTAR     Shooting Star #射击之星
// 51. CDLSHORTLINE        Short Line Candle #短蜡烛
// 52. CDLSPINNINGTOP      Spinning Top #旋转顶部
// 53. CDLSTALLEDPATTERN   Stalled Pattern #停滞模式
// 54. CDLSTICKSANDWICH    Stick Sandwich #针刺三明治
// 55. CDLTAKURI           Takuri (Dragonfly Doji with very long lower shadow) #蜻蜓十字星
// 56. CDLTASUKIGAP        Tasuki Gap #田中缺口
// 57. CDLTHRUSTING        Thrusting Pattern #刺穿模式
// 58. CDLTRISTAR          Tristar Pattern #三星模式
// 59. CDLUNIQUE3RIVER     Unique 3 River #唯一三河
// 60. CDLUPSIDEGAP2CROWS  Upside Gap Two Crows #上缺口两只乌鸦
// 61. CDLXSIDEGAP3METHODS Upside/Downside Gap Three Methods #上/下缺口三法

// CDL2CROWS            Two Crows #两只乌鸦
define_indicator!(CDL2CROWS,
    params => [],
    output => [(timestamp: i64), (two_crows: i32)],
);
// CDL3BLACKCROWS       Three Black Crows #三只乌鸦
define_indicator!(CDL3BLACKCROWS,
    params => [],
    output => [(timestamp: i64), (three_black_crows: i32)],
);

// CDL3INSIDE           Three Inside Up/Down #三内部上涨/下跌
define_indicator!(CDL3INSIDE,
    params => [],
    output => [(timestamp: i64), (three_inside: i32)],
);

// CDL3LINESTRIKE       Three-Line Strike #三线打击
define_indicator!(CDL3LINESTRIKE,
    params => [],
    output => [(timestamp: i64), (three_line_strike: i32)],
);

// CDL3OUTSIDE          Three Outside Up/Down #三外部上涨/下跌
define_indicator!(CDL3OUTSIDE,
    params => [],
    output => [(timestamp: i64), (three_outside: i32)],
);

// CDL3STARSINSOUTH     Three Stars In The South #三颗星在南方
define_indicator!(CDL3STARSINSOUTH,
    params => [],
    output => [(timestamp: i64), (three_stars_in_south: i32)],
);

// CDL3WHITESOLDIERS    Three Advancing White Soldiers #三只白兵
define_indicator!(CDL3WHITESOLDIERS,
    params => [],
    output => [(timestamp: i64), (three_white_soldiers: i32)],
);

// CDLABANDONEDBABY     Abandoned Baby #弃婴
define_indicator!(CDLABANDONEDBABY,
    params => [(penetration: f64)],
    output => [(timestamp: i64), (abandoned_baby: i32)],
);

// CDLADVANCEBLOCK      Advance Block #前进阻挡
define_indicator!(CDLADVANCEBLOCK,
    params => [],
    output => [(timestamp: i64), (advance_block: i32)],
);

// CDLBELTHOLD          Belt-hold #带柄
define_indicator!(CDLBELTHOLD,
    params => [],
    output => [(timestamp: i64), (belt_hold: i32)],
);

// CDLBREAKAWAY         Breakaway #突破
define_indicator!(CDLBREAKAWAY,
    params => [],
    output => [(timestamp: i64), (breakaway: i32)],
);

// CDLCLOSINGMARUBOZU   Closing Marubozu #收盘十字星
define_indicator!(CDLCLOSINGMARUBOZU,
    params => [],
    output => [(timestamp: i64), (closing_marubozu: i32)],
);

// CDLCONCEALBABYSWALL  Concealing Baby Swallow #隐藏婴儿吞噬
define_indicator!(CDLCONCEALBABYSWALL,
    params => [],
    output => [(timestamp: i64), (conceal_baby_swallow: i32)],
);

// CDLCOUNTERATTACK     Counterattack #反击
define_indicator!(CDLCOUNTERATTACK,
    params => [],
    output => [(timestamp: i64), (counterattack: i32)],
);

// CDLDARKCLOUDCOVER    Dark Cloud Cover #乌云盖顶
define_indicator!(CDLDARKCLOUDCOVER,
    params => [(penetration: f64)],
    output => [(timestamp: i64), (dark_cloud_cover: i32)],
);

// CDLDOJI              Doji #十字星
define_indicator!(CDLDOJI,
    params => [],
    output => [(timestamp: i64), (doji: i32)],
);

// CDLDOJISTAR          Doji Star #十字星
define_indicator!(CDLDOJISTAR,
    params => [],
    output => [(timestamp: i64), (doji_star: i32)],
);

// CDLDRAGONFLYDOJI     Dragonfly Doji #蜻蜓十字星
define_indicator!(CDLDRAGONFLYDOJI,
    params => [],
    output => [(timestamp: i64), (dragonfly_doji: i32)],
);

// CDLENGULFING         Engulfing Pattern #吞噬模式
define_indicator!(CDLENGULFING,
    params => [],
    output => [(timestamp: i64), (engulfing: i32)],
);

// CDLEVENINGDOJISTAR   Evening Doji Star #黄昏十字星
define_indicator!(CDLEVENINGDOJISTAR,
    params => [(penetration: f64)],
    output => [(timestamp: i64), (evening_doji_star: i32)],
);

// CDLEVENINGSTAR       Evening Star #黄昏星
define_indicator!(CDLEVENINGSTAR,
    params => [(penetration: f64)],
    output => [(timestamp: i64), (evening_star: i32)],
);

// CDLGAPSIDESIDEWHITE  Up/Down-gap side-by-side white lines #上/下缺口侧边白线
define_indicator!(CDLGAPSIDESIDEWHITE,
    params => [],
    output => [(timestamp: i64), (gap_side_side_white: i32)],
);

// CDLGRAVESTONEDOJI    Gravestone Doji #墓碑十字星
define_indicator!(CDLGRAVESTONEDOJI,
    params => [],
    output => [(timestamp: i64), (gravestone_doji: i32)],
);

// CDLHAMMER            Hammer #锤子
define_indicator!(CDLHAMMER,
    params => [],
    output => [(timestamp: i64), (hammer: i32)],
);

// CDLHANGINGMAN        Hanging Man #吊人
define_indicator!(CDLHANGINGMAN,
    params => [],
    output => [(timestamp: i64), (hanging_man: i32)],
);

// CDLHARAMI            Harami Pattern #孕线模式
define_indicator!(CDLHARAMI,
    params => [],
    output => [(timestamp: i64), (harami: i32)],
);

// CDLHARAMICROSS       Harami Cross Pattern #孕线交叉模式
define_indicator!(CDLHARAMICROSS,
    params => [],
    output => [(timestamp: i64), (harami_cross: i32)],
);

// CDLHIGHWAVE          High-Wave Candle #高浪烛
define_indicator!(CDLHIGHWAVE,
    params => [],
    output => [(timestamp: i64), (high_wave: i32)],
);

// CDLHIKKAKE           Hikkake Pattern #跳空模式
define_indicator!(CDLHIKKAKE,
    params => [],
    output => [(timestamp: i64), (hikkake: i32)],
);

// CDLHIKKAKEMOD        Modified Hikkake Pattern #修改跳空模式
define_indicator!(CDLHIKKAKEMOD,
    params => [],
    output => [(timestamp: i64), (hikkake_mod: i32)],
);

// CDLHOMINGPIGEON      Homing Pigeon #归巢鸽
define_indicator!(CDLHOMINGPIGEON,
    params => [],
    output => [(timestamp: i64), (homing_pigeon: i32)],
);

// CDLIDENTICAL3CROWS   Identical Three Crows #相同三只乌鸦
define_indicator!(CDLIDENTICAL3CROWS,
    params => [],
    output => [(timestamp: i64), (identical_three_crows: i32)],
);

// CDLINNECK            In-Neck Pattern #颈线模式
define_indicator!(CDLINNECK,
    params => [],
    output => [(timestamp: i64), (in_neck: i32)],
);

// CDLINVERTEDHAMMER    Inverted Hammer #倒锤子
define_indicator!(CDLINVERTEDHAMMER,
    params => [],
    output => [(timestamp: i64), (inverted_hammer: i32)],
);

// CDLKICKING           Kicking #踢腿
define_indicator!(CDLKICKING,
    params => [],
    output => [(timestamp: i64), (kicking: i32)],
);

// CDLKICKINGBYLENGTH   Kicking - bull/bear determined by the longer marubozu #踢腿 - 牛/熊由更长的十字星决定
define_indicator!(CDLKICKINGBYLENGTH,
    params => [],
    output => [(timestamp: i64), (kicking_by_length: i32)],
);

// CDLLADDERBOTTOM      Ladder Bottom #梯底
define_indicator!(CDLLADDERBOTTOM,
    params => [],
    output => [(timestamp: i64), (ladder_bottom: i32)],
);

// CDLLONGLEGGEDDOJI    Long Legged Doji #长脚十字星
define_indicator!(CDLLONGLEGGEDDOJI,
    params => [],
    output => [(timestamp: i64), (long_legged_doji: i32)],
);

// CDLLONGLINE          Long Line Candle #长蜡烛
define_indicator!(CDLLONGLINE,
    params => [],
    output => [(timestamp: i64), (long_line: i32)],
);

// CDLMARUBOZU          Marubozu #实体蜡烛
define_indicator!(CDLMARUBOZU,
    params => [],
    output => [(timestamp: i64), (marubozu: i32)],
);

// CDLMATCHINGLOW       Matching Low #匹配低点
define_indicator!(CDLMATCHINGLOW,
    params => [],
    output => [(timestamp: i64), (matching_low: i32)],
);

// CDLMATHOLD           Mat Hold #支撑
define_indicator!(CDLMATHOLD,
    params => [(penetration: f64)],
    output => [(timestamp: i64), (mat_hold: i32)],
);

// CDLMORNINGDOJISTAR   Morning Doji Star #早晨十字星
define_indicator!(CDLMORNINGDOJISTAR,
    params => [(penetration: f64)],
    output => [(timestamp: i64), (morning_doji_star: i32)],
);

// CDLMORNINGSTAR       Morning Star #早晨之星
define_indicator!(CDLMORNINGSTAR,
    params => [(penetration: f64)],
    output => [(timestamp: i64), (morning_star: i32)],
);

// CDLONNECK            On-Neck Pattern #颈线模式
define_indicator!(CDLONNECK,
    params => [],
    output => [(timestamp: i64), (on_neck: i32)],
);

// CDLPIERCING          Piercing Pattern #刺透模式
define_indicator!(CDLPIERCING,
    params => [],
    output => [(timestamp: i64), (piercing: i32)],
);

// CDLRICKSHAWMAN       Rickshaw Man #人力车夫
define_indicator!(CDLRICKSHAWMAN,
    params => [],
    output => [(timestamp: i64), (rickshaw_man: i32)],
);

// CDLRISEFALL3METHODS  Rising/Falling Three Methods #上升/下降三法
define_indicator!(CDLRISEFALL3METHODS,
    params => [],
    output => [(timestamp: i64), (rise_fall_three_methods: i32)],
);

// CDLSEPARATINGLINES   Separating Lines #分离线
define_indicator!(CDLSEPARATINGLINES,
    params => [],
    output => [(timestamp: i64), (separating_lines: i32)],
);

// CDLSHOOTINGSTAR      Shooting Star #射击之星
define_indicator!(CDLSHOOTINGSTAR,
    params => [],
    output => [(timestamp: i64), (shooting_star: i32)],
);

// CDLSHORTLINE         Short Line Candle #短蜡烛
define_indicator!(CDLSHORTLINE,
    params => [],
    output => [(timestamp: i64), (short_line: i32)],
);

// CDLSPINNINGTOP       Spinning Top #旋转顶部
define_indicator!(CDLSPINNINGTOP,
    params => [],
    output => [(timestamp: i64), (spinning_top: i32)],
);

// CDLSTALLEDPATTERN    Stalled Pattern #停滞模式
define_indicator!(CDLSTALLEDPATTERN,
    params => [],
    output => [(timestamp: i64), (stalled_pattern: i32)],
);

// CDLSTICKSANDWICH     Stick Sandwich #针刺三明治
define_indicator!(CDLSTICKSANDWICH,
    params => [],
    output => [(timestamp: i64), (stick_sandwich: i32)],
);

// CDLTAKURI            Takuri (Dragonfly Doji with very long lower shadow) #蜻蜓十字星
define_indicator!(CDLTAKURI,
    params => [],
    output => [(timestamp: i64), (takuri: i32)],
);

// CDLTASUKIGAP         Tasuki Gap #田中缺口
define_indicator!(CDLTASUKIGAP,
    params => [],
    output => [(timestamp: i64), (tasuki_gap: i32)],
);

// CDLTHRUSTING         Thrusting Pattern #刺穿模式
define_indicator!(CDLTHRUSTING,
    params => [],
    output => [(timestamp: i64), (thrusting: i32)],
);

// CDLTRISTAR           Tristar Pattern #三星模式
define_indicator!(CDLTRISTAR,
    params => [],
    output => [(timestamp: i64), (tristar: i32)],
);

// CDLUNIQUE3RIVER      Unique 3 River #唯一三河
define_indicator!(CDLUNIQUE3RIVER,
    params => [],
    output => [(timestamp: i64), (unique_three_river: i32)],
);

// CDLUPSIDEGAP2CROWS   Upside Gap Two Crows #上缺口两只乌鸦
define_indicator!(CDLUPSIDEGAP2CROWS,
    params => [],
    output => [(timestamp: i64), (upside_gap_two_crows: i32)],
);

// CDLXSIDEGAP3METHODS  Upside/Downside Gap Three Methods #上/下缺口三法
define_indicator!(CDLXSIDEGAP3METHODS,
    params => [],
    output => [(timestamp: i64), (xside_gap_three_methods: i32)],
);
