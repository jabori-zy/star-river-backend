use super::{
    BacktestStrategyContext, BacktestStrategyError, CustomVariable, Indicator, IndicatorKey, KeyTrait, Kline, KlineKey,
    KlineKeyNotFoundSnafu, PlayIndexOutOfRangeSnafu, QuantData, strategy_utils::apply_variable_operation,
};

mod kline {

    use super::*;

    impl BacktestStrategyContext {
        pub async fn init_kline_data(&mut self, kline_key: &KlineKey, init_kline_data: Vec<Kline>) {
            // 初始化k线数据
            let mut kline_data_guard = self.kline_data.write().await;
            if let Some(kline_data) = kline_data_guard.get(kline_key) {
                if kline_data.len() == 0 {
                    kline_data_guard.insert(kline_key.clone(), init_kline_data);
                }
            } else {
                kline_data_guard.insert(kline_key.clone(), init_kline_data);
            }
        }

        pub async fn append_kline_data(&mut self, kline_key: &KlineKey, kline_series: Vec<Kline>) {
            let mut kline_data_guard = self.kline_data.write().await;
            if let Some(kline_data) = kline_data_guard.get_mut(kline_key) {
                kline_data.extend(kline_series);
                // 按时间戳排序，确保K线数据的时序正确性
                kline_data.sort_by(|a, b| a.datetime().cmp(&b.datetime()));
                // 去重：移除相同datetime的重复数据，保留最后一个
                kline_data.dedup_by(|a, b| a.datetime() == b.datetime());
            } else {
                // 新插入的数据也需要排序和去重
                let mut sorted_series = kline_series;
                sorted_series.sort_by(|a, b| a.datetime().cmp(&b.datetime()));
                sorted_series.dedup_by(|a, b| a.datetime() == b.datetime());
                kline_data_guard.insert(kline_key.clone(), sorted_series);
            }
        }

        pub async fn get_kline_slice(
            &self,
            kline_key: &KlineKey,
            play_index: Option<i32>,
            limit: Option<i32>,
        ) -> Result<Vec<Kline>, BacktestStrategyError> {
            let kline_data = self.get_kline_data(kline_key).await;
            if let Some(data) = kline_data {
                let kline_data_length = data.len() as u32;
                match (play_index, limit) {
                    // 有index，有limit
                    (Some(play_index), Some(limit)) => {
                        // 如果索引超出范围，返回空
                        if play_index as u32 >= kline_data_length {
                            Err(PlayIndexOutOfRangeSnafu {
                                kline_data_length: kline_data_length,
                                play_index: play_index as u32,
                            }
                            .build())
                        } else {
                            // 计算从索引开始向前取limit个元素
                            let end = play_index as usize + 1;
                            let start = if limit as usize >= end { 0 } else { end - limit as usize };
                            Ok(data[start..end].to_vec())
                        }
                    }
                    // 有index，无limit
                    (Some(play_index), None) => {
                        // 如果索引超出范围，返回空
                        if play_index as u32 >= kline_data_length {
                            Err(PlayIndexOutOfRangeSnafu {
                                kline_data_length: kline_data_length,
                                play_index: play_index as u32,
                            }
                            .build())
                        } else {
                            // 从索引开始向前取所有元素（到开头）
                            let end = play_index as usize + 1;
                            Ok(data[0..end].to_vec())
                        }
                    }
                    // 无index，有limit
                    (None, Some(limit)) => {
                        // 从后往前取limit条数据
                        if limit as u32 >= kline_data_length {
                            Ok(data.clone())
                        } else {
                            let start = (kline_data_length as usize).saturating_sub(limit as usize);
                            Ok(data[start..].to_vec())
                        }
                    }
                    // 无index，无limit
                    (None, None) => {
                        // 如果limit和index都为None，则返回所有数据
                        Ok(data.clone())
                    }
                }
            } else {
                Err(KlineKeyNotFoundSnafu {
                    kline_key: kline_key.get_key_str(),
                }
                .build())
            }
        }

