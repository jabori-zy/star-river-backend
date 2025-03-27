from fastapi import FastAPI, WebSocket, WebSocketDisconnect
import uvicorn
import datetime
import random
import asyncio
from typing import Dict, Any, Tuple, Optional, List
from mt5_client import Mt5Client
from utils import get_time_frame
import logging


active_connections = []
subscribed_data = {}

def register_websocket_routes(app: FastAPI):
    @app.websocket("/ws")
    async def websocket_endpoint(websocket: WebSocket):
        await websocket.accept()
        active_connections.append(websocket)
        print(f"客户端连接成功: {websocket}")
        print(f"active_connections: {active_connections}")

        try:
            # 发送欢迎消息
            await websocket.send_json({
                "type": "welcome",
                "message": "已连接到MT5行情服务",
                "timestamp": datetime.datetime.now().isoformat()
            })
            # 启动模拟数据推送任务
            # data_task = asyncio.create_task(send_demo_data(websocket))
            while True:
                try:
                    message = await websocket.receive_json()
                    print(f"收到: {message}")
                    await process_command(websocket, message)
                except Exception as e:
                    print(f"错误: {e}")
                    break

        except WebSocketDisconnect:
            print("客户端断开连接")
        except Exception as e:
            print(f"WebSocket错误: {e}")
        finally:
            active_connections.remove(websocket)


async def send_demo_data(websocket: WebSocket):
    """发送模拟行情数据"""
    symbols = ["XAUUSD"]
    timeframes = ["M1"]
    
    try:
        while True:
            # 为每个交易对生成随机数据
            for symbol in symbols:
                # 基础价格
                base_price = {
                    "XAUUSD": 2000.0,
                    "EURUSD": 1.1,
                    "GBPUSD": 1.3,
                    "USDJPY": 150.0
                }.get(symbol, 100.0)
                
                # 随机波动
                price_change = (random.random() - 0.5) * 0.01 * base_price
                current_price = base_price + price_change
                
                # 生成K线数据
                for timeframe in timeframes:
                    # 随机波动
                    open_price = current_price - (random.random() - 0.5) * 0.005 * base_price
                    high_price = max(open_price, current_price) + random.random() * 0.002 * base_price
                    low_price = min(open_price, current_price) - random.random() * 0.002 * base_price
                    close_price = current_price
                    volume = random.randint(100, 1000)
                    
                    # 当前时间
                    now = datetime.datetime.now()
                    
                    # 发送数据
                    kline_data = {
                        "type": "kline",
                        "symbol": symbol,
                        "timeframe": timeframe,
                        "data": {
                            "time": now.isoformat(),
                            "open": round(open_price, 5),
                            "high": round(high_price, 5),
                            "low": round(low_price, 5),
                            "close": round(close_price, 5),
                            "volume": volume
                        }
                    }
                    
                    await websocket.send_json(kline_data)
                    
                    # 短暂延迟，避免消息过多
                    await asyncio.sleep(0.1)
            
            # 每秒更新一次
            await asyncio.sleep(1)
    
    except asyncio.CancelledError:
        # 任务被取消时正常退出
        pass
    except Exception as e:
        print(f"发送数据错误: {e}")



# 数据类型处理器
class DataTypeHandler:
    @staticmethod
    def get_subscription_key(data_type: str, params: Dict[str, Any]) -> Optional[Tuple]:
        """根据数据类型和参数生成订阅键"""
        if data_type == "kline":
            symbol = params.get("symbol")
            interval = params.get("interval")
            if not symbol or not interval:
                return None
            return ("kline", symbol, interval)
            
        elif data_type == "order":
            account_id = params.get("account_id")
            if not account_id:
                return None
            return ("order", account_id)
            
        elif data_type == "position":
            account_id = params.get("account_id")
            symbol = params.get("symbol")  # 可选
            if not account_id:
                return None
            return ("position", account_id, symbol) if symbol else ("position", account_id)
            
        elif data_type == "account":
            account_id = params.get("account_id")
            if not account_id:
                return None
            return ("account", account_id)
            
        elif data_type == "tick":
            symbol = params.get("symbol")
            if not symbol:
                return None
            return ("tick", symbol)
            
        return None  # 未知数据类型
    
    @staticmethod
    def get_required_params(data_type: str) -> List[str]:
        """获取指定数据类型的必要参数"""
        if data_type == "kline":
            return ["symbol", "interval"]
        elif data_type == "order":
            return ["account_id"]
        elif data_type == "position":
            return ["account_id"]
        elif data_type == "account":
            return ["account_id"]
        elif data_type == "tick":
            return ["symbol"]
        return []
    
    @staticmethod
    def validate_params(data_type: str, params: Dict[str, Any]) -> Tuple[bool, str]:
        """验证参数是否满足数据类型要求"""
        required_params = DataTypeHandler.get_required_params(data_type)
        
        for param in required_params:
            if param not in params or not params[param]:
                return False, f"缺少必要参数 {param}"
                
        return True, ""
    

