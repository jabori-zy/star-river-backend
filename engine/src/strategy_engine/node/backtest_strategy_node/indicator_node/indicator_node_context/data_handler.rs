use super::{
    IndicatorNodeContext,
    IndicatorKey,
    Kline,
    QuantData,
};


impl IndicatorNodeContext {
    pub(super) async fn update_kline_data(&mut self, indicator_key: IndicatorKey, kline_data: Kline) {
        // 如果指标缓存键不存在，则直接插入
        if !self.kline_value.contains_key(&indicator_key) {
            self.kline_value.insert(indicator_key.clone(), vec![kline_data]);
            return;
        }

        // 如果指标缓存键存在，则更新
        if let Some(kline_list) = self.kline_value.get_mut(&indicator_key) {
            if let Some(last_kline) = kline_list.last() {
                // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k线
                if last_kline.get_datetime() == kline_data.get_datetime() {
                    kline_list.pop();
                    kline_list.push(kline_data);
                } else {
                    // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
                    kline_list.push(kline_data);

                    // 检查是否需要限制长度
                    if let Some(lookback) = self.indicator_lookback.get(&indicator_key) {
                        if kline_list.len() > *lookback + 1 {
                            kline_list.remove(0);
                        }
                    }
                }
            } else {
                // 如果列表为空，直接插入
                kline_list.push(kline_data);
            }
        }
    }
}