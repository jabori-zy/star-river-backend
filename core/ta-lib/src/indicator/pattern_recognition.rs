use chrono::{DateTime, Utc};

use crate::define_indicator;

// Candlestick Pattern Recognition Indicators
// 1. CDL2CROWS            Two Crows
// 2. CDL3BLACKCROWS       Three Black Crows
// 3. CDL3INSIDE           Three Inside Up/Down
// 4. CDL3LINESTRIKE       Three-Line Strike
// 5. CDL3OUTSIDE          Three Outside Up/Down
// 6. CDL3STARSINSOUTH     Three Stars In The South
// 7. CDL3WHITESOLDIERS    Three Advancing White Soldiers
// 8. CDLABANDONEDBABY     Abandoned Baby
// 9. CDLADVANCEBLOCK      Advance Block
// 10. CDLBELTHOLD         Belt-hold
// 11. CDLBREAKAWAY        Breakaway
// 12. CDLCLOSINGMARUBOZU  Closing Marubozu
// 13. CDLCONCEALBABYSWALL Concealing Baby Swallow
// 14. CDLCOUNTERATTACK    Counterattack
// 15. CDLDARKCLOUDCOVER   Dark Cloud Cover
// 16. CDLDOJI             Doji
// 17. CDLDOJISTAR         Doji Star
// 18. CDLDRAGONFLYDOJI    Dragonfly Doji
// 19. CDLENGULFING        Engulfing Pattern
// 20. CDLEVENINGDOJISTAR  Evening Doji Star
// 21. CDLEVENINGSTAR      Evening Star
// 22. CDLGAPSIDESIDEWHITE Up/Down-gap side-by-side white lines
// 23. CDLGRAVESTONEDOJI   Gravestone Doji
// 24. CDLHAMMER           Hammer
// 25. CDLHANGINGMAN       Hanging Man
// 26. CDLHARAMI           Harami Pattern
// 27. CDLHARAMICROSS      Harami Cross Pattern
// 28. CDLHIGHWAVE         High-Wave Candle
// 29. CDLHIKKAKE          Hikkake Pattern
// 30. CDLHIKKAKEMOD       Modified Hikkake Pattern
// 31. CDLHOMINGPIGEON     Homing Pigeon
// 32. CDLIDENTICAL3CROWS  Identical Three Crows
// 33. CDLINNECK           In-Neck Pattern
// 34. CDLINVERTEDHAMMER   Inverted Hammer
// 35. CDLKICKING          Kicking
// 36. CDLKICKINGBYLENGTH  Kicking - bull/bear determined by the longer marubozu
// 37. CDLLADDERBOTTOM     Ladder Bottom
// 38. CDLLONGLEGGEDDOJI   Long Legged Doji
// 39. CDLLONGLINE         Long Line Candle
// 40. CDLMARUBOZU         Marubozu
// 41. CDLMATCHINGLOW      Matching Low
// 42. CDLMATHOLD          Mat Hold
// 43. CDLMORNINGDOJISTAR  Morning Doji Star
// 44. CDLMORNINGSTAR      Morning Star
// 45. CDLONNECK           On-Neck Pattern
// 46. CDLPIERCING         Piercing Pattern
// 47. CDLRICKSHAWMAN      Rickshaw Man
// 48. CDLRISEFALL3METHODS Rising/Falling Three Methods
// 49. CDLSEPARATINGLINES  Separating Lines
// 50. CDLSHOOTINGSTAR     Shooting Star
// 51. CDLSHORTLINE        Short Line Candle
// 52. CDLSPINNINGTOP      Spinning Top
// 53. CDLSTALLEDPATTERN   Stalled Pattern
// 54. CDLSTICKSANDWICH    Stick Sandwich
// 55. CDLTAKURI           Takuri (Dragonfly Doji with very long lower shadow)
// 56. CDLTASUKIGAP        Tasuki Gap
// 57. CDLTHRUSTING        Thrusting Pattern
// 58. CDLTRISTAR          Tristar Pattern
// 59. CDLUNIQUE3RIVER     Unique 3 River
// 60. CDLUPSIDEGAP2CROWS  Upside Gap Two Crows
// 61. CDLXSIDEGAP3METHODS Upside/Downside Gap Three Methods

