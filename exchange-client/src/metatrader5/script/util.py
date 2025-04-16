import MetaTrader5 as mt5

def get_time_frame(time_frame: str) -> int:
    time_frame_dict = {
        "M1": mt5.TIMEFRAME_M1,
        "M5": mt5.TIMEFRAME_M5,
        "M15": mt5.TIMEFRAME_M15,
        "M30": mt5.TIMEFRAME_M30,
        "H1": mt5.TIMEFRAME_H1,
        "H2": mt5.TIMEFRAME_H2,
        "H4": mt5.TIMEFRAME_H4,
        "H6": mt5.TIMEFRAME_H6,
        "H8": mt5.TIMEFRAME_H8,
        "H12": mt5.TIMEFRAME_H12,
        "D1": mt5.TIMEFRAME_D1,
        "W1": mt5.TIMEFRAME_W1,
        "MN1": mt5.TIMEFRAME_MN1
    }
    return time_frame_dict.get(time_frame)

def get_order_type(order_type: int) -> str:
    order_type_dict = {
        mt5.ORDER_TYPE_BUY: "order_type_buy",
        mt5.ORDER_TYPE_SELL: "order_type_sell",
        mt5.ORDER_TYPE_BUY_LIMIT: "order_type_buy_limit",
        mt5.ORDER_TYPE_SELL_LIMIT: "order_type_sell_limit",
        mt5.ORDER_TYPE_BUY_STOP: "order_type_buy_stop",
        mt5.ORDER_TYPE_SELL_STOP: "order_type_sell_stop",
        mt5.ORDER_TYPE_BUY_STOP_LIMIT: "order_type_buy_stop_limit",
        mt5.ORDER_TYPE_SELL_STOP_LIMIT: "order_type_sell_stop_limit",
        mt5.ORDER_TYPE_CLOSE_BY: "order_type_close_by"
    }
    return order_type_dict.get(order_type)

def get_order_type_time(order_type_time: int) -> str:
    order_type_time_dict = {
        mt5.ORDER_TIME_GTC: "order_type_time_gtc",
        mt5.ORDER_TIME_DAY: "order_type_time_day",
        mt5.ORDER_TIME_SPECIFIED: "order_type_time_specified",
        mt5.ORDER_TIME_SPECIFIED_DAY: "order_type_time_specified_day"
    }
    return order_type_time_dict.get(order_type_time)


def get_order_type_filling(order_type_filling: int) -> str: 
    order_type_filling_dict = {
        mt5.ORDER_FILLING_FOK: "order_type_filling_fok",
        mt5.ORDER_FILLING_IOC: "order_type_filling_ioc",
        mt5.ORDER_FILLING_RETURN: "order_type_filling_return",
        mt5.ORDER_FILLING_BOC: "order_type_filling_boc"
    }
    return order_type_filling_dict.get(order_type_filling)


def get_order_state(state: int) -> str:
    order_state_dict = {
        mt5.ORDER_STATE_STARTED: "started",
        mt5.ORDER_STATE_PLACED: "placed",
        mt5.ORDER_STATE_CANCELED : "canceled",
        mt5.ORDER_STATE_PARTIAL: "partial",
        mt5.ORDER_STATE_FILLED: "filled",
        mt5.ORDER_STATE_REJECTED: "rejected",
        mt5.ORDER_STATE_EXPIRED: "expired",
        mt5.ORDER_STATE_REQUEST_ADD: "request_add",
        mt5.ORDER_STATE_REQUEST_MODIFY: "request_modify",
        mt5.ORDER_STATE_REQUEST_CANCEL: "request_cancel",
    }
    return order_state_dict.get(state)


def get_order_reason(reason: int) -> str:
    order_reason_dict = {
        mt5.ORDER_REASON_CLIENT: "client",
        mt5.ORDER_REASON_MOBILE: "mobile",
        mt5.ORDER_REASON_WEB: "web",
        mt5.ORDER_REASON_EXPERT: "expert",
        mt5.ORDER_REASON_SL: "sl",
        mt5.ORDER_REASON_TP: "tp",
        mt5.ORDER_REASON_SO: "so"
    }
    return order_reason_dict.get(reason)




def get_position_type(position_type: int) -> str:
    position_type_dict = {
        mt5.POSITION_TYPE_BUY: "long",
        mt5.POSITION_TYPE_SELL: "short"
    }
    return position_type_dict.get(position_type)







