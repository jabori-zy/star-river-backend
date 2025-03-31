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


def get_position_type(position_type: int) -> int:
    position_type_dict = {
        mt5.POSITION_TYPE_BUY: "buy",
        mt5.POSITION_TYPE_SELL: "sell"
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

