from fastapi import APIRouter, Query, Body
from pydantic import BaseModel
from typing import Dict
from client_manager import mt5_client_manager
from .__init__ import standardize_response



router = APIRouter(prefix="/market", tags=["market"])

@router.get("/get_symbols")
async def get_symbols(terminal_id: int = Query()):
    terminal = mt5_client_manager.get_terminal(terminal_id)
    if terminal is None:
        return standardize_response(
            success=False,
            message="终端不存在",
            error_code=1
        )
    symbols = await terminal.market.get_symbols()
    if not symbols[0]:
        return standardize_response(
            success=False,
            message="获取交易品种失败",
            error_code=2,
            data=symbols[1]
        )
    return standardize_response(
        success=True,
        message="获取交易品种成功",
        data=symbols[1]
    )


@router.get("/get_symbol_info")
async def get_symbol_info(terminal_id: int = Query(),symbol: str = Query()) -> Dict:
    terminal = mt5_client_manager.get_terminal(terminal_id)
    if terminal is None:
        return standardize_response(
            success=False,
            message="终端不存在",
            error_code=1
        )
    symbol_info = await terminal.market.get_symbol_info(symbol)
    if not symbol_info[0]:
        return standardize_response(
            success=False,
            message="获取交易品种信息失败",
            error_code=2,
            data=symbol_info[1]
        )
    return standardize_response(
        success=True,
        message="获取交易品种信息成功",
        data=symbol_info[1]
    )


@router.get("/get_symbol_info_tick")
async def get_symbol_info_tick(terminal_id: int = Query(
        default=None,
        description="终端ID",
        examples=1
    ),
    symbol: str = Query(
        default=None,
        description="MT5交易品种",
        examples="XAUUSD"
    )) -> Dict:
    terminal = mt5_client_manager.get_terminal(terminal_id) 
    if terminal is None:
        return standardize_response(
            success=False,
            message="终端不存在",
            error_code=1
        )
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


# @router.get("/get_kline_series_by_time_range")
# async def get_kline_series_by_time_range(symbol: str
#                             =Query(
#                                 "XAUUSD",  # 默认值
#                                 description="MT5交易品种",
#                                 examples="XAUUSD"
#                             ),
#                             time_frame: str = Query(
#                                 "M5",  # 默认值
#                                 description="MT5时间框架",
#                                 examples="M5"
#                             ),
#                             start_time: str = Query(
#                                 "2025-01-23",  # 默认值
#                                 description="MT5开始时间",
#                                 examples="2025-01-23"
#                             ),
#                             end_time: str = Query(
#                                 "2025-01-24",  # 默认值
#                                 description="MT5结束时间",
#                                 examples="2025-01-24"
#                             )) -> Dict:
#     from util import get_time_frame
#     time_frame_value = get_time_frame(time_frame)
#     history_kline = await mt5_client111.kline.get_kline_series_by_time_range(symbol, time_frame_value, start_time, end_time)
#     return {
#         "code": 0,
#         "message": "success",
#         "data": history_kline
#     }


@router.get("/get_kline_series")
async def get_kline_series(terminal_id: int = Query(),symbol: str = Query(),interval: str = Query(),limit: int = Query()) -> Dict:
    terminal = mt5_client_manager.get_terminal(terminal_id)
    if terminal is None:
        return standardize_response(
            success=False,
            message="终端不存在",
            error_code=1
        )
    
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


@router.get("/get_latest_kline")
async def get_latest_kline(terminal_id: int = Query(),symbol: str = Query(),interval: str = Query()) -> Dict:
    terminal = mt5_client_manager.get_terminal(terminal_id)
    if terminal is None:
        return standardize_response(
            success=False,
            message="终端不存在",
            error_code=1
        )
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