use crate::calculate_fn;
use crate::indicator_engine::calculate::CalculateIndicatorFunction;
use crate::indicator_engine::talib::TALib;
use star_river_core::cache::CacheValue;
use star_river_core::error::engine_error::indicator_engine_error::*;
use star_river_core::indicator::Indicator;
use star_river_core::market::Kline;
use star_river_core::indicator::indicator_define::pattern_recognition::*;
use std::sync::Arc;

impl CalculateIndicatorFunction {
    // CDL2CROWS - Two Crows #两只乌鸦
    calculate_fn!(CDL2CROWS,
        input => [open,high,low,close]
    );

    // CDL3BLACKCROWS - Three Black Crows #三只乌鸦
    calculate_fn!(CDL3BLACKCROWS,
        input => [open,high,low,close]
    );

    // CDL3INSIDE - Three Inside Up/Down #三内部上涨/下跌
    calculate_fn!(CDL3INSIDE,
        input => [open,high,low,close]
    );

    // CDL3LINESTRIKE - Three-Line Strike #三线打击
    calculate_fn!(CDL3LINESTRIKE,
        input => [open,high,low,close]
    );

    // CDL3OUTSIDE - Three Outside Up/Down #三外部上涨/下跌
    calculate_fn!(CDL3OUTSIDE,
        input => [open,high,low,close]
    );

    // CDL3STARSINSOUTH - Three Stars In The South #三颗星在南方
    calculate_fn!(CDL3STARSINSOUTH,
        input => [open,high,low,close]
    );

    // CDL3WHITESOLDIERS - Three Advancing White Soldiers #三只白兵
    calculate_fn!(CDL3WHITESOLDIERS,
        input => [open,high,low,close]
    );

    // CDLABANDONEDBABY - Abandoned Baby #弃婴
    calculate_fn!(CDLABANDONEDBABY,
        input => [open,high,low,close],
        talib_params => [(penetration: f64)]
    );

    // CDLADVANCEBLOCK - Advance Block #前进阻挡
    calculate_fn!(CDLADVANCEBLOCK,
        input => [open,high,low,close]
    );

    // CDLBELTHOLD - Belt-hold #带柄
    calculate_fn!(CDLBELTHOLD,
        input => [open,high,low,close]
    );

    // CDLBREAKAWAY - Breakaway #突破
    calculate_fn!(CDLBREAKAWAY,
        input => [open,high,low,close]
    );

    // CDLCLOSINGMARUBOZU - Closing Marubozu #收盘十字星
    calculate_fn!(CDLCLOSINGMARUBOZU,
        input => [open,high,low,close]
    );

    // CDLCONCEALBABYSWALL - Concealing Baby Swallow #隐藏婴儿吞噬
    calculate_fn!(CDLCONCEALBABYSWALL,
        input => [open,high,low,close]
    );

    // CDLCOUNTERATTACK - Counterattack #反击
    calculate_fn!(CDLCOUNTERATTACK,
        input => [open,high,low,close]
    );

    // CDLDARKCLOUDCOVER - Dark Cloud Cover #乌云盖顶
    calculate_fn!(CDLDARKCLOUDCOVER,
        input => [open,high,low,close],
        talib_params => [(penetration: f64)]
    );

    // CDLDOJI - Doji #十字星
    calculate_fn!(CDLDOJI,
        input => [open,high,low,close]
    );

    // CDLDOJISTAR - Doji Star #十字星
    calculate_fn!(CDLDOJISTAR,
        input => [open, high, low, close]
    );

    // CDLDRAGONFLYDOJI - Dragonfly Doji #蜻蜓十字星
    calculate_fn!(CDLDRAGONFLYDOJI,
        input => [open, high, low, close]
    );

    // CDLENGULFING - Engulfing Pattern #吞噬模式
    calculate_fn!(CDLENGULFING,
        input => [open, high, low, close]
    );

    // CDLEVENINGDOJISTAR - Evening Doji Star #黄昏十字星
    calculate_fn!(CDLEVENINGDOJISTAR,
        input => [open, high, low, close],
        talib_params => [(penetration: f64)]
    );

    // CDLEVENINGSTAR - Evening Star #黄昏星
    calculate_fn!(CDLEVENINGSTAR,
        input => [open, high, low, close],
        talib_params => [(penetration: f64)]
    );

    // CDLGAPSIDESIDEWHITE - Up/Down-gap side-by-side white lines #上/下缺口侧边白线
    calculate_fn!(CDLGAPSIDESIDEWHITE,
        input => [open, high, low, close]
    );

    // CDLGRAVESTONEDOJI - Gravestone Doji #墓碑十字星
    calculate_fn!(CDLGRAVESTONEDOJI,
        input => [open, high, low, close]
    );

    // CDLHAMMER - Hammer #锤子
    calculate_fn!(CDLHAMMER,
        input => [open, high, low, close]
    );

    // CDLHANGINGMAN - Hanging Man #吊人
    calculate_fn!(CDLHANGINGMAN,
        input => [open, high, low, close]
    );

    // CDLHARAMI - Harami Pattern #孕线模式
    calculate_fn!(CDLHARAMI,
        input => [open, high, low, close]
    );

    // CDLHARAMICROSS - Harami Cross Pattern #孕线交叉模式
    calculate_fn!(CDLHARAMICROSS,
        input => [open, high, low, close]
    );

    // CDLHIGHWAVE - High-Wave Candle #高浪烛
    calculate_fn!(CDLHIGHWAVE,
        input => [open, high, low, close]
    );

    // CDLHIKKAKE - Hikkake Pattern #跳空模式
    calculate_fn!(CDLHIKKAKE,
        input => [open, high, low, close]
    );

