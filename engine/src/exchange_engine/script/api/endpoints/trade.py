from fastapi import APIRouter
from pydantic import BaseModel
from typing import Optional
from client_manager import mt5_client_manager
from .__init__ import standardize_response

router = APIRouter(prefix="/trade", tags=["trade"])

class CreateOrderRequest(BaseModel):
    terminal_id: int
    order_type: str
    order_side: str
    symbol: str
    volume: float
    price: float
    tp: Optional[float] = None
    sl: Optional[float] = None

@router.post("/create_order")
async def create_order(request: CreateOrderRequest):
    terminal_id = request.terminal_id
    terminal = mt5_client_manager.get_terminal(terminal_id)
    if terminal is None:
        return standardize_response(
            success=False,
            message="终端不存在",
            data=None
        )
    order_info = await terminal.order.create_order(request.order_type, request.order_side, request.symbol, request.volume, request.price, request.tp, request.sl)
    if order_info[0]:
        if order_info[1].get("retcode") == 10009:
            return standardize_response(
                success=True,
                message="订单创建成功",
                data=order_info[1]
            )
        else:
            return standardize_response(
                success=False,
                message="订单创建失败",
                data={
                    "error": order_info[1]
                }
            )
    else:
        return standardize_response(
            success=False,
            message="订单创建失败",
            data={
                "error": order_info[1]
            }
        )