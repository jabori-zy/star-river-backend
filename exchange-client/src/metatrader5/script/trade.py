from fastapi import APIRouter
from pydantic import BaseModel
from client import mt5_client

trade_router = APIRouter(prefix="/trade", tags=["trade"])

class CreateOrderRequest(BaseModel):
    order_type: str
    order_side: str
    symbol: str
    volume: float
    price: float
    tp: float
    sl: float

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