        pub async fn update_kline_data(&mut self, kline_key: &KlineKey, kline: &Kline) -> Kline {
            // 先检查键是否存在，释放锁
            let key_exists = { self.kline_data.read().await.contains_key(kline_key) };

            if !key_exists {
                // 如果缓存键不存在，先初始化空的Vec
                let mut kline_data_guard = self.kline_data.write().await;
                kline_data_guard.insert(kline_key.clone(), Vec::new());
            }

            // 重新获取锁并更新
            let mut kline_data_guard = self.kline_data.write().await;
            let kline_data = kline_data_guard.get_mut(kline_key).unwrap();

            if !key_exists || kline_data.len() == 0 {
                // 判断是否为初始化
                kline_data.clear();
                kline_data.push(kline.clone());
            } else {
                // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k线
                if let Some(last_kline) = kline_data.last() {
                    if last_kline.datetime() == kline.datetime() {
                        kline_data.pop();
                        kline_data.push(kline.clone());
                    } else {
                        // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
                        kline_data.push(kline.clone());
                    }
                } else {
                    kline_data.push(kline.clone());
                }
            }

            kline.clone()
        }
    }
}

mod indicator {

    use super::*;
    impl BacktestStrategyContext {
        pub async fn init_indicator_data(&mut self, indicator_key: &IndicatorKey, indicator_series: Vec<Indicator>) {
            // 初始化指标数据
            let mut indicator_data_guard = self.indicator_data.write().await;
            // 如果指标key存在
            if let Some(indicator_data) = indicator_data_guard.get(indicator_key) {
                // 如果指标数据为空，则初始化指标数据
                if indicator_data.len() == 0 {
                    indicator_data_guard.insert(indicator_key.clone(), indicator_series);
                }
            } else {
                // 如果指标key不存在，则初始化指标数据
                indicator_data_guard.insert(indicator_key.clone(), indicator_series);
            }
        }

        pub async fn get_indicator_slice(
            &self,
            indicator_key: &IndicatorKey,
            play_index: Option<i32>,
            limit: Option<i32>,
        ) -> Vec<Indicator> {
            let indicator_data_guard = self.indicator_data.read().await;
            if let Some(indicator_data) = indicator_data_guard.get(indicator_key) {
                match (play_index, limit) {
                    // 有index，有limit
                    (Some(play_index), Some(limit)) => {
                        // 如果索引超出范围，返回空Vec
                        if play_index as usize >= indicator_data.len() {
                            Vec::new()
                        } else {
                            // 计算从索引开始向前取limit个元素
                            let end = play_index as usize + 1;
                            let start = if limit as usize >= end { 0 } else { end - limit as usize };
                            indicator_data[start..end].to_vec()
                        }
                    }
                    // 有index，无limit
                    (Some(play_index), None) => {
                        // 如果索引超出范围，返回空Vec
                        if play_index as usize >= indicator_data.len() {
                            Vec::new()
                        } else {
                            // 从索引开始向前取所有元素（到开头）
                            let end = play_index as usize + 1;
                            indicator_data[0..end].to_vec()
                        }
                    }
                    // 无index，有limit
                    (None, Some(limit)) => {
                        // 从后往前取limit条数据
                        if limit as usize >= indicator_data.len() {
                            indicator_data.clone()
                        } else {
                            let start = indicator_data.len().saturating_sub(limit as usize);
                            indicator_data[start..].to_vec()
                        }
                    }
                    // 无index，无limit
                    (None, None) => {
                        // 如果limit和index都为None，则返回所有数据
                        indicator_data.clone()
                    }
                }
            } else {
                Vec::new()
            }
        }

