use crate::{define_indicator, indicator::PriceSource};


// HT_DCPERIOD          Hilbert Transform - Dominant Cycle Period #希尔伯特变换 - 主导周期
define_indicator!(HtDcPeriod,
    params => [(price_source: PriceSource)],
    output => [(timestamp: i64), (ht_dcperiod: f64)],
);

// HT_DCPHASE           Hilbert Transform - Dominant Cycle Phase #希尔伯特变换 - 主导周期相位
define_indicator!(HtDcPhase,
    params => [(price_source: PriceSource)],
    output => [(timestamp: i64), (ht_dcphase: f64)],
);

// HT_PHASOR            Hilbert Transform - Phasor Components #希尔伯特变换 - 相量分量
define_indicator!(HtPhasor,
    params => [(price_source: PriceSource)],
    output => [(timestamp: i64), (in_phase: f64), (quadrature: f64)],
);

// HT_SINE              Hilbert Transform - SineWave #希尔伯特变换 - 正弦波
define_indicator!(HtSine,
    params => [(price_source: PriceSource)],
    output => [(timestamp: i64), (sine: f64), (lead_sine: f64)],
);


// HT_TRENDMODE         Hilbert Transform - Trend vs Cycle Mode #希尔伯特变换 - 趋势与周期模式
define_indicator!(HtTrendMode,
    params => [(price_source: PriceSource)],
    output => [(timestamp: i64), (ht_trendmode: i32)],
);






