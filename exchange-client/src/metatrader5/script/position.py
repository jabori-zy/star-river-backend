from fastapi import APIRouter
from pydantic import BaseModel
from client import mt5_client
from typing import Optional

position_router = APIRouter(prefix="/position", tags=["position"])


@position_router.get("/get_position_number")
async def get_position_number(symbol: str, position_side: Optional[str] = None):
    position_number = await mt5_client.get_position_number(symbol=symbol, position_side=position_side)

    position_number_result = {
        "symbol": symbol,
        "position_side": position_side,
        "position_number": position_number,
    }

    return {
            "code": 0,
            "message": "success",
            "data": position_number_result
        }

    
    
        
