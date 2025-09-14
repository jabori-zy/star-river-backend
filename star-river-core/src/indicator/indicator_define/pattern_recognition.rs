use crate::define_indicator;
use chrono::{DateTime, Utc};

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
    output => [(datetime: DateTime<Utc>), (two_crows: i32)],
);
// CDL3BLACKCROWS       Three Black Crows #三只乌鸦
define_indicator!(CDL3BLACKCROWS,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_black_crows: i32)],
);

// CDL3INSIDE           Three Inside Up/Down #三内部上涨/下跌
define_indicator!(CDL3INSIDE,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_inside: i32)],
);

// CDL3LINESTRIKE       Three-Line Strike #三线打击
define_indicator!(CDL3LINESTRIKE,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_line_strike: i32)],
);

// CDL3OUTSIDE          Three Outside Up/Down #三外部上涨/下跌
define_indicator!(CDL3OUTSIDE,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_outside: i32)],
);

// CDL3STARSINSOUTH     Three Stars In The South #三颗星在南方
define_indicator!(CDL3STARSINSOUTH,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_stars_in_south: i32)],
);

// CDL3WHITESOLDIERS    Three Advancing White Soldiers #三只白兵
define_indicator!(CDL3WHITESOLDIERS,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_white_soldiers: i32)],
);

// CDLABANDONEDBABY     Abandoned Baby #弃婴
define_indicator!(CDLABANDONEDBABY,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (abandoned_baby: i32)],
);

// CDLADVANCEBLOCK      Advance Block #前进阻挡
define_indicator!(CDLADVANCEBLOCK,
    params => [],
    output => [(datetime: DateTime<Utc>), (advance_block: i32)],
);

// CDLBELTHOLD          Belt-hold #带柄
define_indicator!(CDLBELTHOLD,
    params => [],
    output => [(datetime: DateTime<Utc>), (belt_hold: i32)],
);

// CDLBREAKAWAY         Breakaway #突破
define_indicator!(CDLBREAKAWAY,
    params => [],
    output => [(datetime: DateTime<Utc>), (breakaway: i32)],
);

// CDLCLOSINGMARUBOZU   Closing Marubozu #收盘十字星
define_indicator!(CDLCLOSINGMARUBOZU,
    params => [],
    output => [(datetime: DateTime<Utc>), (closing_marubozu: i32)],
);

// CDLCONCEALBABYSWALL  Concealing Baby Swallow #隐藏婴儿吞噬
define_indicator!(CDLCONCEALBABYSWALL,
    params => [],
    output => [(datetime: DateTime<Utc>), (conceal_baby_swallow: i32)],
);

// CDLCOUNTERATTACK     Counterattack #反击
define_indicator!(CDLCOUNTERATTACK,
    params => [],
    output => [(datetime: DateTime<Utc>), (counterattack: i32)],
);

// CDLDARKCLOUDCOVER    Dark Cloud Cover #乌云盖顶
define_indicator!(CDLDARKCLOUDCOVER,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (dark_cloud_cover: i32)],
);

// CDLDOJI              Doji #十字星
define_indicator!(CDLDOJI,
    params => [],
    output => [(datetime: DateTime<Utc>), (doji: i32)],
);

// CDLDOJISTAR          Doji Star #十字星
define_indicator!(CDLDOJISTAR,
    params => [],
    output => [(datetime: DateTime<Utc>), (doji_star: i32)],
);

// CDLDRAGONFLYDOJI     Dragonfly Doji #蜻蜓十字星
define_indicator!(CDLDRAGONFLYDOJI,
    params => [],
    output => [(datetime: DateTime<Utc>), (dragonfly_doji: i32)],
);

// CDLENGULFING         Engulfing Pattern #吞噬模式
define_indicator!(CDLENGULFING,
    params => [],
    output => [(datetime: DateTime<Utc>), (engulfing: i32)],
);

// CDLEVENINGDOJISTAR   Evening Doji Star #黄昏十字星
define_indicator!(CDLEVENINGDOJISTAR,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (evening_doji_star: i32)],
);

// CDLEVENINGSTAR       Evening Star #黄昏星
define_indicator!(CDLEVENINGSTAR,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (evening_star: i32)],
);

// CDLGAPSIDESIDEWHITE  Up/Down-gap side-by-side white lines #上/下缺口侧边白线
define_indicator!(CDLGAPSIDESIDEWHITE,
    params => [],
    output => [(datetime: DateTime<Utc>), (gap_side_side_white: i32)],
);

