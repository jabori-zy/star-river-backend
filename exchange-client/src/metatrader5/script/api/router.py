from fastapi import APIRouter
from .endpoints import basic, order, position, trade, account, market

# 创建路由工厂函数，为每个终端管理器创建独立的路由
def create_router(client_manager):
    """
    为指定的客户端管理器创建独立的路由集合
    
    Args:
        client_manager: 终端专用的客户端管理器实例
        
    Returns:
        APIRouter: 包含所有路由的主路由实例
    """
    # 创建主路由
    router = APIRouter()
    
    # 从各个模块创建路由，并传入对应的客户端管理器
    basic_router = basic.create_router(client_manager)
    order_router = order.create_router(client_manager)
    position_router = position.create_router(client_manager)
    trade_router = trade.create_router(client_manager)
    account_router = account.create_router(client_manager)
    market_router = market.create_router(client_manager)
    
    # 包含各个模块的路由
    router.include_router(basic_router)
    router.include_router(order_router)
    router.include_router(position_router)
    router.include_router(trade_router)
    router.include_router(account_router)
    router.include_router(market_router)
    
    return router
