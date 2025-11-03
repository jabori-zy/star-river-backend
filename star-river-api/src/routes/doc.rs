// use crate::api::account_api::ExchangeType;
// use crate::api::account_api::{AccountConfigType, AddAccountConfigParams, Mt5AccountConfigParams};
use crate::api::response::ApiResponse;
use crate::api::system_api::SystemConfigUpdateParams;
use star_river_core::account::AccountConfig;
use utoipa::OpenApi;


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
        
        crate::api::strategy_api::stop_strategy,


        // 回测策略
        crate::api::strategy_api::backtest::play,
        crate::api::strategy_api::backtest::pause,
        crate::api::strategy_api::backtest::reset,
        crate::api::strategy_api::backtest::play_one,
        crate::api::strategy_api::backtest::update_backtest_chart_config,
        crate::api::strategy_api::backtest::get_backtest_chart_config,
        crate::api::strategy_api::backtest::get_play_index,
        crate::api::strategy_api::backtest::get_virtual_orders,
        crate::api::strategy_api::backtest::get_current_positions,
        crate::api::strategy_api::backtest::get_history_positions,
        crate::api::strategy_api::backtest::get_stats_history,
        crate::api::strategy_api::backtest::get_virtual_transactions,
        crate::api::strategy_api::backtest::get_strategy_status,
        crate::api::strategy_api::backtest::get_running_log,
        crate::api::strategy_api::backtest::get_strategy_data,
        crate::api::strategy_api::backtest::get_strategy_data_by_datetime,
        crate::api::strategy_api::backtest::get_strategy_variable,
        crate::api::strategy_api::backtest::get_strategy_performance_report,
        crate::api::strategy_api::backtest::get_strategy_keys,
        // 账户相关路径
        // crate::api::account_api::get_account_configs,
        // crate::api::account_api::add_account_config,
        // crate::api::account_api::delete_account_config,
        // crate::api::account_api::update_account_config,
        // crate::api::account_api::update_account_is_available,
        // crate::api::account_api::start_mt5_terminal,
        // crate::sse::account_sse::account_sse_handler,
        // 市场相关路径
        // crate::api::market_api::get_symbol_list,
        // crate::api::market_api::get_support_kline_intervals,
        // crate::api::market_api::get_symbol,

        // 交易所相关路径
        // crate::api::exchange_api::get_exchange_status,
        // crate::api::exchange_api::connect_exchange,

        //sse
        crate::sse::backtest_strategy_state_log_sse::backtest_strategy_state_log_sse_handler,
        crate::sse::backtest_strategy_event_sse::backtest_strategy_event_sse_handler,
        crate::sse::backtest_strategy_running_log_sse::backtest_strategy_running_log_sse_handler,

        // // 其他路径
        // crate::api::strategy_api::enable_strategy_data_push,
        // crate::api::strategy_api::disable_strategy_data_push,
        // crate::api::strategy_api::pause,
        // crate::api::strategy_api::play_one,
        // crate::api::strategy_api::stop,

        // 系统配置相关路径
        crate::api::system_api::update_system_config,
        crate::api::system_api::get_system_config,
        crate::api::system_api::get_timezones,
    ),
    components(
        schemas(
            ApiResponse<AccountConfig>,
            ApiResponse<String>,
            // 账户相关类型
            // Mt5AccountConfigParams,
            // AccountConfigType,
            // AddAccountConfigParams,
            // 账户配置
            AccountConfig,
            // ExchangeType,

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
        (name = "市场", description = "市场数据"),
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
