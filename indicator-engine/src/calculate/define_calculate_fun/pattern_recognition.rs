use ta_lib::{Indicator, indicator::pattern_recognition::*};

use crate::{calculate::CalculateIndicatorFunction, calculate_fn};

impl CalculateIndicatorFunction {
    // CDL2CROWS - Two Crows
    calculate_fn!(CDL2CROWS,
        input => [open,high,low,close]
    );

    // CDL3BLACKCROWS - Three Black Crows
    calculate_fn!(CDL3BLACKCROWS,
        input => [open,high,low,close]
    );

    // CDL3INSIDE - Three Inside Up/Down
    calculate_fn!(CDL3INSIDE,
        input => [open,high,low,close]
    );

    // CDL3LINESTRIKE - Three-Line Strike
    calculate_fn!(CDL3LINESTRIKE,
        input => [open,high,low,close]
    );

    // CDL3OUTSIDE - Three Outside Up/Down
    calculate_fn!(CDL3OUTSIDE,
        input => [open,high,low,close]
    );

    // CDL3STARSINSOUTH - Three Stars In The South
    calculate_fn!(CDL3STARSINSOUTH,
        input => [open,high,low,close]
    );

    // CDL3WHITESOLDIERS - Three Advancing White Soldiers
    calculate_fn!(CDL3WHITESOLDIERS,
        input => [open,high,low,close]
    );

    // CDLABANDONEDBABY - Abandoned Baby
    calculate_fn!(CDLABANDONEDBABY,
        input => [open,high,low,close],
        talib_params => [(penetration: f64)]
    );

    // CDLADVANCEBLOCK - Advance Block
    calculate_fn!(CDLADVANCEBLOCK,
        input => [open,high,low,close]
    );

    // CDLBELTHOLD - Belt-hold
    calculate_fn!(CDLBELTHOLD,
        input => [open,high,low,close]
    );

    // CDLBREAKAWAY - Breakaway
    calculate_fn!(CDLBREAKAWAY,
        input => [open,high,low,close]
    );

    // CDLCLOSINGMARUBOZU - Closing Marubozu
    calculate_fn!(CDLCLOSINGMARUBOZU,
        input => [open,high,low,close]
    );

    // CDLCONCEALBABYSWALL - Concealing Baby Swallow
    calculate_fn!(CDLCONCEALBABYSWALL,
        input => [open,high,low,close]
    );

    // CDLCOUNTERATTACK - Counterattack
    calculate_fn!(CDLCOUNTERATTACK,
        input => [open,high,low,close]
    );

    // CDLDARKCLOUDCOVER - Dark Cloud Cover
    calculate_fn!(CDLDARKCLOUDCOVER,
        input => [open,high,low,close],
        talib_params => [(penetration: f64)]
    );

    // CDLDOJI - Doji
    calculate_fn!(CDLDOJI,
        input => [open,high,low,close]
    );

    // CDLDOJISTAR - Doji Star
    calculate_fn!(CDLDOJISTAR,
        input => [open, high, low, close]
    );

    // CDLDRAGONFLYDOJI - Dragonfly Doji
    calculate_fn!(CDLDRAGONFLYDOJI,
        input => [open, high, low, close]
    );

    // CDLENGULFING - Engulfing Pattern
    calculate_fn!(CDLENGULFING,
        input => [open, high, low, close]
    );

    // CDLEVENINGDOJISTAR - Evening Doji Star
    calculate_fn!(CDLEVENINGDOJISTAR,
        input => [open, high, low, close],
        talib_params => [(penetration: f64)]
    );

    // CDLEVENINGSTAR - Evening Star
    calculate_fn!(CDLEVENINGSTAR,
        input => [open, high, low, close],
        talib_params => [(penetration: f64)]
    );

    // CDLGAPSIDESIDEWHITE - Up/Down-gap side-by-side white lines
    calculate_fn!(CDLGAPSIDESIDEWHITE,
        input => [open, high, low, close]
    );

    // CDLGRAVESTONEDOJI - Gravestone Doji
    calculate_fn!(CDLGRAVESTONEDOJI,
        input => [open, high, low, close]
    );

    // CDLHAMMER - Hammer
    calculate_fn!(CDLHAMMER,
        input => [open, high, low, close]
    );

    // CDLHANGINGMAN - Hanging Man
    calculate_fn!(CDLHANGINGMAN,
        input => [open, high, low, close]
    );

    // CDLHARAMI - Harami Pattern
    calculate_fn!(CDLHARAMI,
        input => [open, high, low, close]
    );

    // CDLHARAMICROSS - Harami Cross Pattern
    calculate_fn!(CDLHARAMICROSS,
        input => [open, high, low, close]
    );

    // CDLHIGHWAVE - High-Wave Candle
    calculate_fn!(CDLHIGHWAVE,
        input => [open, high, low, close]
    );

    // CDLHIKKAKE - Hikkake Pattern
    calculate_fn!(CDLHIKKAKE,
        input => [open, high, low, close]
    );

    // CDLHIKKAKEMOD - Modified Hikkake Pattern
    calculate_fn!(CDLHIKKAKEMOD,
        input => [open, high, low, close]
    );

