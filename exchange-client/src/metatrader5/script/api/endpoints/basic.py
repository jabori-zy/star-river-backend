from fastapi import APIRouter, Query, Body
from pydantic import BaseModel
from typing import Dict
from client_manager import mt5_client_manager
from .__init__ import standardize_response



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
    client_id: int
    terminal_path: str

@router.post("/initialize_terminal")
# 初始化MT5客户端
async def initialize_terminal(request: InitializeTerminalParams) -> Dict:
    client_id = request.client_id
    terminal_path = request.terminal_path
    # 新建一个客户端
    client = mt5_client_manager.create_terminal(client_id)
    init_result = await client.initialize_terminal(terminal_path=terminal_path)
    # 如果初始化成功，则返回客户端信息
    if init_result[0]:
        return standardize_response(
            success=True,
            message=f"MT5客户端-{client_id}初始化成功",
            data={
                "client_id": client_id,
                "terminal_path": terminal_path,
                "terminal_info": init_result[1]
            }
        )
    else:
        return standardize_response(
            success=False,
            message=f"MT5客户端-{client_id}初始化失败",
            error_code=1,
            data={
                "error": init_result[1]
            }
        )
    
@router.get("/ping_terminal")
async def ping_terminal(terminal_id: int = Query(
        default=None,
        description="终端ID",
        examples=1
    )):
    ping_result = mt5_client_manager.ping_terminal(terminal_id)
    if ping_result[0]:
        return standardize_response(
        success=True,
            message="pong",
            data={
                "terminal_id": terminal_id
            }
        )
    else: 
        return standardize_response(
            success=False,
            message=ping_result[1],
            error_code=1
        )








