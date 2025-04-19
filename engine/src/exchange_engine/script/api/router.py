from fastapi import APIRouter
from .endpoints import basic, order, position, trade, account, market

# 创建主路由
router = APIRouter()

# 包含各个模块的路由
router.include_router(basic.router)
router.include_router(order.router)
router.include_router(position.router)
router.include_router(trade.router)
router.include_router(account.router)
router.include_router(market.router)
