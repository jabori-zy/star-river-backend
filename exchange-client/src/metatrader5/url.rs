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
}


#[derive(Display, Debug, Clone)]
pub(crate) enum Mt5WsUrl {
    #[strum(serialize = "ws://localhost:8000/ws")]
    BaseUrl,
}