def get_position_reason(position_reason: int) -> int:
    position_reason_dict = {
        mt5.POSITION_REASON_CLIENT: "client",
        mt5.POSITION_REASON_MOBILE: "mobile",
        mt5.POSITION_REASON_WEB: "web",
        mt5.POSITION_REASON_EXPERT: "expert"
    }
    return position_reason_dict.get(position_reason)


def get_deal_type(deal_type: int) -> str:
    deal_type_dict = {
        mt5.DEAL_TYPE_BUY: "buy",
        mt5.DEAL_TYPE_SELL: "sell",
        mt5.DEAL_TYPE_BALANCE: "balance",
        mt5.DEAL_TYPE_CREDIT: "credit",
        mt5.DEAL_TYPE_CHARGE: "charge",
        mt5.DEAL_TYPE_CORRECTION: "correction",
        mt5.DEAL_TYPE_BONUS: "bonus",
        mt5.DEAL_TYPE_COMMISSION: "commission",
        mt5.DEAL_TYPE_COMMISSION_DAILY: "commission_daily",
        mt5.DEAL_TYPE_COMMISSION_MONTHLY: "commission_monthly",
        mt5.DEAL_TYPE_COMMISSION_AGENT_DAILY: "commission_agent_daily",
        mt5.DEAL_TYPE_COMMISSION_AGENT_MONTHLY: "commission_agent_monthly",
        mt5.DEAL_TYPE_INTEREST: "interest",
        mt5.DEAL_TYPE_BUY_CANCELED: "buy_canceled",
        mt5.DEAL_TYPE_SELL_CANCELED: "sell_canceled",
        mt5.DEAL_DIVIDEND: "dividend",
        mt5.DEAL_DIVIDEND_FRANKED: "dividend_franked",
        mt5.DEAL_TAX: "tax",
    }
    return deal_type_dict.get(deal_type)


def get_deal_entry(deal_entry: int) -> str:
    deal_entry_dict = {
        mt5.DEAL_ENTRY_IN: "in",
        mt5.DEAL_ENTRY_OUT: "out",
        mt5.DEAL_ENTRY_INOUT: "inout",
        mt5.DEAL_ENTRY_OUT_BY: "out_by"
    }
    return deal_entry_dict.get(deal_entry)


def get_deal_reason(deal_reason: int) -> str:
    deal_reason_dict = {
        mt5.DEAL_REASON_CLIENT: "client",
        mt5.DEAL_REASON_MOBILE: "mobile",
        mt5.DEAL_REASON_WEB: "web",
        mt5.DEAL_REASON_EXPERT: "expert",
        mt5.DEAL_REASON_SL: "sl",
        mt5.DEAL_REASON_TP: "tp",
        mt5.DEAL_REASON_SO: "so",
        mt5.DEAL_REASON_ROLLOVER: "rollover",
        mt5.DEAL_REASON_VMARGIN: "vmargin",
        mt5.DEAL_REASON_SPLIT: "split"
    }
    return deal_reason_dict.get(deal_reason)


def get_trade_mode(trade_mode: int) -> str:
    trade_mode_dict = {
        mt5.ACCOUNT_TRADE_MODE_DEMO: "demo",
        mt5.ACCOUNT_TRADE_MODE_CONTEST: "contest",
        mt5.ACCOUNT_TRADE_MODE_REAL: "real"
    }
    return trade_mode_dict.get(trade_mode)


def get_margin_stopout_mode(margin_stopout_mode: int) -> str:
    margin_stopout_mode_dict = {
        mt5.ACCOUNT_STOPOUT_MODE_PERCENT: "percent",
        mt5.ACCOUNT_STOPOUT_MODE_MONEY: "money"
    }
    return margin_stopout_mode_dict.get(margin_stopout_mode)

def get_margin_mode(margin_mode: int) -> str:
    margin_mode_dict = {
        mt5.ACCOUNT_MARGIN_MODE_RETAIL_NETTING: "retail_netting",
        mt5.ACCOUNT_MARGIN_MODE_EXCHANGE: "exchange",
        mt5.ACCOUNT_MARGIN_MODE_RETAIL_HEDGING: "retail_hedging"
    }
    return margin_mode_dict.get(margin_mode)
