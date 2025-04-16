import logging
from datetime import datetime
from parse import parse_deal
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .client import Mt5Client

class DealManager:
    def __init__(self, client: "Mt5Client"):
        self.client = client
    
    # 根据持仓id获取成交明细
    async def get_deals_by_position_id(self, position_id: int) -> tuple[bool, list]:
        
        deals = self.client.client.history_deals_get(position=position_id)
        if deals is None:
            return False, self.client.client.last_error()
        
        deal_list = []
        for deal in deals:
            deal_info = parse_deal(deal)
            deal_list.append(deal_info)
        return True, deal_list
    
    async def get_deals_by_deal_id(self, deal_id: int):
        
        deals = self.client.client.history_deals_get(ticket=deal_id)
        if deals is None:
            return False, self.client.client.last_error()
        
        deal_list = []
        for deal in deals:
            logging.info(deals)
            deal_info = parse_deal(deal)
            deal_list.append(deal_info)
        return True, deal_list

    async def get_deals_by_symbol(self, symbol: str):
        
        from_date=datetime(2020,1,1) 
        to_date=datetime.now()
        symbol_name = f"*{symbol}*"
        deals = self.client.client.history_deals_get(from_date, to_date, group=symbol_name)
        if deals is None:
            return False, self.client.client.last_error()
        
        deal_list = []
        for deal in deals:
            deal_info = parse_deal(deal)
            deal_list.append(deal_info)

        return True, deal_list
    
    async def get_deals_by_order_id(self, order_id: int):
        
        # 1. 根据order_id获取position_id
        order = await self.client.order.get_order_by_id(order_id)
        # 如果列表为空，则从历史订单中获取
        if not order[0]:
            return False, order[1]
        
        if len(order[1]) == 0:
            order = await self.client.order.get_history_order_by_id(order_id)
        
        if len(order[1]) == 0:
            return False, "订单不存在"
        
        logging.debug("get deals by order id order: " + str(order))
        position_id = order[1][0]["position_id"]
        # 2. 根据position_id获取成交明细
        deals = await self.get_deals_by_position_id(position_id)
        if not deals[0]:
            return False, deals[1]
        
        logging.debug("get deals by order id deals: " + str(deals))
        # 3. 仓位的成交明细会有多个，找到匹配的order_id的明细
        deal_result = []
        for deal in deals[1]:
            if deal["order_id"] == order_id:
                deal_result.append(deal)
        return True, deal_result