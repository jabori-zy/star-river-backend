use super::CacheEngineContext;
use event_center::communication::Command;
use event_center::communication::engine::cache_engine::*;
use star_river_core::key::KeyTrait;
use star_river_core::market::QuantData;
use std::sync::Arc;

mod kline {
    use super::*;

    impl CacheEngineContext {
        pub async fn handle_add_kline_key(&mut self, cmd: AddKlineKeyCommand) {
            self.add_kline_key(cmd.strategy_id, cmd.key.clone(), cmd.max_size, cmd.duration)
                .await;
            let payload = AddKlineKeyRespPayload::new(cmd.key.clone());
            let response = AddKlineKeyResponse::success(Some(payload));
            cmd.respond(response);
        }

        pub async fn handle_get_kline_cache(&self, cmd: GetKlineCacheCommand) {
            let result = self.get_kline_cache(&cmd.key, cmd.index, cmd.limit).await;
            match result {
                Ok(data) => {
                    let payload = GetKlineCacheRespPayload::new(data);
                    let response = GetKlineCacheResponse::success(Some(payload));
                    cmd.respond(response);
                }
                Err(error) => {
                    let response = GetKlineCacheResponse::error(Arc::new(error));
                    cmd.respond(response);
                }
            }
        }

        pub async fn handle_get_kline_cache_multi(&self, cmd: GetKlineCacheMultiCommand) {
            let result = self.get_kline_cache_multi(&cmd.keys, cmd.index, cmd.limit).await;
            match result {
                Ok(multi_data) => {
                    let multi_data_result = multi_data
                        .into_iter()
                        .map(|(cache_key, data)| {
                            (
                                cache_key.get_key_str(),
                                data.into_iter().map(|cache_value| cache_value.to_list()).collect(),
                            )
                        })
                        .collect();
                    let payload = GetKlineCacheMultiRespPayload::new(multi_data_result);
                    let response = GetKlineCacheMultiResponse::success(Some(payload));
                    cmd.respond(response);
                }
                Err(error) => {
                    let response = GetKlineCacheMultiResponse::error(Arc::new(error));
                    cmd.respond(response);
                }
            }
        }

        pub async fn handle_get_kline_cache_length_multi(&self, cmd: GetKlineCacheLengthMultiCommand) {
            let result = self.get_kline_cache_length_multi(&cmd.keys).await;
            match result {
                Ok(length_result) => {
                    let payload = GetKlineCacheLengthMultiRespPayload::new(cmd.keys.clone(), length_result);
                    let response = GetKlineCacheLengthMultiResponse::success(Some(payload));
                    cmd.respond(response);
                }
                Err(error) => {
                    let response = GetKlineCacheLengthMultiResponse::error(Arc::new(error));
                    cmd.respond(response);
                }
            }
        }

        pub async fn handle_update_kline_cache(&mut self, cmd: UpdateKlineCacheCommand) {
            let result = self
                .update_kline_cache(cmd.strategy_id, cmd.key.clone(), cmd.value.clone())
                .await;
            match result {
                Ok(()) => {
                    let payload = UpdateKlineCacheRespPayload::new(cmd.key.clone());
                    let response = UpdateKlineCacheResponse::success(Some(payload));
                    cmd.respond(response);
                }
                Err(error) => {
                    let response = UpdateKlineCacheResponse::error(Arc::new(error));
                    cmd.respond(response);
                }
            }
        }

        pub async fn handle_clear_kline_cache(&mut self, cmd: ClearKlineCacheCommand) {
            let result = self.clear_kline_cache(&cmd.key).await;
            match result {
                Ok(()) => {
                    let payload = ClearKlineCacheRespPayload::new(cmd.key.clone());
                    let response = ClearKlineCacheResponse::success(Some(payload));
                    cmd.respond(response);
                }
                Err(error) => {
                    let response = ClearKlineCacheResponse::error(Arc::new(error));
                    cmd.respond(response);
                }
            }
        }
    }
}

