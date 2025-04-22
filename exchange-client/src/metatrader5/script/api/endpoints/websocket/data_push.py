import datetime
import asyncio
import logging
from typing import Dict, Any
from mt5_terminal.terminal import Mt5Terminal
from .connection import subscribed_data

async def data_push_task(terminal: Mt5Terminal):
    """数据推送任务"""
    logging.info("数据推送任务启动")
    
    while True:
        try:
            # 目前只实现K线数据推送
            await send_kline_data(terminal)
            
            # 加入其他数据类型推送...
            
            # 控制循环频率，避免过度消耗资源
            await asyncio.sleep(0.1)  # 100ms检查一次
        except asyncio.CancelledError:
            logging.info("数据推送任务被取消")
            break
        except Exception as e:
            logging.error(f"数据推送错误: {e}")
            # 出错后短暂延迟
            await asyncio.sleep(5)


async def send_kline_data(terminal: Mt5Terminal):
    """发送K线数据"""
    current_time = datetime.datetime.now().timestamp() * 1000  # 当前时间毫秒
    
    # 查找所有K线订阅
    for key, value in list(subscribed_data.items()):
        if key[0] != "kline":
            continue
            
        # 获取订阅信息
        symbol = key[1]
        interval = key[2]
        websockets = value["websockets"]
        frequency = value["frequency"]
        
        # 检查是否应该发送数据 (利用last_push_time字段)
        last_push_time = value.get("last_push_time", 0)
        if current_time - last_push_time < frequency:
            continue
            
        # 获取最新K线数据
        try:
            latest_kline = await terminal.market.get_latest_kline(symbol, interval)
            # 如果获取失败，则不发送数据
            if not latest_kline[0]:
                continue
                
            # 更新最后推送时间
            subscribed_data[key]["last_push_time"] = current_time
            
            # 格式化数据
            kline_data = {
                "type": "kline",
                "symbol": symbol,
                "timeframe": interval,
                "data": latest_kline[1]
            }
            
            # 发送给所有订阅的连接
            for ws in websockets[:]:  # 使用副本遍历，以便安全移除
                try:
                    await ws.send_json(kline_data)
                except Exception as e:
                    logging.error(f"发送K线数据错误: {e}")
                    # 如果发送失败，移除这个连接
                    try:
                        websockets.remove(ws)
                    except ValueError:
                        pass
            
            # 如果没有连接，清除整个订阅
            if not websockets:
                del subscribed_data[key]
                
        except Exception as e:
            logging.error(f"获取K线数据错误: {e}")
            continue 