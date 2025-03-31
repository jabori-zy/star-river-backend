import asyncio
import pytz
import MetaTrader5 as mt5
from datetime import datetime
from utils import get_time_frame, get_position_type, get_position_reason

class Mt5Client:
    def __init__(self) -> None:
        self.account_id = None
        self.password = None
        self.server = None
        self.terminal_path = None
        self.is_initialized = False
        self.timezone = pytz.timezone("Etc/UTC")
        self.client = mt5

    
    async def initialize(self, terminal_path):
        # 使用asyncio.to_thread将同步操作转换为异步
        if not self.is_initialized:
            def _initialize():
                if not self.client.initialize(terminal_path=terminal_path):
                    print("初始化 MetaTrader 5 失败。错误码：", self.client.last_error())
                    self.client.shutdown()
                    return False
                return True
            
            result = await asyncio.to_thread(_initialize)
            self.is_initialized = result
            return result
        return True


    def set_account_id(self, account_id):
        self.account_id = account_id

    def set_password(self, password):
        self.password = password

    def set_server(self, server):
        self.server = server

    def set_terminal_path(self, terminal_path):
        self.terminal_path = terminal_path
    
    def get_account_id(self):
        return self.account_id

    def get_password(self):
        return self.password

    def get_server(self):
        return self.server



        
    async def connect(self) -> bool:
        if not self.is_initialized:
            return False
            
        def _connect():
            return mt5.login(self.account_id, self.password, self.server)
            
        authorized = await asyncio.to_thread(_connect)
        if authorized:
            print("Connected to MetaTrader 5")
            return True
        else:
            print("连接 MetaTrader 5 失败。错误码：", mt5.last_error())
            return False

    
    async def get_symbols(self) -> list:
        if not self.is_initialized:
            return []
            
        def _get_symbols():
            symbols = mt5.symbols_get()
            return [symbol.name for symbol in symbols]
            
        return await asyncio.to_thread(_get_symbols)
    
    async def get_symbol_info(self, symbol: str) -> dict:
        if not self.is_initialized:
            return {}

        symbol_info = mt5.symbol_info(symbol)
        if not symbol_info:
            return {}
        return {
            "symbol_name": symbol_info.name,
            "description": symbol_info.description,
            "point": symbol_info.point,
            "spread_float": symbol_info.spread_float,
        }
    
    async def get_symbol_info_tick(self, symbol: str) -> dict:
        if not self.is_initialized:
            return {}
        
        symbol_info_tick = mt5.symbol_info_tick(symbol)
        symbol_info_tick_dict = {
            "symbol": symbol,
            "bid": symbol_info_tick.bid,
            "ask": symbol_info_tick.ask,
            "last": symbol_info_tick.last,
            "time": symbol_info_tick.time,
            "time_msc": symbol_info_tick.time_msc,
            "flags": symbol_info_tick.flags,
        }
        return symbol_info_tick_dict
        

    
    # 获取历史k线
    async def get_kline_series_by_time_range(self, symbol: str, time_frame: int, start_time: str, end_time: str) -> list:
        if not self.is_initialized:
            return []
        
        def _get_history_kline():
            start = datetime.strptime(start_time,'%Y-%m-%d').replace(tzinfo=self.timezone)
            end = datetime.strptime(end_time,'%Y-%m-%d').replace(tzinfo=self.timezone)
            
            history_kline = mt5.copy_rates_range(symbol, time_frame, start, end)
            if history_kline is None:
                return []
            return [row[:-2] for row in history_kline.tolist()]
            
        return await asyncio.to_thread(_get_history_kline)
    
    # 获取k线系列
    async def get_kline_series(self, symbol: str, interval: str, limit: int) -> list:
        time_frame = get_time_frame(interval)
        if not self.is_initialized:
            return []
        
        def _get_kline_series():
            kline_series = self.client.copy_rates_from_pos(symbol, time_frame, 0, limit)
            if kline_series is None or len(kline_series) == 0:
                return []
            
            result = []
            for kline in kline_series:
                kline_data = kline.tolist()[:-2]  # 去掉最后两个字段
                result.append(kline_data)
        
            return result
        return await asyncio.to_thread(_get_kline_series)
    
    async def get_latest_kline(self, symbol: str, interval: str) -> dict:
        time_frame = get_time_frame(interval)
        if not self.is_initialized:
            return {}
            
        def _get_latest_kline():
            latest_kline = self.client.copy_rates_from_pos(symbol, time_frame, 0, 1)
            if latest_kline is None or len(latest_kline) == 0:
                return {}
                
            # 转换为字典格式，更易于JSON序列化
            latest_kline = self.client.copy_rates_from_pos(symbol, time_frame, 0, 1)
            kline = latest_kline[0].tolist()[:-2]
            kline_dict = {
                "symbol": symbol,
                "interval": interval,
                "timestamp": kline[0],
                "open": kline[1],
                "high": kline[2],
                "low": kline[3],
                "close": kline[4],
                "volume": kline[5]
            }

            return kline_dict
            
        return await asyncio.to_thread(_get_latest_kline)
    
    # 开仓
    async def create_order(self, order_type, order_side, symbol, volume, price, tp, sl):
        if not self.is_initialized:
            return False
        
        point = self.client.symbol_info(symbol).point
        # 限价单
        if order_type == "limit":
            # 挂单
            action = self.client.TRADE_ACTION_PENDING
            # 多头
            if order_side == "long":
                type = self.client.ORDER_TYPE_BUY_LIMIT
                tp_price = price + tp * point
                sl_price = price - sl * point
            # 空头
            elif order_side == "short":
                type = self.client.ORDER_TYPE_SELL_LIMIT
                tp_price = price - tp * point
                sl_price = price + sl * point

            request = {
                "action": action,
                "symbol": symbol,
                "volume": volume,
                "type": type,
                "price": price,
                "tp": tp_price,
                "sl": sl_price,
                "deviation": 20, #偏差
                "magic": 123456,
                "comment": "star river open position",
                "type_time": self.client.ORDER_TIME_GTC,
                "type_filling": self.client.ORDER_FILLING_FOK,
                }
            order_result = self.client.order_send(request)
        # 市价单
        elif order_type == "market":
            # 市价单
            action = self.client.TRADE_ACTION_DEAL
            current_price = self.client.symbol_info(symbol).ask
            # 多头
            if order_side == "long":
                type = self.client.ORDER_TYPE_BUY
                tp_price = current_price + tp * point
                sl_price = current_price - sl * point
                
            # 空头
            elif order_side == "short":
                type = self.client.ORDER_TYPE_SELL
                tp_price = current_price - tp * point
                sl_price = current_price + sl * point

            request = {
                "action": action,
                "symbol": symbol,
                "volume": volume,
                "type": type,
                "price": current_price,
                "tp": tp_price,
                "sl": sl_price,
                "deviation": 20, #偏差
                "magic": 123456,
                "comment": "star river open position",
                "type_time": self.client.ORDER_TIME_GTC,
                "type_filling": self.client.ORDER_FILLING_FOK,

            }
            order_result = self.client.order_send(request)
        
        order_info = {
                "retcode": order_result.retcode,
                "deal": order_result.deal,
                "order_id": order_result.order,
                "symbol": symbol,
                "order_type": order_type,
                "order_side": order_side,
                "volume": order_result.volume,
                "price": order_result.price,
                "bid": order_result.bid,
                "ask": order_result.ask,
                "comment": order_result.comment,
                "request_id": order_result.request_id
            }
        
        return order_info
    
    async def close_position(self, ticket: int):
        if not self.is_initialized:
            return False
        

        position = await self.get_position(ticket)
        symbol_info_tick = await self.get_symbol_info_tick(position['symbol'])

        if position['type'] == "buy":
            order_type = self.client.ORDER_TYPE_SELL
            price = symbol_info_tick['bid']
        else:
            order_type = self.client.ORDER_TYPE_BUY
            price = symbol_info_tick['ask']

        request = {
            "action": self.client.TRADE_ACTION_DEAL,
            "symbol": position['symbol'],
            "volume": position['volume'],
            "type": order_type,
            "price": price,
            "deviation": 20,
            "magic": 123456,
            "comment": "star river close position",
            "position": position['ticket'],
        }
        order_result = self.client.order_send(request)
        order_info = {
                    "retcode": order_result.retcode,
                    "deal": order_result.deal,
                    "order": order_result.order,
                    "volume": order_result.volume,
                    "price": order_result.price,
                    "bid": order_result.bid,
                    "ask": order_result.ask,
                    "comment": order_result.comment,
                    "request_id": order_result.request_id
        }
        return order_info

    

    async def get_order(self, order_id: int) -> dict:
        if not self.is_initialized:
            return {}
        
        def _get_order():
            order = self.client.orders_get(ticket=order_id)
            return order
        
        return await asyncio.to_thread(_get_order)
    
    async def get_position(self, ticket: int) -> dict:
        if not self.is_initialized:
            return {}
        
        def _get_position():
            position = self.client.positions_get(ticket=ticket)[0]
            position_info = {
                "ticket": position.ticket,
                "time": position.time,
                "time_msc": position.time_msc,
                "time_update": position.time_update,
                "time_update_msc": position.time_update_msc,
                "type": get_position_type(position.type),
                "magic": position.magic,
                "identifier": position.identifier,
                "reason": get_position_reason(position.reason),
                "volume": position.volume,
                "price_open": position.price_open,
                "sl": position.sl,
                "tp": position.tp,
                "price_current": position.price_current,
                "swap": position.swap,
                "profit": position.profit,
                "symbol": position.symbol,
                "comment": position.comment,
                "external_id": position.external_id
            }
            return position_info
        
        return await asyncio.to_thread(_get_position)
    


