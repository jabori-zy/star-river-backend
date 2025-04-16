from fastapi import FastAPI, WebSocket, WebSocketDisconnect
import datetime
from typing import List, Dict, Any
import logging

# 全局变量
active_connections: List[WebSocket] = []
subscribed_data: Dict[Any, Dict] = {}

def setup_websocket_endpoint(app: FastAPI, process_command_handler):
    """设置WebSocket端点"""
    @app.websocket("/ws")
    async def websocket_endpoint(websocket: WebSocket):
        await websocket.accept()
        active_connections.append(websocket)
        logging.info(f"客户端连接成功: {websocket}")
        logging.info(f"active_connections: {active_connections}")

        try:
            # 发送欢迎消息
            await websocket.send_json({
                "type": "welcome",
                "message": "已连接到MT5行情服务",
                "timestamp": datetime.datetime.now().isoformat()
            })
            while True:
                try:
                    message = await websocket.receive_json()
                    await process_command_handler(websocket, message)
                except Exception as e:
                    logging.error(f"错误: {e}")
                    break

        except WebSocketDisconnect:
            logging.info("客户端断开连接")
        except Exception as e:
            logging.error(f"WebSocket错误: {e}")
        finally:
            active_connections.remove(websocket) 