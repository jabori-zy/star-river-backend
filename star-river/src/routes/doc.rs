use utoipa::OpenApi;
use crate::api::response::ApiResponse;
use types::account::AccountConfig;
use crate::api::account_api::{
    Mt5AccountConfigParams, AccountConfigType, AddAccountConfigParams,
};
use crate::api::cache_api::CacheKeyType;
use crate::api::account_api::ExchangeType;
use types::system::system_config::SystemConfigUpdateParams;

#[derive(OpenApi)]
#[openapi(
    paths(
        // 策略相关路径
        crate::api::strategy_api::create_strategy,
        crate::api::strategy_api::get_strategy_list,
        crate::api::strategy_api::get_strategy_by_id,
        crate::api::strategy_api::update_strategy,
        crate::api::strategy_api::delete_strategy,
        crate::api::strategy_api::init_strategy,
        crate::api::strategy_api::get_strategy_cache_keys,
        crate::api::strategy_api::stop_strategy,

        // 实盘策略
        // crate::api::strategy_api::run_strategy,
        // crate::api::strategy_api::stop_strategy,

        // 回测策略
        crate::api::strategy_api::backtest_strategy::play,
        crate::api::strategy_api::backtest_strategy::pause,
        crate::api::strategy_api::backtest_strategy::reset,
        crate::api::strategy_api::backtest_strategy::play_one,
        crate::api::strategy_api::backtest_strategy::update_backtest_chart_config,
        crate::api::strategy_api::backtest_strategy::get_backtest_chart_config,
        crate::api::strategy_api::backtest_strategy::get_play_index,
        crate::api::strategy_api::backtest_strategy::get_virtual_orders,
        crate::api::strategy_api::backtest_strategy::get_current_positions,
        
        // 账户相关路径
        crate::api::account_api::get_account_configs,
        crate::api::account_api::add_account_config,
        crate::api::account_api::delete_account_config,
        crate::api::account_api::update_account_config,
        crate::api::account_api::update_account_is_available,   
        crate::api::account_api::start_mt5_terminal,
        crate::sse::account_sse::account_sse_handler,
        
        // // 缓存相关路径
        crate::api::cache_api::get_cache_keys,
        crate::api::cache_api::get_cache_value,
        crate::api::cache_api::get_memory_size,
        
        // // 其他路径
        // crate::api::strategy_api::enable_strategy_data_push,
        // crate::api::strategy_api::disable_strategy_data_push,
        
        // crate::api::strategy_api::pause,
        // crate::api::strategy_api::play_one,
        // crate::api::strategy_api::stop,

        // 系统配置相关路径
        crate::api::system_api::update_system_config,
        crate::api::system_api::get_system_config,
    ),
    components(
        schemas(
            ApiResponse<AccountConfig>,
            ApiResponse<String>,
            
            // 账户相关类型
            Mt5AccountConfigParams,
            AccountConfigType,
            AddAccountConfigParams,
            
            // 账户配置
            AccountConfig,
            ExchangeType,

            // 缓存相关类型
            CacheKeyType,

            // 系统配置相关类型
            SystemConfigUpdateParams,
        )
    ),
    tags(
        (name = "策略管理", description = "策略创建、更新、删除、查询等管理功能"),
        (name = "策略控制", description = "策略运行、停止、播放、暂停等控制功能"),
        (name = "账户管理", description = "账户配置管理和MT5账户操作"),
        (name = "缓存管理", description = "缓存数据查询和内存管理"),
        (name = "实时数据", description = "SSE实时数据推送"),
    ),
    info(
        title = "Star River API",
        description = "Star River 量化交易系统 API",
        version = "1.0.0",
        contact(
            name = "Star River Team",
            email = "admin@star-river.com"
        ),
        license(
            name = "MIT"
        )
    ),
    servers(
        (url = "http://localhost:3100", description = "本地开发服务器"),
        (url = "https://api.star-river.com", description = "生产服务器")
    )
)]
pub struct ApiDoc;