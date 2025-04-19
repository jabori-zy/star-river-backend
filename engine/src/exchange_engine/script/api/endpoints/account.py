from fastapi import APIRouter, Query, HTTPException, Body
from typing import Optional
from client_manager import mt5_client_manager
from pydantic import BaseModel
from typing import Dict
from .__init__ import standardize_response

router = APIRouter(prefix="/account", tags=["account"])




@router.get("/get_account_info")
async def get_account_info(terminal_id: int = Query(default=None)):
    terminal = mt5_client_manager.get_terminal(terminal_id)
    if terminal is None:
        return standardize_response(
            success=False,
            message="终端不存在",
            error_code=1
        )
    account_info = await terminal.get_account_info()
    if account_info[0]:
        account_info[1]["terminal_id"] = terminal_id
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

@router.get("/get_terminal_info")
async def get_terminal_info(terminal_id: int = Query(
        default=None,
        description="终端ID",
        examples=1
    )):
    terminal = mt5_client_manager.get_terminal(terminal_id)
    if terminal is None:
        return standardize_response(
            success=False,
            message="终端不存在",
            error_code=1
        )
    terminal_info = await terminal.get_terminal_info()
    return standardize_response(
        success=True,
        message="获取终端信息成功",
        data=terminal_info
    )