// CDL2CROWS            Two Crows
define_indicator!(CDL2CROWS,
    params => [],
    output => [(datetime: DateTime<Utc>), (two_crows: Option<i32>)],
);
// CDL3BLACKCROWS       Three Black Crows
define_indicator!(CDL3BLACKCROWS,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_black_crows: Option<i32>)],
);

// CDL3INSIDE           Three Inside Up/Down
define_indicator!(CDL3INSIDE,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_inside: Option<i32>)],
);

// CDL3LINESTRIKE       Three-Line Strike
define_indicator!(CDL3LINESTRIKE,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_line_strike: Option<i32>)],
);

// CDL3OUTSIDE          Three Outside Up/Down
define_indicator!(CDL3OUTSIDE,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_outside: Option<i32>)],
);

// CDL3STARSINSOUTH     Three Stars In The South
define_indicator!(CDL3STARSINSOUTH,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_stars_in_south: Option<i32>)],
);

// CDL3WHITESOLDIERS    Three Advancing White Soldiers
define_indicator!(CDL3WHITESOLDIERS,
    params => [],
    output => [(datetime: DateTime<Utc>), (three_white_soldiers: Option<i32>)],
);

// CDLABANDONEDBABY     Abandoned Baby
define_indicator!(CDLABANDONEDBABY,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (abandoned_baby: Option<i32>)],
);

// CDLADVANCEBLOCK      Advance Block
define_indicator!(CDLADVANCEBLOCK,
    params => [],
    output => [(datetime: DateTime<Utc>), (advance_block: Option<i32>)],
);

// CDLBELTHOLD          Belt-hold
define_indicator!(CDLBELTHOLD,
    params => [],
    output => [(datetime: DateTime<Utc>), (belt_hold: Option<i32>)],
);

// CDLBREAKAWAY         Breakaway
define_indicator!(CDLBREAKAWAY,
    params => [],
    output => [(datetime: DateTime<Utc>), (breakaway: Option<i32>)],
);

// CDLCLOSINGMARUBOZU   Closing Marubozu
define_indicator!(CDLCLOSINGMARUBOZU,
    params => [],
    output => [(datetime: DateTime<Utc>), (closing_marubozu: Option<i32>)],
);

// CDLCONCEALBABYSWALL  Concealing Baby Swallow
define_indicator!(CDLCONCEALBABYSWALL,
    params => [],
    output => [(datetime: DateTime<Utc>), (conceal_baby_swallow: Option<i32>)],
);

// CDLCOUNTERATTACK     Counterattack
define_indicator!(CDLCOUNTERATTACK,
    params => [],
    output => [(datetime: DateTime<Utc>), (counterattack: Option<i32>)],
);

// CDLDARKCLOUDCOVER    Dark Cloud Cover
define_indicator!(CDLDARKCLOUDCOVER,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (dark_cloud_cover: Option<i32>)],
);

// CDLDOJI              Doji
define_indicator!(CDLDOJI,
    params => [],
    output => [(datetime: DateTime<Utc>), (doji: Option<i32>)],
);

// CDLDOJISTAR          Doji Star
define_indicator!(CDLDOJISTAR,
    params => [],
    output => [(datetime: DateTime<Utc>), (doji_star: Option<i32>)],
);

// CDLDRAGONFLYDOJI     Dragonfly Doji
define_indicator!(CDLDRAGONFLYDOJI,
    params => [],
    output => [(datetime: DateTime<Utc>), (dragonfly_doji: Option<i32>)],
);

// CDLENGULFING         Engulfing Pattern
define_indicator!(CDLENGULFING,
    params => [],
    output => [(datetime: DateTime<Utc>), (engulfing: Option<i32>)],
);

// CDLEVENINGDOJISTAR   Evening Doji Star
define_indicator!(CDLEVENINGDOJISTAR,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (evening_doji_star: Option<i32>)],
);

// CDLEVENINGSTAR       Evening Star
define_indicator!(CDLEVENINGSTAR,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (evening_star: Option<i32>)],
);

// CDLGAPSIDESIDEWHITE  Up/Down-gap side-by-side white lines
define_indicator!(CDLGAPSIDESIDEWHITE,
    params => [],
    output => [(datetime: DateTime<Utc>), (gap_side_side_white: Option<i32>)],
);

