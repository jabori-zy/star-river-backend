from fastapi import FastAPI
import logging
from client_manager import Mt5ClientManager

# 导入拆分后的各个模块
from .connection import setup_websocket_endpoint
from .commands import process_command
    
# 兼容性函数，保持与原来的代码调用方式一致
def register_websocket_routes(app: FastAPI):
    """兼容旧版调用，转发到新的注册函数"""
    setup_websocket_endpoint(app, process_command)