async def process_command(websocket: WebSocket, message: Dict[str, Any]):
    """处理WebSocket命令"""
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
        await handle_subscribe(websocket, data_type, params, frequency)
    elif command == "unsubscribe":
        await handle_unsubscribe(websocket, data_type, params)
    elif command == "list_subscriptions":
        await handle_list_subscriptions(websocket)
    elif command == "update_frequency":
        await handle_update_frequency(websocket, data_type, params, frequency)
    else:
        await websocket.send_json({
            "command": command,
            "status": "error",
            "message": f"未知命令: {command}"
        })

async def handle_subscribe(websocket: WebSocket, data_type: str, params: Dict[str, Any], frequency: int):
    """处理订阅命令"""
    # 检查数据类型
    if not data_type:
        await websocket.send_json({
            "command": "subscribe",
            "status": "error",
            "message": "缺少必要参数 data_type"
        })
        return
    
    # 验证参数
    is_valid, error_msg = DataTypeHandler.validate_params(data_type, params)
    if not is_valid:
        await websocket.send_json({
            "command": "subscribe",
            "status": "error",
            "data_type": data_type,
            "message": error_msg
        })
        return
    
    # 获取订阅键
    subscription_key = DataTypeHandler.get_subscription_key(data_type, params)
    if not subscription_key:
        await websocket.send_json({
            "command": "subscribe",
            "status": "error",
            "data_type": data_type,
            "message": f"无法创建订阅键，参数无效或数据类型不支持: {data_type}"
        })
        return
    
    # 添加到订阅列表
    if websocket not in subscribed_data:
        subscribed_data[websocket] = {}
        
    # 检查是否已订阅
    is_new = subscription_key not in subscribed_data[websocket]
    
    # 存储订阅信息
    subscribed_data[websocket][subscription_key] = {
        "data_type": data_type,
        "params": params.copy(),
        "frequency": frequency,
        "last_update": 0
    }
    
    print(f"客户端订阅: {data_type} - {subscription_key} (频率: {frequency}ms)")
    
    await websocket.send_json({
        "command": "subscribe",
        "status": "success",
        "data_type": data_type,
        "params": params,
        "frequency": frequency,
        "message": "订阅成功" if is_new else "已更新订阅"
    })

async def handle_unsubscribe(websocket: WebSocket, data_type: str, params: Dict[str, Any]):
    """处理取消订阅命令"""
    # 检查数据类型
    if not data_type:
        await websocket.send_json({
            "command": "unsubscribe",
            "status": "error",
            "message": "缺少必要参数 data_type"
        })
        return
    
    # 验证参数
    is_valid, error_msg = DataTypeHandler.validate_params(data_type, params)
    if not is_valid:
        await websocket.send_json({
            "command": "unsubscribe",
            "status": "error",
            "data_type": data_type,
            "message": error_msg
        })
        return
    
    # 获取订阅键
    subscription_key = DataTypeHandler.get_subscription_key(data_type, params)
    if not subscription_key:
        await websocket.send_json({
            "command": "unsubscribe",
            "status": "error",
            "data_type": data_type,
            "message": f"无法创建订阅键，参数无效或数据类型不支持: {data_type}"
        })
        return
    
    # 检查客户端是否有任何订阅
    if websocket not in subscribed_data:
        await websocket.send_json({
            "command": "unsubscribe",
            "status": "success",
            "data_type": data_type,
            "params": params,
            "message": "未订阅"
        })
        return
    
    # 从订阅列表中移除
    was_subscribed = subscription_key in subscribed_data[websocket]
    
    if was_subscribed:
        del subscribed_data[websocket][subscription_key]
        print(f"客户端取消订阅: {data_type} - {subscription_key}")
        
        # 如果客户端没有其他订阅，清理字典
        if not subscribed_data[websocket]:
            del subscribed_data[websocket]
    
    await websocket.send_json({
        "command": "unsubscribe",
        "status": "success",
        "data_type": data_type,
        "params": params,
        "message": "取消订阅成功" if was_subscribed else "未订阅"
    })