        pub async fn update_indicator_data(&mut self, indicator_key: &IndicatorKey, indicator: &Indicator) -> Indicator {
            // 先检查键是否存在，释放锁
            let key_exists = { self.indicator_data.read().await.contains_key(indicator_key) };

            if !key_exists {
                // 如果缓存键不存在，先初始化空的Vec
                let mut indicator_data_guard = self.indicator_data.write().await;
                indicator_data_guard.insert(indicator_key.clone(), Vec::new());
            }

            // 重新获取锁并更新
            let mut indicator_data_guard = self.indicator_data.write().await;
            let indicator_data = indicator_data_guard.get_mut(indicator_key).unwrap();

            if !key_exists || indicator_data.len() == 0 {
                // 判断是否为初始化
                indicator_data.clear();
                indicator_data.push(indicator.clone());
            } else {
                // 如果最新的一条数据时间戳等于最后一个指标的时间戳，则更新最后一条指标
                if let Some(last_indicator) = indicator_data.last() {
                    if last_indicator.get_datetime() == indicator.get_datetime() {
                        indicator_data.pop();
                        indicator_data.push(indicator.clone());
                    } else {
                        // 如果最新的一条数据时间戳不等于最后一个指标的时间戳，则插入新数据
                        indicator_data.push(indicator.clone());
                    }
                } else {
                    indicator_data.push(indicator.clone());
                }
            }

            indicator.clone()
        }
    }
}

mod custom_variable {
    use super::*;
    use snafu::OptionExt;
    use star_river_core::error::strategy_error::backtest_strategy_error::CustomVariableNotExistSnafu;
    use star_river_core::node::variable_node::variable_config::UpdateVariableConfig;
    use star_river_core::strategy::sys_varibale::SysVariable;
    impl BacktestStrategyContext {
        pub async fn init_custom_variables(&mut self, custom_variables: Vec<CustomVariable>) {
            // tracing::info!("Initializing custom variables");
            // 初始化自定义变量
            let mut custom_var_guard = self.custom_variable.write().await;
            for custom_var in custom_variables {
                let var_name = custom_var.var_name.clone();
                // 如果变量不存在，则插入；如果已存在，则不做修改
                custom_var_guard.entry(var_name).or_insert(custom_var);
            }
            // 在写锁范围内直接使用 custom_var_guard 进行调试输出，避免死锁
            // tracing::debug!("custom_variable: {:#?}", *custom_var_guard);
        }

        pub async fn get_custom_variable_value(&mut self, var_name: String) -> Result<CustomVariable, BacktestStrategyError> {
            let custom_var_guard = self.custom_variable.read().await;
            let custom_variable = custom_var_guard
                .get(var_name.as_str())
                .context(CustomVariableNotExistSnafu { var_name })?;
            Ok(custom_variable.clone())
        }

        pub async fn update_custom_variable_value(
            &mut self,
            update_var_config: &UpdateVariableConfig,
        ) -> Result<CustomVariable, BacktestStrategyError> {
            let var_name = update_var_config.var_name.clone();
            let operation = &update_var_config.update_var_value_operation;

            let mut custom_var_guard = self.custom_variable.write().await;

            let custom_var = custom_var_guard.get_mut(&var_name).context(CustomVariableNotExistSnafu {
                var_name: var_name.clone(),
            })?;

            // 使用工具函数计算新值
            let new_value = apply_variable_operation(
                &var_name,
                &custom_var.var_value,
                operation,
                update_var_config.update_operation_value.as_ref(),
            )?;
            // 更新前一个值
            custom_var.previous_value = custom_var.var_value.clone();
            // 更新当前值
            custom_var.var_value = new_value.clone();
            Ok(custom_var.clone())
        }

        pub async fn reset_custom_variables(&mut self, var_name: String) -> Result<CustomVariable, BacktestStrategyError> {
            let mut custom_var_guard = self.custom_variable.write().await;
            // 直接获取可变引用，避免重复查找
            let custom_var = custom_var_guard
                .get_mut(&var_name)
                .context(CustomVariableNotExistSnafu { var_name })?;
            // 将变量值重置为初始值
            custom_var.var_value = custom_var.initial_value.clone();

            Ok(custom_var.clone())
        }

        /// 更新系统变量的值
        ///
        /// # 参数
        /// - `sys_variable`: 系统变量
        ///
        /// # 返回
        /// 返回更新后的变量值
        pub async fn update_sys_variable(&mut self, sys_variable: &SysVariable) {
            let mut sys_var_guard = self.sys_variable.write().await;
            // 插入或更新系统变量
            sys_var_guard.insert(sys_variable.var_name.clone(), sys_variable.clone());
        }
    }
}
