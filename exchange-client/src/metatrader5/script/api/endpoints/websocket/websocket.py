from fastapi import FastAPI
import logging
from mt5_terminal.terminal import Mt5Terminal

# 导入拆分后的各个模块
from .connection import setup_websocket_endpoint
from .commands import process_command
    
def register_websocket_routes(app: FastAPI):
    """兼容旧版调用，转发到新的注册函数"""
    setup_websocket_endpoint(app, process_command)