// CDLGRAVESTONEDOJI    Gravestone Doji #墓碑十字星
define_indicator!(CDLGRAVESTONEDOJI,
    params => [],
    output => [(datetime: DateTime<Utc>), (gravestone_doji: i32)],
);

// CDLHAMMER            Hammer #锤子
define_indicator!(CDLHAMMER,
    params => [],
    output => [(datetime: DateTime<Utc>), (hammer: i32)],
);

// CDLHANGINGMAN        Hanging Man #吊人
define_indicator!(CDLHANGINGMAN,
    params => [],
    output => [(datetime: DateTime<Utc>), (hanging_man: i32)],
);

// CDLHARAMI            Harami Pattern #孕线模式
define_indicator!(CDLHARAMI,
    params => [],
    output => [(datetime: DateTime<Utc>), (harami: i32)],
);

// CDLHARAMICROSS       Harami Cross Pattern #孕线交叉模式
define_indicator!(CDLHARAMICROSS,
    params => [],
    output => [(datetime: DateTime<Utc>), (harami_cross: i32)],
);

// CDLHIGHWAVE          High-Wave Candle #高浪烛
define_indicator!(CDLHIGHWAVE,
    params => [],
    output => [(datetime: DateTime<Utc>), (high_wave: i32)],
);

// CDLHIKKAKE           Hikkake Pattern #跳空模式
define_indicator!(CDLHIKKAKE,
    params => [],
    output => [(datetime: DateTime<Utc>), (hikkake: i32)],
);

// CDLHIKKAKEMOD        Modified Hikkake Pattern #修改跳空模式
define_indicator!(CDLHIKKAKEMOD,
    params => [],
    output => [(datetime: DateTime<Utc>), (hikkake_mod: i32)],
);

// CDLHOMINGPIGEON      Homing Pigeon #归巢鸽
define_indicator!(CDLHOMINGPIGEON,
    params => [],
    output => [(datetime: DateTime<Utc>), (homing_pigeon: i32)],
);

// CDLIDENTICAL3CROWS   Identical Three Crows #相同三只乌鸦
define_indicator!(CDLIDENTICAL3CROWS,
    params => [],
    output => [(datetime: DateTime<Utc>), (identical_three_crows: i32)],
);

// CDLINNECK            In-Neck Pattern #颈线模式
define_indicator!(CDLINNECK,
    params => [],
    output => [(datetime: DateTime<Utc>), (in_neck: i32)],
);

// CDLINVERTEDHAMMER    Inverted Hammer #倒锤子
define_indicator!(CDLINVERTEDHAMMER,
    params => [],
    output => [(datetime: DateTime<Utc>), (inverted_hammer: i32)],
);

// CDLKICKING           Kicking #踢腿
define_indicator!(CDLKICKING,
    params => [],
    output => [(datetime: DateTime<Utc>), (kicking: i32)],
);

// CDLKICKINGBYLENGTH   Kicking - bull/bear determined by the longer marubozu #踢腿 - 牛/熊由更长的十字星决定
define_indicator!(CDLKICKINGBYLENGTH,
    params => [],
    output => [(datetime: DateTime<Utc>), (kicking_by_length: i32)],
);

// CDLLADDERBOTTOM      Ladder Bottom #梯底
define_indicator!(CDLLADDERBOTTOM,
    params => [],
    output => [(datetime: DateTime<Utc>), (ladder_bottom: i32)],
);

// CDLLONGLEGGEDDOJI    Long Legged Doji #长脚十字星
define_indicator!(CDLLONGLEGGEDDOJI,
    params => [],
    output => [(datetime: DateTime<Utc>), (long_legged_doji: i32)],
);

// CDLLONGLINE          Long Line Candle #长蜡烛
define_indicator!(CDLLONGLINE,
    params => [],
    output => [(datetime: DateTime<Utc>), (long_line: i32)],
);

// CDLMARUBOZU          Marubozu #实体蜡烛
define_indicator!(CDLMARUBOZU,
    params => [],
    output => [(datetime: DateTime<Utc>), (marubozu: i32)],
);

// CDLMATCHINGLOW       Matching Low #匹配低点
define_indicator!(CDLMATCHINGLOW,
    params => [],
    output => [(datetime: DateTime<Utc>), (matching_low: i32)],
);

