import asyncio
from parse import parse_symbol_info, parse_symbol_info_tick
from datetime import datetime
from util import get_time_frame
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .terminal import Mt5Terminal

class MarketManager:
    def __init__(self, terminal: "Mt5Terminal"):
        self.terminal = terminal
    
    # 获取交易品种列表
    async def get_symbols(self) -> list: 
        symbols = self.terminal.terminal.symbols_get()
        if symbols is None:
            return False, self.terminal.terminal.last_error()
        return True, [symbol.name for symbol in symbols]
    

    # 获取交易品种信息
    async def get_symbol_info(self, symbol: str) -> dict:

        symbol_info = self.terminal.terminal.symbol_info(symbol)
        if not symbol_info:
            return False, self.terminal.terminal.last_error()
        return True, parse_symbol_info(symbol_info)
    
    # 获取交易品种最新报价
    async def get_symbol_info_tick(self, symbol: str) -> dict:
        
        symbol_info_tick = self.terminal.terminal.symbol_info_tick(symbol)
        if not symbol_info_tick:
            return False, self.terminal.terminal.last_error()
        return True, parse_symbol_info_tick(symbol_info_tick)
    

    # 获取历史k线
    async def get_kline_series_by_time_range(self, symbol: str, interval: str, start_time: str, end_time: str) -> list:
        time_frame = get_time_frame(interval)
        start = datetime.strptime(start_time,'%Y-%m-%d').replace(tzinfo=self.terminal.timezone)
        end = datetime.strptime(end_time,'%Y-%m-%d').replace(tzinfo=self.terminal.timezone)
        
        history_kline = self.terminal.terminal.copy_rates_range(symbol, time_frame, start, end)
        if history_kline is None:
            return False, self.terminal.terminal.last_error()
        return True, [row[:-2] for row in history_kline.tolist()]
    
    
    # 获取k线系列
    async def get_kline_series(self, symbol: str, interval: str, limit: int) -> list:
        time_frame = get_time_frame(interval)

        kline_series = self.terminal.terminal.copy_rates_from_pos(symbol, time_frame, 0, limit)
        if kline_series is None:
            return False, self.terminal.terminal.last_error()
        
        if len(kline_series) == 0:
            return True, []
        
        result = []
        for kline in kline_series:
            kline_data = kline.tolist()[:-2]  # 去掉最后两个字段
            result.append(kline_data)
    
        return True, result
    
    async def get_latest_kline(self, symbol: str, interval: str) -> tuple[bool, dict]:
        time_frame = get_time_frame(interval)
            
        latest_kline = self.terminal.terminal.copy_rates_from_pos(symbol, time_frame, 0, 1)
        if latest_kline is None:
            return False, self.terminal.terminal.last_error()
        
        if len(latest_kline) == 0:
            return True, []
            
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

        return True, kline_dict
