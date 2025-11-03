use crate::talib::TALib;
use crate::talib_bindings::*;
// use crate::indicator_engine::talib_error::TalibError;
use crate::talib_fn;
use chrono::{DateTime, Utc};
use crate::Indicator;
use crate::indicator::pattern_recognition::*;

impl TALib {
    // CDL2CROWS            Two Crows #两只乌鸦
    talib_fn!(
        CDL2CROWS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(two_crows: i32)],
    );

    // CDL3BLACKCROWS       Three Black Crows #三只乌鸦
    talib_fn!(
        CDL3BLACKCROWS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_black_crows: i32)],
    );

    // CDL3INSIDE           Three Inside Up/Down #三内部上涨/下跌
    talib_fn!(
        CDL3INSIDE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_inside: i32)],
    );

    // CDL3LINESTRIKE       Three-Line Strike #三线打击
    talib_fn!(
        CDL3LINESTRIKE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_line_strike: i32)],
    );

    // CDL3OUTSIDE          Three Outside Up/Down #三外部上涨/下跌
    talib_fn!(
        CDL3OUTSIDE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_outside: i32)],
    );

    // CDL3STARSINSOUTH     Three Stars In The South #三颗星在南方
    talib_fn!(
        CDL3STARSINSOUTH,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_stars_in_south: i32)],
    );

    // CDL3WHITESOLDIERS    Three Advancing White Soldiers #三只白兵
    talib_fn!(
        CDL3WHITESOLDIERS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(three_white_soldiers: i32)],
    );

    // CDLABANDONEDBABY     Abandoned Baby #弃婴
    talib_fn!(
        CDLABANDONEDBABY,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(abandoned_baby: i32)],
    );

    // CDLADVANCEBLOCK      Advance Block #前进阻挡
    talib_fn!(
        CDLADVANCEBLOCK,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(advance_block: i32)],
    );

    // CDLBELTHOLD          Belt-hold #带柄
    talib_fn!(
        CDLBELTHOLD,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(belt_hold: i32)],
    );

    // CDLBREAKAWAY         Breakaway #突破
    talib_fn!(
        CDLBREAKAWAY,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(breakaway: i32)],
    );

    // CDLCLOSINGMARUBOZU   Closing Marubozu #收盘十字星
    talib_fn!(
        CDLCLOSINGMARUBOZU,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(closing_marubozu: i32)],
    );

    // CDLCONCEALBABYSWALL  Concealing Baby Swallow #隐藏婴儿吞噬
    talib_fn!(
        CDLCONCEALBABYSWALL,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(conceal_baby_swallow: i32)],
    );

    // CDLCOUNTERATTACK     Counterattack #反击
    talib_fn!(
        CDLCOUNTERATTACK,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(counterattack: i32)],
    );

    // CDLDARKCLOUDCOVER    Dark Cloud Cover #乌云盖顶
    talib_fn!(
        CDLDARKCLOUDCOVER,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(dark_cloud_cover: i32)],
    );

    // CDLDOJI              Doji #十字星
    talib_fn!(
        CDLDOJI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(doji: i32)],
    );

    // CDLDOJISTAR          Doji Star #十字星
    talib_fn!(
        CDLDOJISTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(doji_star: i32)],
    );

    // CDLDRAGONFLYDOJI     Dragonfly Doji #蜻蜓十字星
    talib_fn!(
        CDLDRAGONFLYDOJI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(dragonfly_doji: i32)],
    );

    // CDLENGULFING         Engulfing Pattern #吞噬模式
    talib_fn!(
        CDLENGULFING,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(engulfing: i32)],
    );

    // CDLEVENINGDOJISTAR   Evening Doji Star #黄昏十字星
    talib_fn!(
        CDLEVENINGDOJISTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(evening_doji_star: i32)],
    );

    // CDLEVENINGSTAR       Evening Star #黄昏星
    talib_fn!(
        CDLEVENINGSTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(evening_star: i32)],
    );

    // CDLGAPSIDESIDEWHITE  Up/Down-gap side-by-side white lines #上/下缺口侧边白线
    talib_fn!(
        CDLGAPSIDESIDEWHITE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(gap_side_side_white: i32)],
    );

    // CDLGRAVESTONEDOJI    Gravestone Doji #墓碑十字星
    talib_fn!(
        CDLGRAVESTONEDOJI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(gravestone_doji: i32)],
    );

    // CDLHAMMER            Hammer #锤子
    talib_fn!(
        CDLHAMMER,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(hammer: i32)],
    );

    // CDLHANGINGMAN        Hanging Man #吊人
    talib_fn!(
        CDLHANGINGMAN,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(hanging_man: i32)],
    );

    // CDLHARAMI            Harami Pattern #孕线模式
    talib_fn!(
        CDLHARAMI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(harami: i32)],
    );

    // CDLHARAMICROSS       Harami Cross Pattern #孕线交叉模式
    talib_fn!(
        CDLHARAMICROSS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(harami_cross: i32)],
    );

    // CDLHIGHWAVE          High-Wave Candle #高浪烛
    talib_fn!(
        CDLHIGHWAVE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(high_wave: i32)],
    );

    // CDLHIKKAKE           Hikkake Pattern #跳空模式
    talib_fn!(
        CDLHIKKAKE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(hikkake: i32)],
    );

    // CDLHIKKAKEMOD        Modified Hikkake Pattern #修改跳空模式
    talib_fn!(
        CDLHIKKAKEMOD,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(hikkake_mod: i32)],
    );

    // CDLHOMINGPIGEON      Homing Pigeon #归巢鸽
    talib_fn!(
        CDLHOMINGPIGEON,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(homing_pigeon: i32)],
    );

    // CDLIDENTICAL3CROWS   Identical Three Crows #相同三只乌鸦
    talib_fn!(
        CDLIDENTICAL3CROWS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(identical_three_crows: i32)],
    );

    // CDLINNECK            In-Neck Pattern #颈线模式
    talib_fn!(
        CDLINNECK,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(in_neck: i32)],
    );

    // CDLINVERTEDHAMMER    Inverted Hammer #倒锤子
    talib_fn!(
        CDLINVERTEDHAMMER,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(inverted_hammer: i32)],
    );

    // CDLKICKING           Kicking #踢腿
    talib_fn!(
        CDLKICKING,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(kicking: i32)],
    );

    // CDLKICKINGBYLENGTH   Kicking - bull/bear determined by the longer marubozu #踢腿 - 牛/熊由更长的十字星决定
    talib_fn!(
        CDLKICKINGBYLENGTH,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(kicking_by_length: i32)],
    );

    // CDLLADDERBOTTOM      Ladder Bottom #梯底
    talib_fn!(
        CDLLADDERBOTTOM,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(ladder_bottom: i32)],
    );

    // CDLLONGLEGGEDDOJI    Long Legged Doji #长脚十字星
    talib_fn!(
        CDLLONGLEGGEDDOJI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(long_legged_doji: i32)],
    );

    // CDLLONGLINE          Long Line Candle #长蜡烛
    talib_fn!(
        CDLLONGLINE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(long_line: i32)],
    );

    // CDLMARUBOZU          Marubozu #实体蜡烛
    talib_fn!(
        CDLMARUBOZU,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(marubozu: i32)],
    );

    // CDLMATCHINGLOW       Matching Low #匹配低点
    talib_fn!(
        CDLMATCHINGLOW,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(matching_low: i32)],
    );

    // CDLMATHOLD           Mat Hold #支撑
    talib_fn!(
        CDLMATHOLD,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(mat_hold: i32)],
    );

    // CDLMORNINGDOJISTAR   Morning Doji Star #早晨十字星
    talib_fn!(
        CDLMORNINGDOJISTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(morning_doji_star: i32)],
    );

    // CDLMORNINGSTAR       Morning Star #早晨之星
    talib_fn!(
        CDLMORNINGSTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(penetration: f64)],
        output => [(morning_star: i32)],
    );

    // CDLONNECK            On-Neck Pattern #颈线模式
    talib_fn!(
        CDLONNECK,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(on_neck: i32)],
    );

    // CDLPIERCING          Piercing Pattern #刺透模式
    talib_fn!(
        CDLPIERCING,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(piercing: i32)],
    );

    // CDLRICKSHAWMAN       Rickshaw Man #人力车夫
    talib_fn!(
        CDLRICKSHAWMAN,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(rickshaw_man: i32)],
    );

    // CDLRISEFALL3METHODS  Rising/Falling Three Methods #上升/下降三法
    talib_fn!(
        CDLRISEFALL3METHODS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(rise_fall_three_methods: i32)],
    );

    // CDLSEPARATINGLINES   Separating Lines #分离线
    talib_fn!(
        CDLSEPARATINGLINES,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(separating_lines: i32)],
    );

    // CDLSHOOTINGSTAR      Shooting Star #射击之星
    talib_fn!(
        CDLSHOOTINGSTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(shooting_star: i32)],
    );

    // CDLSHORTLINE         Short Line Candle #短蜡烛
    talib_fn!(
        CDLSHORTLINE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(short_line: i32)],
    );

    // CDLSPINNINGTOP       Spinning Top #旋转顶部
    talib_fn!(
        CDLSPINNINGTOP,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(spinning_top: i32)],
    );

    // CDLSTALLEDPATTERN    Stalled Pattern #停滞模式
    talib_fn!(
        CDLSTALLEDPATTERN,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(stalled_pattern: i32)],
    );

    // CDLSTICKSANDWICH     Stick Sandwich #针刺三明治
    talib_fn!(
        CDLSTICKSANDWICH,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(stick_sandwich: i32)],
    );

    // CDLTAKURI            Takuri (Dragonfly Doji with very long lower shadow) #蜻蜓十字星
    talib_fn!(
        CDLTAKURI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(takuri: i32)],
    );

    // CDLTASUKIGAP         Tasuki Gap #田中缺口
    talib_fn!(
        CDLTASUKIGAP,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(tasuki_gap: i32)],
    );

    // CDLTHRUSTING         Thrusting Pattern #刺穿模式
    talib_fn!(
        CDLTHRUSTING,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(thrusting: i32)],
    );

    // CDLTRISTAR           Tristar Pattern #三星模式
    talib_fn!(
        CDLTRISTAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(tristar: i32)],
    );

    // CDLUNIQUE3RIVER      Unique 3 River #唯一三河
    talib_fn!(
        CDLUNIQUE3RIVER,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(unique_three_river: i32)],
    );

    // CDLUPSIDEGAP2CROWS   Upside Gap Two Crows #上缺口两只乌鸦
    talib_fn!(
        CDLUPSIDEGAP2CROWS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(upside_gap_two_crows: i32)],
    );

    // CDLXSIDEGAP3METHODS  Upside/Downside Gap Three Methods #上/下缺口三法
    talib_fn!(
        CDLXSIDEGAP3METHODS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(xside_gap_three_methods: i32)],
    );
}