// CDLMATHOLD           Mat Hold #支撑
define_indicator!(CDLMATHOLD,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (mat_hold: i32)],
);

// CDLMORNINGDOJISTAR   Morning Doji Star #早晨十字星
define_indicator!(CDLMORNINGDOJISTAR,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (morning_doji_star: i32)],
);

// CDLMORNINGSTAR       Morning Star #早晨之星
define_indicator!(CDLMORNINGSTAR,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (morning_star: i32)],
);

// CDLONNECK            On-Neck Pattern #颈线模式
define_indicator!(CDLONNECK,
    params => [],
    output => [(datetime: DateTime<Utc>), (on_neck: i32)],
);

// CDLPIERCING          Piercing Pattern #刺透模式
define_indicator!(CDLPIERCING,
    params => [],
    output => [(datetime: DateTime<Utc>), (piercing: i32)],
);

// CDLRICKSHAWMAN       Rickshaw Man #人力车夫
define_indicator!(CDLRICKSHAWMAN,
    params => [],
    output => [(datetime: DateTime<Utc>), (rickshaw_man: i32)],
);

// CDLRISEFALL3METHODS  Rising/Falling Three Methods #上升/下降三法
define_indicator!(CDLRISEFALL3METHODS,
    params => [],
    output => [(datetime: DateTime<Utc>), (rise_fall_three_methods: i32)],
);

// CDLSEPARATINGLINES   Separating Lines #分离线
define_indicator!(CDLSEPARATINGLINES,
    params => [],
    output => [(datetime: DateTime<Utc>), (separating_lines: i32)],
);

// CDLSHOOTINGSTAR      Shooting Star #射击之星
define_indicator!(CDLSHOOTINGSTAR,
    params => [],
    output => [(datetime: DateTime<Utc>), (shooting_star: i32)],
);

// CDLSHORTLINE         Short Line Candle #短蜡烛
define_indicator!(CDLSHORTLINE,
    params => [],
    output => [(datetime: DateTime<Utc>), (short_line: i32)],
);

// CDLSPINNINGTOP       Spinning Top #旋转顶部
define_indicator!(CDLSPINNINGTOP,
    params => [],
    output => [(datetime: DateTime<Utc>), (spinning_top: i32)],
);

// CDLSTALLEDPATTERN    Stalled Pattern #停滞模式
define_indicator!(CDLSTALLEDPATTERN,
    params => [],
    output => [(datetime: DateTime<Utc>), (stalled_pattern: i32)],
);

// CDLSTICKSANDWICH     Stick Sandwich #针刺三明治
define_indicator!(CDLSTICKSANDWICH,
    params => [],
    output => [(datetime: DateTime<Utc>), (stick_sandwich: i32)],
);

// CDLTAKURI            Takuri (Dragonfly Doji with very long lower shadow) #蜻蜓十字星
define_indicator!(CDLTAKURI,
    params => [],
    output => [(datetime: DateTime<Utc>), (takuri: i32)],
);

// CDLTASUKIGAP         Tasuki Gap #田中缺口
define_indicator!(CDLTASUKIGAP,
    params => [],
    output => [(datetime: DateTime<Utc>), (tasuki_gap: i32)],
);

// CDLTHRUSTING         Thrusting Pattern #刺穿模式
define_indicator!(CDLTHRUSTING,
    params => [],
    output => [(datetime: DateTime<Utc>), (thrusting: i32)],
);

// CDLTRISTAR           Tristar Pattern #三星模式
define_indicator!(CDLTRISTAR,
    params => [],
    output => [(datetime: DateTime<Utc>), (tristar: i32)],
);

// CDLUNIQUE3RIVER      Unique 3 River #唯一三河
define_indicator!(CDLUNIQUE3RIVER,
    params => [],
    output => [(datetime: DateTime<Utc>), (unique_three_river: i32)],
);

// CDLUPSIDEGAP2CROWS   Upside Gap Two Crows #上缺口两只乌鸦
define_indicator!(CDLUPSIDEGAP2CROWS,
    params => [],
    output => [(datetime: DateTime<Utc>), (upside_gap_two_crows: i32)],
);

// CDLXSIDEGAP3METHODS  Upside/Downside Gap Three Methods #上/下缺口三法
define_indicator!(CDLXSIDEGAP3METHODS,
    params => [],
    output => [(datetime: DateTime<Utc>), (xside_gap_three_methods: i32)],
);