    // CDLHIKKAKEMOD - Modified Hikkake Pattern #修改跳空模式
    calculate_fn!(CDLHIKKAKEMOD,
        input => [open, high, low, close]
    );

    // CDLHOMINGPIGEON - Homing Pigeon #归巢鸽
    calculate_fn!(CDLHOMINGPIGEON,
        input => [open, high, low, close]
    );

    // CDLIDENTICAL3CROWS - Identical Three Crows #相同三只乌鸦
    calculate_fn!(CDLIDENTICAL3CROWS,
        input => [open, high, low, close]
    );

    // CDLINNECK - In-Neck Pattern #颈线模式
    calculate_fn!(CDLINNECK,
        input => [open, high, low, close]
    );

    // CDLINVERTEDHAMMER - Inverted Hammer #倒锤子
    calculate_fn!(CDLINVERTEDHAMMER,
        input => [open, high, low, close]
    );

    // CDLKICKING - Kicking #踢腿
    calculate_fn!(CDLKICKING,
        input => [open, high, low, close]
    );

    // CDLKICKINGBYLENGTH - Kicking - bull/bear determined by the longer marubozu #踢腿（由更长的实体决定）
    calculate_fn!(CDLKICKINGBYLENGTH,
        input => [open, high, low, close]
    );

    // CDLLADDERBOTTOM - Ladder Bottom #梯底
    calculate_fn!(CDLLADDERBOTTOM,
        input => [open, high, low, close]
    );

    // CDLLONGLEGGEDDOJI - Long Legged Doji #长脚十字星
    calculate_fn!(CDLLONGLEGGEDDOJI,
        input => [open, high, low, close]
    );

    // CDLLONGLINE - Long Line Candle #长蜡烛
    calculate_fn!(CDLLONGLINE,
        input => [open, high, low, close]
    );

    // CDLMARUBOZU - Marubozu #实体蜡烛
    calculate_fn!(CDLMARUBOZU,
        input => [open, high, low, close]
    );

    // CDLMATCHINGLOW - Matching Low #匹配低点
    calculate_fn!(CDLMATCHINGLOW,
        input => [open, high, low, close]
    );

    // CDLMATHOLD - Mat Hold #支撑
    calculate_fn!(CDLMATHOLD,
        input => [open, high, low, close],
        talib_params => [(penetration: f64)]
    );

    // CDLMORNINGDOJISTAR - Morning Doji Star #早晨十字星
    calculate_fn!(CDLMORNINGDOJISTAR,
        input => [open, high, low, close],
        talib_params => [(penetration: f64)]
    );

    // CDLMORNINGSTAR - Morning Star #早晨之星
    calculate_fn!(CDLMORNINGSTAR,
        input => [open, high, low, close],
        talib_params => [(penetration: f64)]
    );

    // CDLONNECK - On-Neck Pattern #颈线模式
    calculate_fn!(CDLONNECK,
        input => [open, high, low, close]
    );

    // CDLPIERCING - Piercing Pattern #刺透模式
    calculate_fn!(CDLPIERCING,
        input => [open, high, low, close]
    );

    // CDLRICKSHAWMAN - Rickshaw Man #人力车夫
    calculate_fn!(CDLRICKSHAWMAN,
        input => [open, high, low, close]
    );

    // CDLRISEFALL3METHODS - Rising/Falling Three Methods #上升/下降三法
    calculate_fn!(CDLRISEFALL3METHODS,
        input => [open, high, low, close]
    );

    // CDLSEPARATINGLINES - Separating Lines #分离线
    calculate_fn!(CDLSEPARATINGLINES,
        input => [open, high, low, close]
    );

    // CDLSHOOTINGSTAR - Shooting Star #射击之星
    calculate_fn!(CDLSHOOTINGSTAR,
        input => [open, high, low, close]
    );

    // CDLSHORTLINE - Short Line Candle #短蜡烛
    calculate_fn!(CDLSHORTLINE,
        input => [open, high, low, close]
    );

    // CDLSPINNINGTOP - Spinning Top #旋转顶部
    calculate_fn!(CDLSPINNINGTOP,
        input => [open, high, low, close]
    );

    // CDLSTALLEDPATTERN - Stalled Pattern #停滞模式
    calculate_fn!(CDLSTALLEDPATTERN,
        input => [open, high, low, close]
    );

    // CDLSTICKSANDWICH - Stick Sandwich #针刺三明治
    calculate_fn!(CDLSTICKSANDWICH,
        input => [open, high, low, close]
    );

    // CDLTAKURI - Takuri (Dragonfly Doji with very long lower shadow) #蜻蜓十字星（带很长下影线）
    calculate_fn!(CDLTAKURI,
        input => [open, high, low, close]
    );

    // CDLTASUKIGAP - Tasuki Gap #田中缺口
    calculate_fn!(CDLTASUKIGAP,
        input => [open, high, low, close]
    );

    // CDLTHRUSTING - Thrusting Pattern #刺穿模式
    calculate_fn!(CDLTHRUSTING,
        input => [open, high, low, close]
    );

    // CDLTRISTAR - Tristar Pattern #三星模式
    calculate_fn!(CDLTRISTAR,
        input => [open, high, low, close]
    );

    // CDLUNIQUE3RIVER - Unique 3 River #唯一三河
    calculate_fn!(CDLUNIQUE3RIVER,
        input => [open, high, low, close]
    );

    // CDLUPSIDEGAP2CROWS - Upside Gap Two Crows #上缺口两只乌鸦
    calculate_fn!(CDLUPSIDEGAP2CROWS,
        input => [open, high, low, close]
    );

    // CDLXSIDEGAP3METHODS - Upside/Downside Gap Three Methods #上/下缺口三法
    calculate_fn!(CDLXSIDEGAP3METHODS,
        input => [open, high, low, close]
    );
}
