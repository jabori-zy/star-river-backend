use strum::Display;



#[derive(Display, Debug, Clone)]
pub(crate) enum Mt5HttpUrl {
    #[strum(serialize = "http://localhost")]
    BaseUrl,
    #[strum(serialize = "/ping")]
    Ping,
    #[strum(serialize = "/account/initialize_terminal")]
    InitializeTerminal,
    #[strum(serialize = "/account/get_terminal_info")]
    GetTerminalInfo,
    #[strum(serialize = "/account/login")]
    Login,
    #[strum(serialize = "/market/get_symbol_list")]
    GetSymbolList,
    #[strum(serialize = "/market/get_kline_series")]
    GetKlineSeries,
    #[strum(serialize = "/market/get_kline_series_by_time_range")]
    GetKlineHistory,
    #[strum(serialize = "/trade/create_order")]
    CreateOrder,
    #[strum(serialize = "/order/get_order")]
    GetOrder,
    #[strum(serialize = "/position/get_position")]
    GetPosition,
    #[strum(serialize = "/order/get_deal")]
    GetDeal,
    #[strum(serialize = "/position/get_position_number")]
    GetPositionNumber,
    #[strum(serialize = "/account/get_account_info")]
    GetAccountInfo,
}


#[derive(Display, Debug, Clone)]
pub(crate) enum Mt5WsUrl {
    #[strum(serialize = "ws://localhost")]
    BaseUrl,
}

