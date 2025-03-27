from mt5_client import Mt5Client
from fastapi import FastAPI, Query
import uvicorn
import MetaTrader5 as mt5
from typing import Dict
from utils import get_time_frame
from pydantic import BaseModel
from fastapi import Body
from fastapi.middleware.cors import CORSMiddleware
from websocket import register_websocket_routes
from contextlib import asynccontextmanager
import asyncio
from websocket import data_push_task
import numpy as np

@asynccontextmanager
async def lifespan(app: FastAPI):
    # 启动时执行
    task = asyncio.create_task(data_push_task(mt5_client))
    
    yield  # 服务运行期间
    
    # 关闭时执行
    task.cancel()


app = FastAPI(lifespan=lifespan)

# 添加CORS中间件
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # 允许所有来源，生产环境应限制
    allow_credentials=False,
    allow_methods=["*"],
    allow_headers=["*"],
)

mt5_client = Mt5Client()

register_websocket_routes(app)



@app.get("/ping",
         summary="检查MT5服务器是否正常",
         description="检查MT5服务器是否正常",
         response_description="返回服务器状态",
         responses={
             200: {
                "description": "服务器正常"
             }
         }
         )
async def ping():
    return {"code": 0, "message": "pong", "data": None}



@app.get("/client_status",
         summary="检查 MT5 客户端状态",
         description="检查 MT5 客户端状态是否正常",
         response_description="返回客户端连接状态",)
async def client_status() -> Dict:
    global mt5_client
    if mt5_client.is_initialized:
        return {
            "code": 0,
            "message": "MT5 客户端已初始化",
            "data": None
        }
    else:
        return {
            "code": 1,
            "message": "MT5 客户端未初始化",
            "data": None
        }
    
@app.get("/initialize_client",
         summary="初始化 MT5 客户端",
         description="初始化 MT5 客户端",
         response_description="返回初始化状态",
         responses={
            200: {
                "description": "初始化成功",
                "content": {
                    "application/json": {
                        "exampless": {"status": 0, "message": "初始化成功"}
                    }
                }
            }
         }
            )

# 初始化客户端
async def initialize_client(terminal_path: str = Query(
        r'C:/Program Files/MetaTrader 5/terminal64.exe',  # 默认值
        description="MT5终端路径",
        examples=r'C:/Program Files/MetaTrader 5/terminal64.exe'
    )) -> Dict:
    global mt5_client
    if mt5_client.is_initialized:
        return {
            "code": 0,
            "message": "MT5 客户端已初始化",
            "data": None
        }
    if await mt5_client.initialize(terminal_path=terminal_path):
        return {
            "code": 0,
            "message": "初始化成功",
            "data": None
        }
    else:
        return {
            "code": 1,
            "message": "初始化失败",
            "data": None
        }
    

# 定义请求体模型
class LoginRequest(BaseModel):
    account_id: int
    password: str
    server: str
    terminal_path: str = r'C:/Program Files/MetaTrader 5/terminal64.exe'
    
    model_config = {
        "json_schema_extra": {
            "example": {
                "account_id": 23643,
                "password": "HhazJ520!!!!",
                "server": "EBCFinancialGroupKY-Demo",
                "terminal_path": r"C:/Program Files/MetaTrader 5/terminal64.exe"
            }
        }
    }


@app.post("/login",
    summary="登录 MT5 客户端",
    description="尝试连接并登录 MT5 客户端",
    response_description="返回登录状态和详细信息",
    responses={
        200: {
            "description": "登录成功",
            "content": {
                "application/json": {
                    "example": {
                        "status": 0,
                        "message": "登录成功",
                        "data": {
                            "account_id": 23643,
                            "server": "EBCFinancialGroupKY-Demo"
                        }
                    }
                }
            }
        },
        401: {
            "description": "登录失败",
            "content": {
                "application/json": {
                    "example": {
                        "status": 1,
                        "message": "登录失败：账号或密码错误"
                    }
                }
            }
        }
    }
)
async def login(
    request: LoginRequest = Body(...)
) -> Dict:
    global mt5_client
    if not mt5_client.is_initialized:
        return {
            "code": 2,
            "message": "MT5 客户端未初始化",
            "data": None
        }
    await mt5_client.initialize(terminal_path=request.terminal_path)
    mt5_client.set_account_id(request.account_id)
    mt5_client.set_password(request.password)
    mt5_client.set_server(request.server)
    mt5_client.set_terminal_path(request.terminal_path)

    is_connected = await mt5_client.connect()
    if is_connected:
        return {
            "code": 0,
            "message": "登录成功",
            "data": {
                "account_id": request.account_id,
                "server": request.server
            }
        }
    else:
        return {
            "code": 1,
            "message": f"登录失败：{mt5.last_error()}",
            "data": None
        }
    
