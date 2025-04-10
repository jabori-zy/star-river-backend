from fastapi import APIRouter
from pydantic import BaseModel
from client import mt5_client
from typing import Optional
from fastapi import Query, HTTPException
order_router = APIRouter(prefix="/order", tags=["order"])



@order_router.get("/get_order")
async def get_order(
    order_id: Optional[int] = Query(default=None),
    symbol: Optional[str] = Query(default=None),
    position_id: Optional[int] = Query(default=None)
    ):

    provided_params = sum(param is not None for param in [order_id, symbol, position_id])
    # 验证参数：必须且只能提供一个参数
    if provided_params != 1:
        raise HTTPException(
            status_code=400,
            detail="必须且只能提供 order_id、position_id 或 symbol 其中一个参数"
        )
    
    try:
        if order_id is not None:
            order = await mt5_client.get_order_by_id(order_id)
            # 如果列表为空，则从历史订单中获取
            if not order:
                order = await mt5_client.get_history_order_by_id(order_id)
            
            return {
                "code": 0,
                "message": "success",
                "data": order
                }
        elif symbol is not None:
            order = await mt5_client.get_history_order_by_symbol(symbol)
            return {
                "code": 0,
                "message": "success",
                "data": order
                }
        else:
            order = await mt5_client.get_history_order_by_position_id(symbol)
            return {
                "code": 0,
                "message": "success",
                "data": order
                }
        
        
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=str(e)
        )


    
    
        
