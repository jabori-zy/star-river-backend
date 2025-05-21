from fastapi import APIRouter, Query, Body
from pydantic import BaseModel
from typing import Dict, List, Optional
from .__init__ import standardize_response
from mt5_terminal.terminal import Mt5Terminal

def create_router(terminal: Mt5Terminal):
    """
    为指定的客户端管理器创建路由
    
    Args:
        mt5_client_manager: 终端专用的客户端管理器实例
        
    Returns:
        APIRouter: 路由实例
    """
    router = APIRouter(prefix="/market", tags=["market"])

    @router.get("/get_symbols")
    async def get_symbols():
        symbols_info = await terminal.market.get_symbols()
        if symbols_info[0]:
            return standardize_response(
                success=True,
                message="获取品种列表成功",
                data=symbols_info[1]
            )
        else:
            return standardize_response(
                success=False,
                message=f"获取品种列表失败: {symbols_info[1]}"
            )

    @router.get("/get_symbol_info")
    async def get_symbol_info(symbol: str = Query(..., description="交易品种")):
        symbol_info = await terminal.market.get_symbol_info(symbol)
        if symbol_info[0]:
            return standardize_response(
                success=True,
                message="获取品种信息成功",
                data=symbol_info[1]
            )
        else:
            return standardize_response(
                success=False,
                message=f"获取品种信息失败: {symbol_info[1]}"
            )

    @router.get("/get_symbol_info_tick")
    async def get_symbol_info_tick(symbol: str = Query()) -> Dict:
        symbol_info_tick = await terminal.market.get_symbol_info_tick(symbol)
        if not symbol_info_tick[0]:
            return standardize_response(
                success=False,
                message="获取交易品种信息失败",
                error_code=2,
                data=symbol_info_tick[1]
            )
        return standardize_response(
            success=True,
            message="获取交易品种信息成功",
            data=symbol_info_tick[1]
        )

    @router.get("/get_kline_series")
    async def get_kline_series(symbol: str = Query(),interval: str = Query(),limit: int = Query()) -> Dict:
        kline_series = await terminal.market.get_kline_series(symbol, interval, limit)
        if not kline_series[0]:
            return standardize_response(
                success=False,
                message="获取K线系列失败",
                error_code=2,
                data=kline_series[1]
            )
        return standardize_response(
            success=True,
            message="获取K线系列成功",
            data=kline_series[1]
        )
    
    @router.get("/get_kline_series_by_time_range")
    async def get_kline_series_by_time_range(symbol: str = Query(),interval: str = Query(),start_time: str = Query(),end_time: str = Query()) -> Dict:
        print(symbol, interval, start_time, end_time)
        kline_series = await terminal.market.get_kline_series_by_time_range(symbol, interval, start_time, end_time)
        if not kline_series[0]:
            return standardize_response(
                success=False,
                message="获取K线系列失败",
                error_code=2,
                data=kline_series[1]
            )
        return standardize_response(
            success=True,
            message="获取K线系列成功",
            data=kline_series[1]
        )
    

    @router.get("/get_latest_kline")
    async def get_latest_kline(symbol: str = Query(),interval: str = Query()) -> Dict:
        latest_kline = await terminal.market.get_latest_kline(symbol, interval)
        if not latest_kline[0]:
            return standardize_response(
                success=False,
                message="获取最新K线失败",
                error_code=2,
                data=latest_kline[1]
            )
        return standardize_response(
            success=True,
            message="获取最新K线成功",
            data=latest_kline[1]
        )

    return router