// CDLGRAVESTONEDOJI    Gravestone Doji
define_indicator!(CDLGRAVESTONEDOJI,
    params => [],
    output => [(datetime: DateTime<Utc>), (gravestone_doji: Option<i32>)],
);

// CDLHAMMER            Hammer
define_indicator!(CDLHAMMER,
    params => [],
    output => [(datetime: DateTime<Utc>), (hammer: Option<i32>)],
);

// CDLHANGINGMAN        Hanging Man
define_indicator!(CDLHANGINGMAN,
    params => [],
    output => [(datetime: DateTime<Utc>), (hanging_man: Option<i32>)],
);

// CDLHARAMI            Harami Pattern
define_indicator!(CDLHARAMI,
    params => [],
    output => [(datetime: DateTime<Utc>), (harami: Option<i32>)],
);

// CDLHARAMICROSS       Harami Cross Pattern
define_indicator!(CDLHARAMICROSS,
    params => [],
    output => [(datetime: DateTime<Utc>), (harami_cross: Option<i32>)],
);

// CDLHIGHWAVE          High-Wave Candle
define_indicator!(CDLHIGHWAVE,
    params => [],
    output => [(datetime: DateTime<Utc>), (high_wave: Option<i32>)],
);

// CDLHIKKAKE           Hikkake Pattern
define_indicator!(CDLHIKKAKE,
    params => [],
    output => [(datetime: DateTime<Utc>), (hikkake: Option<i32>)],
);

// CDLHIKKAKEMOD        Modified Hikkake Pattern
define_indicator!(CDLHIKKAKEMOD,
    params => [],
    output => [(datetime: DateTime<Utc>), (hikkake_mod: Option<i32>)],
);

// CDLHOMINGPIGEON      Homing Pigeon
define_indicator!(CDLHOMINGPIGEON,
    params => [],
    output => [(datetime: DateTime<Utc>), (homing_pigeon: Option<i32>)],
);

// CDLIDENTICAL3CROWS   Identical Three Crows
define_indicator!(CDLIDENTICAL3CROWS,
    params => [],
    output => [(datetime: DateTime<Utc>), (identical_three_crows: Option<i32>)],
);

// CDLINNECK            In-Neck Pattern
define_indicator!(CDLINNECK,
    params => [],
    output => [(datetime: DateTime<Utc>), (in_neck: Option<i32>)],
);

// CDLINVERTEDHAMMER    Inverted Hammer
define_indicator!(CDLINVERTEDHAMMER,
    params => [],
    output => [(datetime: DateTime<Utc>), (inverted_hammer: Option<i32>)],
);

// CDLKICKING           Kicking
define_indicator!(CDLKICKING,
    params => [],
    output => [(datetime: DateTime<Utc>), (kicking: Option<i32>)],
);

// CDLKICKINGBYLENGTH   Kicking - bull/bear determined by the longer marubozu
define_indicator!(CDLKICKINGBYLENGTH,
    params => [],
    output => [(datetime: DateTime<Utc>), (kicking_by_length: Option<i32>)],
);

// CDLLADDERBOTTOM      Ladder Bottom
define_indicator!(CDLLADDERBOTTOM,
    params => [],
    output => [(datetime: DateTime<Utc>), (ladder_bottom: Option<i32>)],
);

// CDLLONGLEGGEDDOJI    Long Legged Doji
define_indicator!(CDLLONGLEGGEDDOJI,
    params => [],
    output => [(datetime: DateTime<Utc>), (long_legged_doji: Option<i32>)],
);

// CDLLONGLINE          Long Line Candle
define_indicator!(CDLLONGLINE,
    params => [],
    output => [(datetime: DateTime<Utc>), (long_line: Option<i32>)],
);

// CDLMARUBOZU          Marubozu
define_indicator!(CDLMARUBOZU,
    params => [],
    output => [(datetime: DateTime<Utc>), (marubozu: Option<i32>)],
);

// CDLMATCHINGLOW       Matching Low
define_indicator!(CDLMATCHINGLOW,
    params => [],
    output => [(datetime: DateTime<Utc>), (matching_low: Option<i32>)],
);

// CDLMATHOLD           Mat Hold
define_indicator!(CDLMATHOLD,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (mat_hold: Option<i32>)],
);

