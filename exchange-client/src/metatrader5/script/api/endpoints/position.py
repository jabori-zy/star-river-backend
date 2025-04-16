from fastapi import APIRouter, Query
from typing import Optional
from client_manager import mt5_client_manager
from .__init__ import standardize_response

router = APIRouter(prefix="/position", tags=["position"])

# todo: 需要根据terminal_id获取终端
@router.get("/get_position_number")
async def get_position_number(
    terminal_id: int = Query(None, description="终端ID"), 
    symbol: str = Query(None, description="交易品种"), 
    position_side: Optional[str] = Query(None, description="持仓方向")
    ):

    terminal = mt5_client_manager.get_terminal(terminal_id)
    if terminal is None:
        return standardize_response(
            success=False,
            message="终端不存在",
            data=None
        )
    

    position_number = await terminal.position.get_position_number(symbol=symbol, position_side=position_side)
    if not position_number[0]:
        return standardize_response(
            success=False,
            message="获取持仓数量失败",
            data={
                "error": position_number[1]
            }
        )
    

    position_number_result = {
        "symbol": symbol,
        "position_side": position_side,
        "position_number": position_number[1],
    }

    return standardize_response(
        success=True,
        message="success",
        data=position_number_result
    )


@router.get("/get_position")
async def get_position(
    terminal_id: int = Query(None, description="终端ID"), 
    position_id: int = Query(None, description="持仓ID")):

    terminal = mt5_client_manager.get_terminal(terminal_id)
    if terminal is None:
        return standardize_response(
            success=False,
            message="终端不存在",
            data=None
        )
    
    position = await terminal.position.get_position_by_id(position_id)
    if not position[0]:
        return standardize_response(
            success=False,
            message="获取持仓失败",
            data=None
        )
    
    return standardize_response(
        success=True,
        message="success",
        data=position[1]
    )
