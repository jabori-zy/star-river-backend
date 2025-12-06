use chrono::{DateTime, Utc};

// use crate::indicator_engine::talib_error::TalibError;
use crate::{Indicator, indicator::pattern_recognition::*, talib_fn};
use crate::{talib::TALib, talib_bindings::*};

impl TALib {
    // CDL2CROWS - Two Crows
    talib_fn!(
        CDL2CROWS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(two_crows: i32)],
    );

    // CDL3BLACKCROWS - Three Black Crows
    talib_fn!(
        CDL3BLACKCROWS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_black_crows: i32)],
    );

    // CDL3INSIDE - Three Inside Up/Down
    talib_fn!(
        CDL3INSIDE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_inside: i32)],
    );

    // CDL3LINESTRIKE - Three-Line Strike
    talib_fn!(
        CDL3LINESTRIKE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_line_strike: i32)],
    );

    // CDL3OUTSIDE - Three Outside Up/Down
    talib_fn!(
        CDL3OUTSIDE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_outside: i32)],
    );

    // CDL3STARSINSOUTH - Three Stars In The South
    talib_fn!(
        CDL3STARSINSOUTH,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_stars_in_south: i32)],
    );

    // CDL3WHITESOLDIERS    Three Advancing White Soldiers   
    talib_fn!(
        CDL3WHITESOLDIERS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_white_soldiers: i32)],
    );

    // CDLABANDONEDBABY     Abandoned Baby
    talib_fn!(
        CDLABANDONEDBABY,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(abandoned_baby: i32)],
    );

    // CDLADVANCEBLOCK      Advance Block
    talib_fn!(
        CDLADVANCEBLOCK,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(advance_block: i32)],
    );

    // CDLBELTHOLD          Belt-hold   
    talib_fn!(
        CDLBELTHOLD,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(belt_hold: i32)],
    );

    // CDLBREAKAWAY         Breakaway
    talib_fn!(
        CDLBREAKAWAY,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(breakaway: i32)],
    );

    // CDLCLOSINGMARUBOZU   Closing Marubozu
    talib_fn!(
        CDLCLOSINGMARUBOZU,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(closing_marubozu: i32)],
    );

    // CDLCONCEALBABYSWALL  Concealing Baby Swallow
    talib_fn!(
        CDLCONCEALBABYSWALL,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(conceal_baby_swallow: i32)],
    );

    // CDLCOUNTERATTACK     Counterattack   
    talib_fn!(
        CDLCOUNTERATTACK,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(counterattack: i32)],
    );

    // CDLDARKCLOUDCOVER    Dark Cloud Cover
    talib_fn!(
        CDLDARKCLOUDCOVER,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(dark_cloud_cover: i32)],
    );

    // CDLDOJI              Doji
    talib_fn!(
        CDLDOJI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(doji: i32)],
    );

    // CDLDOJISTAR          Doji Star   
    talib_fn!(
        CDLDOJISTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(doji_star: i32)],
    );

    // CDLDRAGONFLYDOJI     Dragonfly Doji
    talib_fn!(
        CDLDRAGONFLYDOJI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(dragonfly_doji: i32)],
    );

    // CDLENGULFING         Engulfing Pattern   
    talib_fn!(
        CDLENGULFING,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(engulfing: i32)],
    );

    // CDLEVENINGDOJISTAR   Evening Doji Star   
    talib_fn!(
        CDLEVENINGDOJISTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(evening_doji_star: i32)],
    );

    // CDLEVENINGSTAR       Evening Star   
    talib_fn!(
        CDLEVENINGSTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(evening_star: i32)],
    );

    // CDLGAPSIDESIDEWHITE  Up/Down-gap side-by-side white lines   
    talib_fn!(
        CDLGAPSIDESIDEWHITE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(gap_side_side_white: i32)],
    );

    // CDLGRAVESTONEDOJI    Gravestone Doji   
    talib_fn!(
        CDLGRAVESTONEDOJI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(gravestone_doji: i32)],
    );

    // CDLHAMMER            Hammer
    talib_fn!(
        CDLHAMMER,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(hammer: i32)],
    );

    // CDLHANGINGMAN        Hanging Man
    talib_fn!(
        CDLHANGINGMAN,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(hanging_man: i32)],
    );

    // CDLHARAMI            Harami Pattern   
    talib_fn!(
        CDLHARAMI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(harami: i32)],
    );

    // CDLHARAMICROSS       Harami Cross Pattern   
    talib_fn!(
        CDLHARAMICROSS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(harami_cross: i32)],
    );

    // CDLHIGHWAVE          High-Wave Candle
    talib_fn!(
        CDLHIGHWAVE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(high_wave: i32)],
    );

    // CDLHIKKAKE           Hikkake Pattern   
    talib_fn!(
        CDLHIKKAKE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(hikkake: i32)],
    );

    // CDLHIKKAKEMOD        Modified Hikkake Pattern   
    talib_fn!(
        CDLHIKKAKEMOD,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(hikkake_mod: i32)],
    );

    // CDLHOMINGPIGEON      Homing Pigeon
    talib_fn!(
        CDLHOMINGPIGEON,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(homing_pigeon: i32)],
    );

    // CDLIDENTICAL3CROWS   Identical Three Crows   
    talib_fn!(
        CDLIDENTICAL3CROWS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(identical_three_crows: i32)],
    );

    // CDLINNECK            In-Neck Pattern   
    talib_fn!(
        CDLINNECK,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(in_neck: i32)],
    );

    // CDLINVERTEDHAMMER    Inverted Hammer   
    talib_fn!(
        CDLINVERTEDHAMMER,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(inverted_hammer: i32)],
    );

    // CDLKICKING           Kicking
    talib_fn!(
        CDLKICKING,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(kicking: i32)],
    );

    // CDLKICKINGBYLENGTH   Kicking - bull/bear determined by the longer marubozu   
    talib_fn!(
        CDLKICKINGBYLENGTH,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(kicking_by_length: i32)],
    );

    // CDLLADDERBOTTOM      Ladder Bottom   
    talib_fn!(
        CDLLADDERBOTTOM,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(ladder_bottom: i32)],
    );

    // CDLLONGLEGGEDDOJI    Long Legged Doji
    talib_fn!(
        CDLLONGLEGGEDDOJI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(long_legged_doji: i32)],
    );

    // CDLLONGLINE          Long Line Candle
    talib_fn!(
        CDLLONGLINE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(long_line: i32)],
    );

    // CDLMARUBOZU          Marubozu
    talib_fn!(
        CDLMARUBOZU,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(marubozu: i32)],
    );

    // CDLMATCHINGLOW       Matching Low   
    talib_fn!(
        CDLMATCHINGLOW,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(matching_low: i32)],
    );

    // CDLMATHOLD           Mat Hold   
    talib_fn!(
        CDLMATHOLD,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(mat_hold: i32)],
    );

    // CDLMORNINGDOJISTAR   Morning Doji Star   
    talib_fn!(
        CDLMORNINGDOJISTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(morning_doji_star: i32)],
    );

    // CDLMORNINGSTAR       Morning Star   
    talib_fn!(
        CDLMORNINGSTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(morning_star: i32)],
    );

    // CDLONNECK            On-Neck Pattern   
    talib_fn!(
        CDLONNECK,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(on_neck: i32)],
    );

    // CDLPIERCING          Piercing Pattern   
    talib_fn!(
        CDLPIERCING,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(piercing: i32)],
    );

    // CDLRICKSHAWMAN       Rickshaw Man
    talib_fn!(
        CDLRICKSHAWMAN,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(rickshaw_man: i32)],
    );

    // CDLRISEFALL3METHODS  Rising/Falling Three Methods   
    talib_fn!(
        CDLRISEFALL3METHODS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(rise_fall_three_methods: i32)],
    );

    // CDLSEPARATINGLINES   Separating Lines   
    talib_fn!(
        CDLSEPARATINGLINES,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(separating_lines: i32)],
    );

    // CDLSHOOTINGSTAR      Shooting Star   
    talib_fn!(
        CDLSHOOTINGSTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(shooting_star: i32)],
    );

    // CDLSHORTLINE         Short Line Candle   
    talib_fn!(
        CDLSHORTLINE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(short_line: i32)],
    );

    // CDLSPINNINGTOP       Spinning Top
    talib_fn!(
        CDLSPINNINGTOP,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(spinning_top: i32)],
    );

    // CDLSTALLEDPATTERN    Stalled Pattern   
    talib_fn!(
        CDLSTALLEDPATTERN,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(stalled_pattern: i32)],
    );

    // CDLSTICKSANDWICH     Stick Sandwich   
    talib_fn!(
        CDLSTICKSANDWICH,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(stick_sandwich: i32)],
    );

    // CDLTAKURI            Takuri (Dragonfly Doji with very long lower shadow)   
    talib_fn!(
        CDLTAKURI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(takuri: i32)],
    );

    // CDLTASUKIGAP         Tasuki Gap
    talib_fn!(
        CDLTASUKIGAP,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(tasuki_gap: i32)],
    );

    // CDLTHRUSTING         Thrusting Pattern   
    talib_fn!(
        CDLTHRUSTING,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(thrusting: i32)],
    );

    // CDLTRISTAR           Tristar Pattern   
    talib_fn!(
        CDLTRISTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(tristar: i32)],
    );

    // CDLUNIQUE3RIVER      Unique 3 River
    talib_fn!(
        CDLUNIQUE3RIVER,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(unique_three_river: i32)],
    );

    // CDLUPSIDEGAP2CROWS   Upside Gap Two Crows
    talib_fn!(
        CDLUPSIDEGAP2CROWS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(upside_gap_two_crows: i32)],
    );

    // CDLXSIDEGAP3METHODS  Upside/Downside Gap Three Methods   
    talib_fn!(
        CDLXSIDEGAP3METHODS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(xside_gap_three_methods: i32)],
    );
}
