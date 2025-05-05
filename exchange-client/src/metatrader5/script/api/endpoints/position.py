from fastapi import APIRouter, Query
from pydantic import BaseModel
from typing import Dict, List, Optional
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
    router = APIRouter(prefix="/position", tags=["position"])

    @router.get("/get_positions")
    async def get_positions(symbol: Optional[str] = None):
        
        positions = await terminal.position.get_positions(symbol)
        if positions[0]:
            return standardize_response(
                success=True,
                message="获取持仓成功",
                data=positions[1]
            )
        else:
            return standardize_response(
                success=False,
                message=f"获取持仓失败: {positions[1]}"
            )

    @router.get("/get_position_number")
    async def get_position_number(symbol: Optional[str] = None):
        """获取持仓数量"""
        positions = await terminal.position.get_positions(symbol)
        if positions[0]:
            return standardize_response(
                success=True,
                message="获取持仓数量成功",
                data=len(positions[1])
            )
        else:
            return standardize_response(
                success=False,
                message=f"获取持仓数量失败: {positions[1]}"
            )
    
    @router.get("/get_position")
    async def get_position(position_id: int = Query(..., description="持仓ID")):
        """获取特定持仓"""
        
        position = await terminal.position.get_position_by_id(position_id)
        if position[0]:
            return standardize_response(
                success=True,
                message="获取持仓成功",
                data=position[1]
            )
        else:
            return standardize_response(
                success=False,
                message=f"获取持仓失败: {position[1]}"
            )
    
    @router.post("/close_position")
    async def close_position(position_id: int = Query(..., description="持仓ID")):
        """关闭持仓"""
        
        result = await terminal.position.close_position(position_id)
        if result[0]:
            return standardize_response(
                success=True,
                message="关闭持仓成功",
                data=result[1]
            )
        else:
            return standardize_response(
                success=False,
                message=f"关闭持仓失败: {result[1]}"
            )
    
    return router
