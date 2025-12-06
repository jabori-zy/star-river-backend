// use crate::api::account_api::ExchangeType;
// use crate::api::account_api::{AccountConfigType, AddAccountConfigParams, Mt5AccountConfigParams};
use star_river_core::account::AccountConfig;
use utoipa::OpenApi;

use crate::api::{response::ApiResponse, system_api::SystemConfigUpdateParams};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Strategy related paths
        crate::api::strategy_api::strategy_management::create_strategy,
        crate::api::strategy_api::strategy_management::get_strategy_list,
        crate::api::strategy_api::strategy_management::get_strategy_by_id,
        crate::api::strategy_api::strategy_management::update_strategy,
        crate::api::strategy_api::strategy_management::delete_strategy,


        // Backtest strategy
        crate::api::strategy_api::backtest::init_strategy,
        crate::api::strategy_api::backtest::stop_strategy,
        crate::api::strategy_api::backtest::play,
        crate::api::strategy_api::backtest::pause,
        crate::api::strategy_api::backtest::reset,
        crate::api::strategy_api::backtest::play_one,
        crate::api::strategy_api::backtest::update_backtest_chart_config,
        crate::api::strategy_api::backtest::get_backtest_chart_config,
        crate::api::strategy_api::backtest::get_cycle_id,
        crate::api::strategy_api::backtest::get_strategy_datetime,
        crate::api::strategy_api::backtest::get_virtual_orders,
        crate::api::strategy_api::backtest::get_current_positions,
        crate::api::strategy_api::backtest::get_history_positions,
        crate::api::strategy_api::backtest::get_stats_history,
        crate::api::strategy_api::backtest::get_virtual_transactions,
        crate::api::strategy_api::backtest::get_strategy_run_state,
        crate::api::strategy_api::backtest::get_running_log,
        crate::api::strategy_api::backtest::get_strategy_data,
        crate::api::strategy_api::backtest::get_strategy_variable,
        crate::api::strategy_api::backtest::get_strategy_performance_report,
        crate::api::strategy_api::backtest::get_strategy_keys,
        // Account related paths
        // crate::api::account_api::get_account_configs,
        // crate::api::account_api::add_account_config,
        // crate::api::account_api::delete_account_config,
        // crate::api::account_api::update_account_config,
        // crate::api::account_api::update_account_is_available,
        // crate::api::account_api::start_mt5_terminal,
        // crate::sse::account_sse::account_sse_handler,
        // Market related paths
        crate::api::market_api::get_symbol_list,
        crate::api::market_api::get_support_kline_intervals,
        crate::api::market_api::get_symbol,

        // Exchange related paths
        crate::api::exchange_api::get_exchange_status,
        crate::api::exchange_api::connect_exchange,

        //sse
        crate::sse::backtest_strategy_state_log_sse::backtest_strategy_state_log_sse_handler,
        crate::sse::backtest_strategy_event_sse::backtest_strategy_event_sse_handler,
        crate::sse::backtest_strategy_running_log_sse::backtest_strategy_running_log_sse_handler,

        // // Other paths
        // crate::api::strategy_api::enable_strategy_data_push,
        // crate::api::strategy_api::disable_strategy_data_push,
        // crate::api::strategy_api::pause,
        // crate::api::strategy_api::play_one,
        // crate::api::strategy_api::stop,

        // System configuration related paths
        crate::api::system_api::update_system_config,
        crate::api::system_api::get_system_config,
        crate::api::system_api::get_timezones,
    ),
    components(
        schemas(
            ApiResponse<AccountConfig>,
            ApiResponse<String>,
            // Account related types
            // Mt5AccountConfigParams,
            // AccountConfigType,
            // AddAccountConfigParams,
            // Account configuration
            AccountConfig,
            // ExchangeType,

            // System configuration related types
            SystemConfigUpdateParams,
        )
    ),
    tags(
        (name = "Strategy Management", description = "Strategy creation, update, deletion, query and other management functions"),
        (name = "Strategy Control", description = "Strategy run, stop, play, pause and other control functions"),
        (name = "Account Management", description = "Account configuration management and MT5 account operations"),
        (name = "Cache Management", description = "Cache data query and memory management"),
        (name = "Real-time Data", description = "SSE real-time data push"),
        (name = "Market", description = "Market data"),
    ),
    info(
        title = "Star River API",
        description = "Star River Quantitative Trading System API",
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
        (url = "http://localhost:3100", description = "Local development server"),
        (url = "https://api.star-river.com", description = "Production server")
    )
)]
pub struct ApiDoc;
