from fastapi import APIRouter
from pydantic import BaseModel
from client import mt5_client
from typing import Optional

trade_router = APIRouter(prefix="/trade", tags=["trade"])

class CreateOrderRequest(BaseModel):
    order_type: str
    order_side: str
    symbol: str
    volume: float
    price: float
    tp: Optional[float] = None
    sl: Optional[float] = None

@trade_router.post("/create_order")
async def create_order(request: CreateOrderRequest):
    order_info = await mt5_client.create_order(request.order_type, request.order_side, request.symbol, request.volume, request.price, request.tp, request.sl)
    if order_info.get("retcode") == 10009:
        return {
            "code": 0,
            "message": "订单创建成功",
            "data": order_info
        }
    else:
        return {
            "code": 1,
            "message": "订单创建失败",
            "data": order_info
        }
    
@trade_router.get("/get_orders_by_id")
async def get_orders_by_id(order_id: int):
    order_info = await mt5_client.get_orders_by_id(order_id)
    return order_info

@trade_router.get("/get_orders_by_symbol")
async def get_orders_by_symbol(symbol: str):
    order_info = await mt5_client.get_orders_by_symbol(symbol)
    return order_info

@trade_router.get("/get_position_by_symbol")
async def get_position_by_symbol(symbol: str):
    position_info = await mt5_client.get_position_by_symbol(symbol)
    return position_info



