import logging
import asyncio
import sys
import os
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from contextlib import asynccontextmanager
from uvicorn.logging import DefaultFormatter
from .endpoints.websocket.data_push import data_push_task
from client_manager import mt5_client_manager

# 将script目录添加到路径中
script_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if script_dir not in sys.path:
    sys.path.append(script_dir)

from .router import router
from .endpoints.websocket.websocket import register_websocket_routes

# 配置日志
handler = logging.StreamHandler()
handler.setFormatter(DefaultFormatter("%(levelprefix)s %(message)s"))
logging.basicConfig(
    level=logging.DEBUG,
    handlers=[handler]
)

@asynccontextmanager
async def setup_lifespan(app: FastAPI):
    # 启动时执行
    task = asyncio.create_task(data_push_task(mt5_client_manager))
    
    yield  # 服务运行期间
    
    # 关闭时执行
    task.cancel()

def create_app() -> FastAPI:
    """创建并配置FastAPI应用"""
    app = FastAPI(lifespan=setup_lifespan)
    
    # 添加CORS中间件    
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],  # 允许所有来源，生产环境应限制
        allow_credentials=False,
        allow_methods=["*"],
        allow_headers=["*"],
    )
    
    # 包含所有路由
    app.include_router(router)
    
    # 注册WebSocket路由
    register_websocket_routes(app)
    
    return app 