import asyncio
import pytz
import MetaTrader5 as mt5
from datetime import datetime
from utils import get_time_frame

class Mt5Client:
    def __init__(self) -> None:
        self.account_id = None
        self.password = None
        self.server = None
        self.terminal_path = None
        self.is_initialized = False
        self.timezone = pytz.timezone("Etc/UTC")

    
    async def initialize(self, terminal_path):
        # 使用asyncio.to_thread将同步操作转换为异步
        if not self.is_initialized:
            def _initialize():
                if not mt5.initialize(terminal_path=terminal_path):
                    print("初始化 MetaTrader 5 失败。错误码：", mt5.last_error())
                    mt5.shutdown()
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
            
        def _get_symbol_info():
            symbol_info = mt5.symbol_info(symbol)
            if not symbol_info:
                return {}
            return {
                "symbol_name": symbol_info.name,
                "description": symbol_info.description,
                "point": symbol_info.point,
                "spread_float": symbol_info.spread_float,
            }
            
        return await asyncio.to_thread(_get_symbol_info)
    
    async def get_history_kline(self, symbol: str, time_frame: int, start_time: str, end_time: str) -> list:
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
    
    async def get_latest_kline(self, symbol: str, interval: str) -> dict:
        time_frame = get_time_frame(interval)
        if not self.is_initialized:
            return {}
            
        def _get_latest_kline():
            latest_kline = mt5.copy_rates_from_pos(symbol, time_frame, 0, 1)
            if latest_kline is None or len(latest_kline) == 0:
                return {}
                
            # 转换为字典格式，更易于JSON序列化
            latest_kline = mt5.copy_rates_from_pos(symbol, time_frame, 0, 1)
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

    


