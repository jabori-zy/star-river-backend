from util import get_time_frame, get_position_type, get_position_reason, get_order_type
from util import *


def parse_order(order):
    order_info = {
        "order_id": order.ticket, # 订单id
        "position_id": order.position_id, # 仓位id
        "time_setup": order.time_setup, # 订单创建时间
        "time_setup_msc": order.time_setup_msc, # 订单创建时间毫秒
        "time_done": order.time_done, # 成交时间
        "time_done_msc": order.time_done_msc, # 成交时间毫秒
        "time_expiration": order.time_expiration, # 到期时间
        "order_type": get_order_type(order.type),
        "type_time": get_order_type_time(order.type_time),
        "type_filling": get_order_type_filling(order.type_filling),
        "status": get_order_state(order.state),
        "magic": order.magic,
        "reason": get_order_reason(order.reason),
        "volume_initial": order.volume_initial,
        "volume_current": order.volume_current,
        "open_price": order.price_open,
        "sl": order.sl,
        "tp": order.tp,
        "symbol": order.symbol,
        "comment": order.comment
    }
    return order_info


def parse_position(position):
    pos_info = {
        "position_id": position.ticket,
        "time": position.time,
        "time_msc": position.time_msc,
        "time_update": position.time_update,
        "time_update_msc": position.time_update_msc,
        "position_type": get_position_type(position.type),
        "magic": position.magic,
        "identifier": position.identifier,
        "reason": get_position_reason(position.reason),
        "volume": position.volume,
        "open_price": position.price_open,
        "sl": position.sl,
        "tp": position.tp,
        "current_price": position.price_current,
        "swap": position.swap,
        "profit": position.profit,
        "symbol": position.symbol,
        "comment": position.comment,
        "external_id": position.external_id
        }
    return pos_info


def parse_deal(deal):
    deal_info = {
        "deal_id": deal.ticket, # 成交明细id
        "order_id": deal.order,
        "position_id": deal.position_id,
        "time": deal.time,
        "time_msc": deal.time_msc,
        "deal_type": get_deal_type(deal.type),
        "entry": get_deal_entry(deal.entry),
        "magic": deal.magic,
        "deal_reason": get_deal_reason(deal.reason),
        "volume": deal.volume,
        "price": deal.price,
        "commission": deal.commission,
        "swap": deal.swap,
        "profit": deal.profit,
        "fee": deal.fee,
        "symbol": deal.symbol,
        "comment": deal.comment,
        "external_id": deal.external_id
        }
    return deal_info


def parse_account_info(account_info):
    account_info_dict = {
        "account_id": account_info.login,
        "trade_mode": get_trade_mode(account_info.trade_mode),
        "leverage": account_info.leverage,
        "limit_orders": account_info.limit_orders,
        "margin_stopout_mode": get_margin_stopout_mode(account_info.margin_so_mode), # 设置最小允许保证金的模式
        "trade_allowed": account_info.trade_allowed,
        "trade_expert": account_info.trade_expert,
        "margin_mode": get_margin_mode(account_info.margin_mode),
        "currency_digits": account_info.currency_digits,
        "fifo_close": account_info.fifo_close, # 表明仓位只能按先进先出规则关闭的指示。如果属性值设置为 “true”，则每个符号头寸将按照开仓时的相同顺序平仓，从最老的头寸开始。如果试图以不同的顺序平仓，交易者将收到相应的错误信息。
        "balance": account_info.balance,
        "credit": account_info.credit,
        "profit": account_info.profit,
        "equity": account_info.equity,
        "margin": account_info.margin,
        "margin_free": account_info.margin_free,
        "margin_level": account_info.margin_level,
        "margin_so_call": account_info.margin_so_call, # 追加保证金水平。根据设置的 ACCOUNT_MARGIN_SO_MODE，以百分比或存款货币表示
        "margin_so_so": account_info.margin_so_so, # 保证金止损水平。取决于设置的 ACCOUNT_MARGIN_SO_MODE，以百分比或存款货币表示
        "margin_initial": account_info.margin_initial,
        "margin_maintenance": account_info.margin_maintenance,
        "assets": account_info.assets,
        "liabilities": account_info.liabilities, # 账户的流动负债
        "commission_blocked": account_info.commission_blocked,
        "name": account_info.name,
        "server": account_info.server,
        "currency": account_info.currency,
        "company": account_info.company
    }
    return account_info_dict


def parse_terminal_info(terminal_info):
    terminal_info_dict = {
        "community_account": terminal_info.community_account,
        "community_connection": terminal_info.community_connection,
        "connected": terminal_info.connected,
        "dlls_allowed": terminal_info.dlls_allowed,
        "trade_allowed": terminal_info.trade_allowed,
        "tradeapi_disabled": terminal_info.tradeapi_disabled,
        "email_enabled": terminal_info.email_enabled,
        "ftp_enabled": terminal_info.ftp_enabled,
        "notifications_enabled": terminal_info.notifications_enabled,
        "mqid": terminal_info.mqid,
        "build": terminal_info.build,
        "maxbars": terminal_info.maxbars,
        "codepage": terminal_info.codepage,
        "ping_last": terminal_info.ping_last,
        "community_balance": terminal_info.community_balance,
        "retransmission": terminal_info.retransmission,
        "company": terminal_info.company,
        "name": terminal_info.name,
        "language": terminal_info.language,
        "path": terminal_info.path,
        "data_path": terminal_info.data_path,
        "commondata_path": terminal_info.commondata_path,
    }
    return terminal_info_dict


def parse_symbol_info(symbol_info):
    symbol_info_dict = {
        "custom": symbol_info.custom,
        "visible": symbol_info.visible,
        "session_deals": symbol_info.session_deals,
        "session_buy_orders": symbol_info.session_buy_orders,
        "session_sell_orders": symbol_info.session_sell_orders,
        "volume": symbol_info.volume,
        "volumehigh": symbol_info.volumehigh,
        "volumelow": symbol_info.volumelow,
        "digits": symbol_info.digits,
        "spread": symbol_info.spread,
        "spread_float": symbol_info.spread_float,
        "ticks_bookdepth": symbol_info.ticks_bookdepth,
        "trade_calc_mode": symbol_info.trade_calc_mode, # 交易计算模式
        "trade_mode": symbol_info.trade_mode, # 交易模式
        "name": symbol_info.name,
        "path": symbol_info.path,
        "point": symbol_info.point
    }
    return symbol_info_dict

def parse_symbol_info_tick(symbol_info_tick):
    symbol_info_tick_dict = {
        "time": symbol_info_tick.time,
        "time_msc": symbol_info_tick.time_msc,
        "bid": symbol_info_tick.bid,
        "ask": symbol_info_tick.ask,
        "last": symbol_info_tick.last,
        "volume": symbol_info_tick.volume,
        "flags": symbol_info_tick.flags,
        "volume_real": symbol_info_tick.volume_real,
    }
    return symbol_info_tick_dict