    // CDLHOMINGPIGEON - Homing Pigeon
    calculate_fn!(CDLHOMINGPIGEON,
        input => [open, high, low, close]
    );

    // CDLIDENTICAL3CROWS - Identical Three Crows
    calculate_fn!(CDLIDENTICAL3CROWS,
        input => [open, high, low, close]
    );

    // CDLINNECK - In-Neck Pattern
    calculate_fn!(CDLINNECK,
        input => [open, high, low, close]
    );

    // CDLINVERTEDHAMMER - Inverted Hammer
    calculate_fn!(CDLINVERTEDHAMMER,
        input => [open, high, low, close]
    );

    // CDLKICKING - Kicking
    calculate_fn!(CDLKICKING,
        input => [open, high, low, close]
    );

    // CDLKICKINGBYLENGTH - Kicking - bull/bear determined by the longer marubozu
    calculate_fn!(CDLKICKINGBYLENGTH,
        input => [open, high, low, close]
    );

    // CDLLADDERBOTTOM - Ladder Bottom
    calculate_fn!(CDLLADDERBOTTOM,
        input => [open, high, low, close]
    );

    // CDLLONGLEGGEDDOJI - Long Legged Doji
    calculate_fn!(CDLLONGLEGGEDDOJI,
        input => [open, high, low, close]
    );

    // CDLLONGLINE - Long Line Candle
    calculate_fn!(CDLLONGLINE,
        input => [open, high, low, close]
    );

    // CDLMARUBOZU - Marubozu
    calculate_fn!(CDLMARUBOZU,
        input => [open, high, low, close]
    );

    // CDLMATCHINGLOW - Matching Low
    calculate_fn!(CDLMATCHINGLOW,
        input => [open, high, low, close]
    );

    // CDLMATHOLD - Mat Hold
    calculate_fn!(CDLMATHOLD,
        input => [open, high, low, close],
        talib_params => [(penetration: f64)]
    );

    // CDLMORNINGDOJISTAR - Morning Doji Star
    calculate_fn!(CDLMORNINGDOJISTAR,
        input => [open, high, low, close],
        talib_params => [(penetration: f64)]
    );

    // CDLMORNINGSTAR - Morning Star
    calculate_fn!(CDLMORNINGSTAR,
        input => [open, high, low, close],
        talib_params => [(penetration: f64)]
    );

    // CDLONNECK - On-Neck Pattern
    calculate_fn!(CDLONNECK,
        input => [open, high, low, close]
    );

    // CDLPIERCING - Piercing Pattern
    calculate_fn!(CDLPIERCING,
        input => [open, high, low, close]
    );

    // CDLRICKSHAWMAN - Rickshaw Man
    calculate_fn!(CDLRICKSHAWMAN,
        input => [open, high, low, close]
    );

    // CDLRISEFALL3METHODS - Rising/Falling Three Methods
    calculate_fn!(CDLRISEFALL3METHODS,
        input => [open, high, low, close]
    );

    // CDLSEPARATINGLINES - Separating Lines
    calculate_fn!(CDLSEPARATINGLINES,
        input => [open, high, low, close]
    );

    // CDLSHOOTINGSTAR - Shooting Star
    calculate_fn!(CDLSHOOTINGSTAR,
        input => [open, high, low, close]
    );

    // CDLSHORTLINE - Short Line Candle
    calculate_fn!(CDLSHORTLINE,
        input => [open, high, low, close]
    );

    // CDLSPINNINGTOP - Spinning Top
    calculate_fn!(CDLSPINNINGTOP,
        input => [open, high, low, close]
    );

    // CDLSTALLEDPATTERN - Stalled Pattern
    calculate_fn!(CDLSTALLEDPATTERN,
        input => [open, high, low, close]
    );

    // CDLSTICKSANDWICH - Stick Sandwich
    calculate_fn!(CDLSTICKSANDWICH,
        input => [open, high, low, close]
    );

    // CDLTAKURI - Takuri (Dragonfly Doji with very long lower shadow)
    calculate_fn!(CDLTAKURI,
        input => [open, high, low, close]
    );

    // CDLTASUKIGAP - Tasuki Gap
    calculate_fn!(CDLTASUKIGAP,
        input => [open, high, low, close]
    );

    // CDLTHRUSTING - Thrusting Pattern
    calculate_fn!(CDLTHRUSTING,
        input => [open, high, low, close]
    );

    // CDLTRISTAR - Tristar Pattern
    calculate_fn!(CDLTRISTAR,
        input => [open, high, low, close]
    );

    // CDLUNIQUE3RIVER - Unique 3 River
    calculate_fn!(CDLUNIQUE3RIVER,
        input => [open, high, low, close]
    );

    // CDLUPSIDEGAP2CROWS - Upside Gap Two Crows
    calculate_fn!(CDLUPSIDEGAP2CROWS,
        input => [open, high, low, close]
    );

    // CDLXSIDEGAP3METHODS - Upside/Downside Gap Three Methods
    calculate_fn!(CDLXSIDEGAP3METHODS,
        input => [open, high, low, close]
    );
}
