import asyncio
import logging
from datetime import datetime
from util import get_order_type, get_order_type_filling
from parse import parse_order
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .client import Mt5Client


class OrderManager:
    def __init__(self, client: "Mt5Client"):
        self.client = client
    
    # 创建订单
    async def create_order(self, order_type, order_side, symbol, volume, price, tp, sl):
        point = await self.client.market.get_symbol_info(symbol)
        if point[0]:
            point = point[1]["point"]
        else:
            return False, point[1]
        
        # 限价单
        if order_type == "limit":
            # 挂单
            action = self.client.client.TRADE_ACTION_PENDING
            # 多头
            if order_side == "long":
                type = self.client.client.ORDER_TYPE_BUY_LIMIT
                if tp is not None:
                    tp_price = price + tp * point
                if sl is not None:
                    sl_price = price - sl * point
            # 空头
            elif order_side == "short":
                type = self.client.client.ORDER_TYPE_SELL_LIMIT
                if tp is not None:
                    tp_price = price - tp * point
                if sl is not None:
                    sl_price = price + sl * point

            request = {
                "action": action,
                "symbol": symbol,
                "volume": volume,
                "type": type,
                "price": price,
                "deviation": 20, #偏差
                "magic": 123456,
                "comment": "star river open position",
                "type_time": self.client.client.ORDER_TIME_GTC,
                "type_filling": self.client.client.ORDER_FILLING_FOK,
            }

            if tp is not None:
                request["tp"] = tp_price
            if sl is not None:
                request["sl"] = sl_price
            order_result = self.client.client.order_send(request)
            logging.debug("order_result: " + str(order_result))
            order_info = {
                "retcode": order_result.retcode,
                "order_id": order_result.order,
                "deal_id": order_result.deal,
                "symbol": symbol,
                "order_type": get_order_type(order_result.request.type),
                "type_filling": get_order_type_filling(order_result.request.type_filling),
                "volume": order_result.volume,
                "open_price": order_result.request.price,
                "bid": order_result.bid,
                "ask": order_result.ask,
                "comment": order_result.comment,
                "request_id": order_result.request_id
            }
        
            return True, order_info

        # 市价单
        elif order_type == "market":
            # 市价单
            action = self.client.client.TRADE_ACTION_DEAL
            current_price = self.client.client.symbol_info(symbol).ask
            # 多头
            if order_side == "long":
                type = self.client.client.ORDER_TYPE_BUY
                if tp is not None:
                    tp_price = current_price + tp * point
                if sl is not None:
                    sl_price = current_price - sl * point
                
            # 空头
            elif order_side == "short":
                type = self.client.client.ORDER_TYPE_SELL
                if tp is not None:
                    tp_price = current_price - tp * point
                if sl is not None:
                    sl_price = current_price + sl * point

            request = {
                "action": action,
                "symbol": symbol,
                "volume": volume,
                "type": type,
                "price": current_price,
                "deviation": 20, #偏差
                "magic": 123456,
                "comment": "star river open position",
                "type_time": self.client.client.ORDER_TIME_GTC,
                "type_filling": self.client.client.ORDER_FILLING_FOK,

            }

            if tp is not None:
                request["tp"] = tp_price
            if sl is not None:
                request["sl"] = sl_price

            order_result = self.client.client.order_send(request)
            logging.debug("order_result: " + str(order_result))
            order_info = {
                "retcode": order_result.retcode,
                "order_id": order_result.order,
                "deal_id": order_result.deal,
                "symbol": symbol,
                "order_type": get_order_type(order_result.request.type),
                "type_filling": get_order_type_filling(order_result.request.type_filling),
                "volume": order_result.volume,
                "open_price": order_result.price,
                "bid": order_result.bid,
                "ask": order_result.ask,
                "comment": order_result.comment,
                "request_id": order_result.request_id
            }
            
            return True, order_info
    
    # 根据id获取订单信息
    async def get_order_by_id(self, order_id: int) -> tuple[bool, list]:
        
        orders = self.client.client.orders_get(ticket=order_id)
        logging.debug("get order by id: " + str(orders))
        if orders is None:
            return False, self.client.client.last_error()
        
        order_result = []
        for order in orders:
            logging.info("get order by id: " + str(order))
            order_info = parse_order(order)
            if (order_info["order_type"] == "order_type_buy" or order_info["order_type"] == "order_type_sell") and order_info["status"] == "filled":
                position_id = order_info["position_id"]
                deals = await self.client.deal.get_deals_by_position_id(position_id)
                # 找到entry=in的deal
                for deal in deals:
                    if deal["entry"] == "in":
                        order_info["open_price"] = deal["price"] # 设置市价单的入场价格
                        break
            order_result.append(order_info)
        return True, order_result
    
    async def get_order_by_symbol(self, symbol: str):
        
        orders = self.client.client.orders_get(symbol=symbol)
        if orders is None:
            return False, self.client.client.last_error()
        
        order_result = []
        for order in orders:
            order_info = parse_order(order)
            order_result.append(order_info)
        return True, order_result
    
    async def get_history_order_by_id(self, order_id: int):
        orders = self.client.client.history_orders_get(ticket=order_id)
        logging.debug("get history order by id: " + str(orders))
        if orders is None:
            return False, self.client.client.last_error()
        
        if len(orders) == 0:
            return False, "订单不存在"
        
        order_result = []
        for order in orders:
            order_info = parse_order(order)
            if (order_info["order_type"] == "order_type_buy" or order_info["order_type"] == "order_type_sell") and order_info["status"] == "filled":
                position_id = order_info["position_id"]
                deals = await self.client.deal.get_deals_by_position_id(position_id)
                if not deals[0]:
                    return False, deals[1]
                # 找到entry=in的deal
                for deal in deals[1]:
                    if deal["entry"] == "in":
                        order_info["open_price"] = deal["price"] # 设置市价单的入场价格
                        break
            logging.debug("get history order by id parsed: " + str(order_info))
            order_result.append(order_info)
        return True, order_result
    
    
    async def get_history_order_by_position_id(self, position_id: int):

        orders = self.client.client.history_orders_get(position=position_id)
        if orders is None:
            return False, self.client.client.last_error()

        order_result = []
        for order in orders:
            order_info = parse_order(order)
            order_result.append(order_info)
        return True, order_result
    
    async def get_history_order_by_symbol(self, symbol: int):
        
        from_date=datetime(2020,1,1)
        to_date=datetime.now()
        symbol_name = f"*{symbol}*"
        orders = self.client.client.history_orders_get(from_date, to_date, group=symbol_name)
        if orders is None:
            return False, self.client.client.last_error()

        order_result = []
        for order in orders:
            order_info = parse_order(order)
            order_result.append(order_info)
        return True, order_result