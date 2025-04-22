from fastapi import APIRouter, Query, Body
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
    router = APIRouter()
    
    # 整个fastapi的ping,而不是客户端的ping
    @router.get("/ping")
    async def ping():
        return standardize_response(
            success=True,
            message="pong",
            data=None
        )


    class InitializeTerminalParams(BaseModel):
        terminal_path: str

    @router.post("/initialize_terminal")
    # 初始化MT5客户端
    async def initialize_terminal(request: InitializeTerminalParams) -> Dict:
        terminal_path = request.terminal_path
        # 设置终端路径
        terminal.set_terminal_path(terminal_path)
        # 初始化终端
        init_result = await terminal.initialize_terminal(terminal_path=terminal_path)
        # 如果初始化成功，则返回客户端信息
        if init_result[0]:
            return standardize_response(
                success=True,
                message=f"MT5客户端初始化成功",
                data={
                    "terminal_path": terminal_path,
                    "terminal_info": init_result[1]
                }
            )
        else:
            return standardize_response(
                success=False,
                message=f"MT5客户端初始化失败",
                error_code=1,
                data={
                    "error": init_result[1]
                }
            )


    @router.get("/ping_terminal")
    async def ping_terminal():
        ping_result = terminal.ping()
        if ping_result[0]:
            return standardize_response(
            success=True,
                message="pong",
                data=None
            )
        else: 
            return standardize_response(
                success=False,
                message=ping_result[1],
                error_code=1
            )

    # 定义请求体模型
    class LoginRequest(BaseModel):
        login: int
        password: str
        server: str


    @router.post("/login")
    async def login(
        request: LoginRequest = Body(...)
    ) -> Dict:
        login_result = await terminal.login_mt5(login=request.login, password=request.password, server=request.server)
        if login_result[0]:
            return {
                "code": 0,
                "message": "登录成功",
                "data": login_result[1]
            }
        else:
            return {
                "code": 1,
                "message": f"登录失败：{login_result[1]}",
                "data": None
            }
            
    return router