@app.get("/get_symbols",
         summary="获取 MT5 支持的交易品种",
         description="获取 MT5 支持的交易品种",
         response_description="返回交易品种列表",
         responses={
            200: {
                "description": "获取成功",
                "content": {
                    "application/json": {
                        "examples": {"symbols": ["EURUSD", "GBPUSD", "USDJPY"]}
                    }
                }
            },
            401: {
                "description": "获取失败",
                "content": {
                    "application/json": {
                        "exampless": {"symbols": []}
                    }
                }
            }
         }
         )
async def get_symbols() -> Dict:
    global mt5_client
    symbols = mt5_client.get_symbols()
    if len(symbols) > 0:
        return {
            "status": 0,
            "message": "获取成功",
            "data": {"symbols": symbols}
        }
    else:
        return {
            "status": 1,
            "message": "获取失败"
        }
    
@app.get("/get_symbol_info",
         summary="获取 MT5 交易品种信息",
         description="获取 MT5 交易品种信息",
         response_description="返回交易品种信息",
         responses={
             200: { 
                "description": "获取成功",
                "content": {
                    "application/json": {
                        "examples": {"symbol_info": {"symbol": "EURUSD", "point": 0.00001, "digits": 5, "min_volume": 0.01, "max_volume": 1000000}}
                    }
                }
             }
         }
         )
async def get_symbol_info(symbol: str = Query(
        "XAUUSD",  # 默认值
        description="获取MT5交易品种信息",
        examples="XAUUSD"
    )) -> Dict:
    global mt5_client
    symbol_info = await mt5_client.get_symbol_info(symbol)
    return {
        "status": 0,
        "message": "获取成功",
        "data": {"symbol_info": symbol_info}
    }

@app.get("/get_history_kline",
         summary="获取MT5历史K线数据",
         description="获取MT5历史K线数据",
         response_description="返回历史K线数据",
         responses={
             200: {
                "description": "获取成功"
             }
         }
         )
async def get_history_kline(symbol: str
                            =Query(
                                "XAUUSD",  # 默认值
                                description="MT5交易品种",
                                examples="XAUUSD"
                            ),
                            time_frame: str = Query(
                                "M5",  # 默认值
                                description="MT5时间框架",
                                examples="M5"
                            ),
                            start_time: str = Query(
                                "2025-01-23",  # 默认值
                                description="MT5开始时间",
                                examples="2025-01-23"
                            ),
                            end_time: str = Query(
                                "2025-01-24",  # 默认值
                                description="MT5结束时间",
                                examples="2025-01-24"
                            )) -> Dict:
    global mt5_client
    time_frame = get_time_frame(time_frame)
    history_kline = await mt5_client.get_history_kline(symbol, time_frame, start_time, end_time)
    if len(history_kline) > 0:
        return {
            "status": 0,
            "message": "获取成功",
            "data": {"history_kline": history_kline}
        }
    else:
        return {
            "status": 1,
            "message": "获取失败"
        }

    
@app.get("/get_latest_kline",
         summary="获取MT5最新K线数据",
         description="获取MT5最新K线数据",
         response_description="返回最新K线数据",
         responses={
             200: {
                "description": "获取成功"
             }
         }
         )

async def get_latest_kline(symbol: str = Query(
        "XAUUSD",  # 默认值
        description="MT5交易品种",
        examples="XAUUSD"
    ),
    time_frame: str = Query(
        "M5",  # 默认值
        description="MT5时间框架",
        examples="M5"
    )) -> Dict:
    global mt5_client
    time_frame = get_time_frame(time_frame)
    latest_kline = await mt5_client.get_latest_kline(symbol, time_frame)
    if len(latest_kline) > 0:
        return {
            "status": 0,
            "message": "获取成功",
            "data": {"latest_kline": latest_kline}
        }
    else:
        return {
            "status": 1,
            "message": "获取失败"
        }





server_port = 8000
server_host = "127.0.0.1"
server_instance = None
def start_server():
    """启动 FastAPI 服务器"""
    global server_instance
    if server_instance is None:
        server_instance = uvicorn.run(app, host=server_host, port=server_port, log_level="debug")
    else:
        print("服务器已启动")





if __name__ == "__main__":
    start_server()


    