mod indicator {
    use super::*;

    impl CacheEngineContext {
        pub async fn handle_add_indicator_key(&mut self, cmd: AddIndicatorKeyCommand) {
            self.add_indicator_key(cmd.strategy_id, cmd.key.clone(), cmd.max_size, cmd.duration)
                .await;
            let payload = AddIndicatorKeyRespPayload::new(cmd.key.clone());
            let response = AddIndicatorKeyResponse::success(Some(payload));
            cmd.respond(response);
        }

        pub async fn handle_get_indicator_cache(&self, cmd: GetIndicatorCacheCommand) {
            let result = self.get_indicator_cache(&cmd.key, cmd.index, cmd.limit).await;
            match result {
                Ok(data) => {
                    let payload = GetIndicatorCacheRespPayload::new(data);
                    let response = GetIndicatorCacheResponse::success(Some(payload));
                    cmd.respond(response);
                }
                Err(error) => {
                    let response = GetIndicatorCacheResponse::error(Arc::new(error));
                    cmd.respond(response);
                }
            }
        }

        pub async fn handle_get_indicator_cache_multi(&self, cmd: GetIndicatorCacheMultiCommand) {
            let result = self.get_indicator_cache_multi(&cmd.keys, cmd.index, cmd.limit).await;
            match result {
                Ok(multi_data) => {
                    let multi_data_result = multi_data
                        .into_iter()
                        .map(|(cache_key, data)| {
                            (
                                cache_key.get_key_str(),
                                data.into_iter().map(|cache_value| cache_value.to_list()).collect(),
                            )
                        })
                        .collect();
                    let payload = GetIndicatorCacheMultiRespPayload::new(multi_data_result);
                    let response = GetIndicatorCacheMultiResponse::success(Some(payload));
                    cmd.respond(response);
                }
                Err(error) => {
                    let response = GetIndicatorCacheMultiResponse::error(Arc::new(error));
                    cmd.respond(response);
                }
            }
        }

        pub async fn handle_get_indicator_cache_length_multi(&self, cmd: GetIndicatorCacheLengthMultiCommand) {
            let result = self.get_indicator_cache_length_multi(&cmd.keys).await;
            match result {
                Ok(length_result) => {
                    let payload = GetIndicatorCacheLengthMultiRespPayload::new(cmd.keys.clone(), length_result);
                    let response = GetIndicatorCacheLengthMultiResponse::success(Some(payload));
                    cmd.respond(response);
                }
                Err(error) => {
                    let response = GetIndicatorCacheLengthMultiResponse::error(Arc::new(error));
                    cmd.respond(response);
                }
            }
        }

        pub async fn handle_update_indicator_cache(&mut self, cmd: UpdateIndicatorCacheCommand) {
            let result = self
                .update_indicator_cache(cmd.strategy_id, cmd.key.clone(), cmd.value.clone())
                .await;
            match result {
                Ok(()) => {
                    let payload = UpdateIndicatorCacheRespPayload::new(cmd.key.clone());
                    let response = UpdateIndicatorCacheResponse::success(Some(payload));
                    cmd.respond(response);
                }
                Err(error) => {
                    let response = UpdateIndicatorCacheResponse::error(Arc::new(error));
                    cmd.respond(response);
                }
            }
        }

        pub async fn handle_clear_indicator_cache(&mut self, cmd: ClearIndicatorCacheCommand) {
            let result = self.clear_indicator_cache(&cmd.key).await;
            match result {
                Ok(()) => {
                    let payload = ClearIndicatorCacheRespPayload::new(cmd.key.clone());
                    let response = ClearIndicatorCacheResponse::success(Some(payload));
                    cmd.respond(response);
                }
                Err(error) => {
                    let response = ClearIndicatorCacheResponse::error(Arc::new(error));
                    cmd.respond(response);
                }
            }
        }
    }
}
