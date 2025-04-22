from fastapi import APIRouter, Query, HTTPException
from typing import Optional
from .__init__ import standardize_response
from pydantic import BaseModel
from mt5_terminal.terminal import Mt5Terminal

def create_router(terminal: Mt5Terminal):
    """
    为指定的客户端管理器创建路由
    
    Args:
        mt5_client_manager: 终端专用的客户端管理器实例
        
    Returns:
        APIRouter: 路由实例
    """
    router = APIRouter(prefix="/order", tags=["order"])

    @router.get("/get_orders")
    async def get_orders():
        orders = await terminal.order.get_orders()
        if orders[0]:
            return standardize_response(
                success=True,
                message="获取订单成功",
                data=orders[1]
            )
        else:
            return standardize_response(
                success=False,
                message=f"获取订单失败: {orders[1]}"
            )

    @router.get("/get_order")
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
                order = await terminal.order.get_order_by_id(order_id)
                # 如果列表为空，则从历史订单中获取
                if not order[1]:
                    order = await terminal.order.get_history_order_by_id(order_id)

                if not order[0]:
                    return standardize_response(
                        success=False,
                        message="获取订单失败",
                        data=order[1]
                    )
                return standardize_response(
                    success=True,
                    message="获取订单成功",
                    data=order[1]
                    )

            elif symbol is not None:
                order = await terminal.order.get_history_order_by_symbol(symbol)
                if not order[0]:
                    return standardize_response(
                        success=False,
                        message="获取订单失败",
                        data=order[1]
                    )
                return standardize_response(
                    success=True,
                    message="获取订单成功",
                    data=order[1]
                    )
            else:
                order = await terminal.order.get_history_order_by_position_id(position_id)
                if not order[0]:
                    return standardize_response(
                        success=False,
                        message="获取订单失败",
                        data=order[1]
                    )
                return standardize_response(
                    success=True,
                    message="获取订单成功",
                    data=order[1]
                    )

        except Exception as e:
            raise HTTPException(
                status_code=500,
                detail=str(e)
            )


    # 获取成交明细
    @router.get("/get_deal")
    async def get_deal(
        deal_id: Optional[int] = Query(default=None),
        order_id: Optional[int] = Query(default=None),
        symbol: Optional[str] = Query(default=None),
        position_id: Optional[int] = Query(default=None)
    ):
        provided_params = sum(param is not None for param in [deal_id, order_id, symbol, position_id])
        # 验证参数：必须且只能提供一个参数
        if provided_params != 1:
            raise HTTPException(
                status_code=400,
                detail="必须且只能提供 deal_id、order_id、position_id 或 symbol 其中一个参数"
            )

        try:
            
            if deal_id is not None:
                deal = await terminal.deal.get_deals_by_deal_id(deal_id)
                if not deal[0]:
                    return standardize_response(
                        success=False,
                        message="获取成交明细失败",
                        data={"error": deal[1]}
                    )
                return standardize_response(
                    success=True,
                    message="获取成交明细成功",
                    data=deal[1]
                    )
            elif order_id is not None:
                deal = await terminal.deal.get_deals_by_order_id(order_id)
                if not deal[0]:
                    return standardize_response(
                        success=False,
                        message="获取成交明细失败",
                        data={"error": deal[1]}
                    )
                return standardize_response(
                    success=True,
                    message="获取成交明细成功",
                    data=deal[1]
                    )
            elif symbol is not None:
                deal = await terminal.deal.get_deals_by_symbol(symbol)
                return standardize_response(
                    success=True,
                    message="获取成交明细成功",
                    data=deal[1]
                    )
            else:
                deal = await terminal.deal.get_deals_by_position_id(position_id)
                return standardize_response(
                    success=True,
                    message="获取成交明细成功",
                    data=deal
                    )

        except Exception as e:
            raise HTTPException(
                status_code=500,
                detail=str(e)
            )

    return router