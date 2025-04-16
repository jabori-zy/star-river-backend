use strum::Display;



#[derive(Display, Debug, Clone)]
pub(crate) enum Mt5HttpUrl {
    #[strum(serialize = "http://localhost:8000")]
    BaseUrl,
    #[strum(serialize = "/ping")]
    Ping,
    #[strum(serialize = "/initialize_client")]
    InitializeClient,
    #[strum(serialize = "/client_status")]
    ClientStatus,
    #[strum(serialize = "/login")]
    Login,
    #[strum(serialize = "/get_kline_series")]
    GetKlineSeries,
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
    #[strum(serialize = "ws://localhost:8000/ws")]
    BaseUrl,
}