async def handle_list_subscriptions(websocket: WebSocket):
    """处理列出订阅命令"""
    client_subscriptions = []
    
    if websocket in subscribed_data:
        for sub_key, sub_info in subscribed_data[websocket].items():
            client_subscriptions.append({
                "data_type": sub_info["data_type"],
                "params": sub_info["params"],
                "frequency": sub_info["frequency"]
            })
    
    await websocket.send_json({
        "command": "list_subscriptions",
        "status": "success",
        "subscriptions": client_subscriptions,
        "count": len(client_subscriptions)
    })

async def handle_update_frequency(websocket: WebSocket, data_type: str, params: Dict[str, Any], frequency: int):
    """处理更新频率命令"""
    # 检查数据类型
    if not data_type:
        await websocket.send_json({
            "command": "update_frequency",
            "status": "error",
            "message": "缺少必要参数 data_type"
        })
        return
    
    # 验证参数
    is_valid, error_msg = DataTypeHandler.validate_params(data_type, params)
    if not is_valid:
        await websocket.send_json({
            "command": "update_frequency",
            "status": "error",
            "data_type": data_type,
            "message": error_msg
        })
        return
    
    # 获取订阅键
    subscription_key = DataTypeHandler.get_subscription_key(data_type, params)
    if not subscription_key:
        await websocket.send_json({
            "command": "update_frequency",
            "status": "error",
            "data_type": data_type,
            "message": f"无法创建订阅键，参数无效或数据类型不支持: {data_type}"
        })
        return
    
    # 检查是否已订阅
    if (websocket not in subscribed_data or 
        subscription_key not in subscribed_data[websocket]):
        await websocket.send_json({
            "command": "update_frequency",
            "status": "error",
            "data_type": data_type,
            "params": params,
            "message": "未找到订阅，请先订阅"
        })
        return
        
    # 更新频率
    subscribed_data[websocket][subscription_key]["frequency"] = frequency
    print(f"客户端更新频率: {data_type} - {subscription_key} (新频率: {frequency}ms)")
    
    await websocket.send_json({
        "command": "update_frequency",
        "status": "success",
        "data_type": data_type,
        "params": params,
        "frequency": frequency,
        "message": "频率更新成功"
    })

async def data_push_task(mt5_client: Mt5Client):
    """数据推送任务"""
    while True:
        await asyncio.sleep(0.1)  # 每100毫秒检查一次
        
        # 这里可以添加从MT5获取数据的逻辑
        # 例如获取K线数据并推送给订阅者
        
        # 示例：推送K线数据
        await send_kline_data(mt5_client)
        
        # 示例：推送订单数据
        # await send_order_data()







async def send_kline_data(mt5_client: Mt5Client):
    """发送K线数据到订阅者"""
    # 检查是否有K线订阅
    current_time = int(datetime.datetime.now().timestamp() * 1000)  # 毫秒时间戳
    
    # 为每个订阅的交易对和时间周期获取K线数据
    for websocket, subscriptions in list(subscribed_data.items()):
        for sub_key, sub_info in list(subscriptions.items()):
            # 只处理K线数据类型
            if sub_info["data_type"] != "kline":
                continue
            
            # 获取交易对和时间周期
            symbol = sub_info["params"].get("symbol")
            interval = sub_info["params"].get("interval")

            # 检查是否需要发送（基于频率）
            last_update = sub_info.get("last_update", 0)
            frequency = sub_info.get("frequency", 1000)

            if not symbol or not interval:
                continue

            if current_time - last_update >= frequency:
                # 更新最后发送时间
                sub_info["last_update"] = current_time

                # await websocket.send_json({
                #                 "type": "data",
                #                 "data_type": "kline",
                #                 "params": sub_info["params"],
                #                 "data": {"interval": interval, "symbol": symbol},
                #                 "timestamp": current_time
                #             })
            
            
                try:
                    # 从MT5获取最新K线数据
                    kline_data = await mt5_client.get_latest_kline(symbol, interval)
                    
                    if kline_data:
                        # 发送数据给订阅者
                        try:
                            await websocket.send_json({
                                    "data_type": "kline",
                                    "data": kline_data,
                                    "timestamp": current_time
                                })
                        except Exception as e:
                            logging.error(f"发送数据失败: {e}")
                            # 如果发送失败，可能连接已断开，清理订阅
                            if websocket in subscribed_data:
                                del subscribed_data[websocket]
                            if websocket in active_connections:
                                active_connections.remove(websocket)
                            continue  # 跳过此订阅的后续处理
                        
                    else:
                        logging.info(f"未获取到K线数据: {symbol} {interval}")
                        
                except Exception as e:
                    logging.error(f"获取K线数据错误 ({symbol} {interval}): {e}")