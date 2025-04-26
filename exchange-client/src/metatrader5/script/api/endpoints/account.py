from fastapi import APIRouter, Query, HTTPException, Body
from typing import Optional
from pydantic import BaseModel
from typing import Dict
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
    router = APIRouter(prefix="/account", tags=["account"])

    @router.get("/get_account_info")
    async def get_account_info():
        account_info = await terminal.get_account_info()
        if account_info[0]:
            return standardize_response(
                success=True,
                message="获取账户信息成功",
                data=account_info[1]
            )
        else:
            return standardize_response(
                success=False,
                message=account_info[1],
                error_code=2
            )
    return router