// CDLMORNINGDOJISTAR   Morning Doji Star
define_indicator!(CDLMORNINGDOJISTAR,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (morning_doji_star: Option<i32>)],
);

// CDLMORNINGSTAR       Morning Star
define_indicator!(CDLMORNINGSTAR,
    params => [(penetration: f64)],
    output => [(datetime: DateTime<Utc>), (morning_star: Option<i32>)],
);

// CDLONNECK            On-Neck Pattern
define_indicator!(CDLONNECK,
    params => [],
    output => [(datetime: DateTime<Utc>), (on_neck: Option<i32>)],
);

// CDLPIERCING          Piercing Pattern
define_indicator!(CDLPIERCING,
    params => [],
    output => [(datetime: DateTime<Utc>), (piercing: Option<i32>)],
);

// CDLRICKSHAWMAN       Rickshaw Man
define_indicator!(CDLRICKSHAWMAN,
    params => [],
    output => [(datetime: DateTime<Utc>), (rickshaw_man: Option<i32>)],
);

// CDLRISEFALL3METHODS  Rising/Falling Three Methods
define_indicator!(CDLRISEFALL3METHODS,
    params => [],
    output => [(datetime: DateTime<Utc>), (rise_fall_three_methods: Option<i32>)],
);

// CDLSEPARATINGLINES   Separating Lines
define_indicator!(CDLSEPARATINGLINES,
    params => [],
    output => [(datetime: DateTime<Utc>), (separating_lines: Option<i32>)],
);

// CDLSHOOTINGSTAR      Shooting Star
define_indicator!(CDLSHOOTINGSTAR,
    params => [],
    output => [(datetime: DateTime<Utc>), (shooting_star: Option<i32>)],
);

// CDLSHORTLINE         Short Line Candle
define_indicator!(CDLSHORTLINE,
    params => [],
    output => [(datetime: DateTime<Utc>), (short_line: Option<i32>)],
);

// CDLSPINNINGTOP       Spinning Top
define_indicator!(CDLSPINNINGTOP,
    params => [],
    output => [(datetime: DateTime<Utc>), (spinning_top: Option<i32>)],
);

// CDLSTALLEDPATTERN    Stalled Pattern
define_indicator!(CDLSTALLEDPATTERN,
    params => [],
    output => [(datetime: DateTime<Utc>), (stalled_pattern: Option<i32>)],
);

// CDLSTICKSANDWICH     Stick Sandwich
define_indicator!(CDLSTICKSANDWICH,
    params => [],
    output => [(datetime: DateTime<Utc>), (stick_sandwich: Option<i32>)],
);

// CDLTAKURI            Takuri (Dragonfly Doji with very long lower shadow)
define_indicator!(CDLTAKURI,
    params => [],
    output => [(datetime: DateTime<Utc>), (takuri: Option<i32>)],
);

// CDLTASUKIGAP         Tasuki Gap
define_indicator!(CDLTASUKIGAP,
    params => [],
    output => [(datetime: DateTime<Utc>), (tasuki_gap: Option<i32>)],
);

// CDLTHRUSTING         Thrusting Pattern
define_indicator!(CDLTHRUSTING,
    params => [],
    output => [(datetime: DateTime<Utc>), (thrusting: Option<i32>)],
);

// CDLTRISTAR           Tristar Pattern
define_indicator!(CDLTRISTAR,
    params => [],
    output => [(datetime: DateTime<Utc>), (tristar: Option<i32>)],
);

// CDLUNIQUE3RIVER      Unique 3 River
define_indicator!(CDLUNIQUE3RIVER,
    params => [],
    output => [(datetime: DateTime<Utc>), (unique_three_river: Option<i32>)],
);

// CDLUPSIDEGAP2CROWS   Upside Gap Two Crows
define_indicator!(CDLUPSIDEGAP2CROWS,
    params => [],
    output => [(datetime: DateTime<Utc>), (upside_gap_two_crows: Option<i32>)],
);

// CDLXSIDEGAP3METHODS  Upside/Downside Gap Three Methods
define_indicator!(CDLXSIDEGAP3METHODS,
    params => [],
    output => [(datetime: DateTime<Utc>), (xside_gap_three_methods: Option<i32>)],
);
