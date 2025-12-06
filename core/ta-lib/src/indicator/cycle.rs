use chrono::{DateTime, Utc};

use crate::{define_indicator, indicator::PriceSource};

// HT_DCPERIOD          Hilbert Transform - Dominant Cycle Period
define_indicator!(HtDcperiod,
    params => [(price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (ht_dcperiod: Option<f64>)],
);

// HT_DCPHASE           Hilbert Transform - Dominant Cycle Phase
define_indicator!(HtDcphase,
    params => [(price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (ht_dcphase: Option<f64>)],
);

// HT_PHASOR            Hilbert Transform - Phasor Components
define_indicator!(HtPhasor,
    params => [(price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (in_phase: Option<f64>), (quadrature: Option<f64>)],
);

// HT_SINE              Hilbert Transform - SineWave
define_indicator!(HtSine,
    params => [(price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (sine: Option<f64>), (lead_sine: Option<f64>)],
);

// HT_TRENDMODE         Hilbert Transform - Trend vs Cycle Mode
define_indicator!(HtTrendmode,
    params => [(price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (ht_trendmode: Option<i32>)],
);
