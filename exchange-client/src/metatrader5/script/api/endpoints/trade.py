from fastapi import APIRouter
from pydantic import BaseModel
from typing import Optional
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
    router = APIRouter(prefix="/trade", tags=["trade"])

    class CreateOrderRequest(BaseModel):
        order_type: str
        order_side: str
        symbol: str
        volume: float
        price: float
        tp: Optional[float] = None
        sl: Optional[float] = None

    @router.post("/create_order")
    async def create_order(request: CreateOrderRequest):
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
            
    return router