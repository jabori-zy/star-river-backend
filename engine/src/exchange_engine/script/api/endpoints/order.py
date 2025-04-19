from fastapi import APIRouter, Query, HTTPException
from typing import Optional
from client_manager import mt5_client_manager
from .__init__ import standardize_response

router = APIRouter(prefix="/order", tags=["order"])

@router.get("/get_order")
async def get_order(
    terminal_id: int = Query(),
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
        terminal = mt5_client_manager.get_terminal(terminal_id)
        if terminal is None:
            return standardize_response(
                success=False,
                message="终端不存在",
                data=None
            )
        
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
    terminal_id: int = Query(),
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
        terminal = mt5_client_manager.get_terminal(terminal_id)
        
        if terminal is None:
            return standardize_response(
                success=False,
                message="终端不存在",
                data=None
            )
        
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