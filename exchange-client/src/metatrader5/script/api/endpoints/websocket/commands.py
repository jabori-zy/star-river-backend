from fastapi import WebSocket
from typing import Dict, Any, List
import logging
from .data_type_handler import DataTypeHandler
from .connection import subscribed_data
from mt5_terminal.terminal import Mt5Terminal

async def process_command(websocket: WebSocket, message: Dict[str, Any], terminal=None):
    """
    处理WebSocket命令
    
    Args:
        websocket: WebSocket连接
        message: 客户端发送的消息
        terminal: MT5终端实例
    """
    if "command" not in message:
        await websocket.send_json({
            "status": "error",
            "message": "缺少命令参数"
        })
        return
    
    command = message.get("command")
    data_type = message.get("data_type")
    params = message.get("params", {})
    frequency = message.get("frequency", 1000)  # 默认1000毫秒
    
    if command == "subscribe":
        await handle_subscribe(websocket, data_type, params, frequency, terminal)
    elif command == "unsubscribe":
        await handle_unsubscribe(websocket, data_type, params)
    elif command == "list_subscriptions":
        await handle_list_subscriptions(websocket)
    elif command == "update_frequency":
        await handle_update_frequency(websocket, data_type, params, frequency)
    else:
        await websocket.send_json({
            "status": "error",
            "message": f"未知命令: {command}"
        })

async def handle_subscribe(websocket: WebSocket, data_type: str, params: Dict[str, Any], frequency: int, terminal: Mt5Terminal):
    """处理订阅命令"""
    if not data_type:
        await websocket.send_json({
            "status": "error",
            "message": "缺少数据类型"
        })
        return
        
    # 验证参数
    is_valid, error_message = DataTypeHandler.validate_params(data_type, params)
    if not is_valid:
        await websocket.send_json({
            "status": "error",
            "message": error_message
        })
        return
        
    # 生成订阅键
    subscription_key = DataTypeHandler.get_subscription_key(data_type, params)
    if not subscription_key:
        await websocket.send_json({
            "status": "error",
            "message": "无法创建订阅键"
        })
        return
    
    #如果已订阅，则无需重复订阅
    if subscription_key in subscribed_data:
        await websocket.send_json({
            "status": "success",
            "message": "已订阅",
            "data": {
                "data_type": data_type,
                "params": params,
                "frequency": frequency
            }
        })
        return
        
    # 添加订阅
    if subscription_key not in subscribed_data:
        subscribed_data[subscription_key] = {"websockets": [], "frequency": frequency}
        
    if websocket not in subscribed_data[subscription_key]["websockets"]:
        subscribed_data[subscription_key]["websockets"].append(websocket)
        subscribed_data[subscription_key]["frequency"] = frequency
        
    # 发送成功响应
    await websocket.send_json({
        "status": "success",
        "message": f"成功订阅 {data_type}",
        "data": {
            "data_type": data_type,
            "params": params,
            "frequency": frequency
        }
    })
    
    logging.info(f"订阅成功: {subscription_key} 频率: {frequency}ms")
    logging.info(f"订阅数据: {subscribed_data}")

async def handle_unsubscribe(websocket: WebSocket, data_type: str, params: Dict[str, Any]):
    """处理取消订阅命令"""
    if not data_type:
        await websocket.send_json({
            "status": "error",
            "message": "缺少数据类型"
        })
        return
        
    # 验证参数
    is_valid, error_message = DataTypeHandler.validate_params(data_type, params)
    if not is_valid:
        await websocket.send_json({
            "status": "error",
            "message": error_message
        })
        return
        
    # 生成订阅键
    subscription_key = DataTypeHandler.get_subscription_key(data_type, params)
    if not subscription_key:
        await websocket.send_json({
            "status": "error",
            "message": "无法创建订阅键"
        })
        return
        
    # 移除订阅
    removed = False
    if subscription_key in subscribed_data and websocket in subscribed_data[subscription_key]["websockets"]:
        subscribed_data[subscription_key]["websockets"].remove(websocket)
        removed = True
        
        # 如果没有连接，清除整个订阅
        if not subscribed_data[subscription_key]["websockets"]:
            del subscribed_data[subscription_key]
            
    # 发送响应
    if removed:
        await websocket.send_json({
            "status": "success",
            "message": f"成功取消订阅 {data_type}",
            "data": {
                "data_type": data_type,
                "params": params
            }
        })
        print(f"取消订阅: {subscription_key}")
    else:
        await websocket.send_json({
            "status": "error",
            "message": f"未找到订阅 {data_type}"
        })

async def handle_list_subscriptions(websocket: WebSocket):
    """列出当前用户的所有订阅"""
    subscriptions = []
    
    for key, value in subscribed_data.items():
        if websocket in value["websockets"]:
            data_type = key[0]
            subscription_info = {
                "data_type": data_type,
                "params": {},
                "frequency": value["frequency"]
            }
            
            # 根据数据类型填充参数
            if data_type == "kline" and len(key) >= 3:
                subscription_info["params"] = {
                    "symbol": key[1],
                    "interval": key[2]
                }
            elif data_type == "order":
                subscription_info["params"] = {}
            elif data_type == "position":
                if len(key) >= 2:
                    subscription_info["params"]["symbol"] = key[1]
            elif data_type == "account":
                subscription_info["params"] = {}
            elif data_type == "tick" and len(key) >= 2:
                subscription_info["params"] = {
                    "symbol": key[1]
                }
                
            subscriptions.append(subscription_info)
    
    await websocket.send_json({
        "status": "success",
        "message": "当前订阅列表",
        "data": subscriptions
    })

async def handle_update_frequency(websocket: WebSocket, data_type: str, params: Dict[str, Any], frequency: int):
    """更新订阅频率"""
    if not data_type:
        await websocket.send_json({
            "status": "error",
            "message": "缺少数据类型"
        })
        return
        
    if frequency <= 0:
        await websocket.send_json({
            "status": "error",
            "message": "频率必须大于0"
        })
        return
        
    # 验证参数
    is_valid, error_message = DataTypeHandler.validate_params(data_type, params)
    if not is_valid:
        await websocket.send_json({
            "status": "error",
            "message": error_message
        })
        return
        
    # 生成订阅键
    subscription_key = DataTypeHandler.get_subscription_key(data_type, params)
    if not subscription_key:
        await websocket.send_json({
            "status": "error",
            "message": "无法创建订阅键"
        })
        return
        
    # 更新频率
    updated = False
    if subscription_key in subscribed_data and websocket in subscribed_data[subscription_key]["websockets"]:
        subscribed_data[subscription_key]["frequency"] = frequency
        updated = True
            
    # 发送响应
    if updated:
        await websocket.send_json({
            "status": "success",
            "message": f"成功更新订阅频率 {data_type}",
            "data": {
                "data_type": data_type,
                "params": params,
                "frequency": frequency
            }
        })
    else:
        await websocket.send_json({
            "status": "error",
            "message": f"未找到订阅 {data_type}"
        }) 