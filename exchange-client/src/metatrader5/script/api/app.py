import logging
import asyncio
import sys
import os
from fastapi import FastAPI, APIRouter, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from contextlib import asynccontextmanager
from uvicorn.logging import DefaultFormatter
from .endpoints.websocket.data_push import data_push_task
from mt5_terminal.terminal import Mt5Terminal

# 将script目录添加到路径中
script_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if script_dir not in sys.path:
    sys.path.append(script_dir)

from .router import create_router
from .endpoints.websocket.websocket import register_websocket_routes

# 配置日志
handler = logging.StreamHandler()
handler.setFormatter(DefaultFormatter("%(levelprefix)s %(message)s"))
logging.basicConfig(
    level=logging.DEBUG,
    handlers=[handler]
)

# 创建终端实例
mt5_terminal = Mt5Terminal()

@asynccontextmanager
async def setup_lifespan(app: FastAPI):
    """应用的生命周期管理器"""
    task = asyncio.create_task(data_push_task(mt5_terminal))
    
    yield 
    
    # 取消数据推送任务
    try:
        task.cancel()
    except Exception as e:
            logging.error(f"取消任务出错: {e}")

def create_app() -> FastAPI:
    """创建并配置FastAPI应用"""
    app = FastAPI(
        title="MT5终端API",
        description="MT5终端REST接口服务",
        version="1.0.0",
        lifespan=setup_lifespan
    )
    
    # 添加CORS中间件
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],  # 允许所有来源，生产环境应限制
        allow_credentials=False,
        allow_methods=["*"],
        allow_headers=["*"],
    )
    
    # 创建并包含路由
    router = create_router(mt5_terminal)
    app.include_router(router)
    
    # 注册WebSocket路由
    register_websocket_routes(app)
    
    return app 