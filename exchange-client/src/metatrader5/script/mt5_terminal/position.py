import logging
from typing import Optional
from parse import parse_position
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .terminal import Mt5Terminal

class PositionManager:
    def __init__(self, client: "Mt5Terminal"):
        self.client = client
    
    async def close_position(self, ticket: int):
        
        position = await self.get_position_by_id(ticket)
        if not position or len(position) == 0:
            return False
            
        position = position[0]
        symbol_info_tick = await self.client.symbol.get_symbol_info_tick(position['symbol'])

        if position['type'] == "buy":
            order_type = self.client.terminal.ORDER_TYPE_SELL
            price = symbol_info_tick['bid']
        else:
            order_type = self.client.terminal.ORDER_TYPE_BUY
            price = symbol_info_tick['ask']

        request = {
            "action": self.client.terminal.TRADE_ACTION_DEAL,
            "symbol": position['symbol'],
            "volume": position['volume'],
            "type": order_type,
            "price": price,
            "deviation": 20,
            "magic": 123456,
            "comment": "star river close position",
            "position": position['ticket'],
        }
        order_result = self.client.terminal.order_send(request)
        order_info = {
            "retcode": order_result.retcode,
            "order_id": order_result.order,
            "deal_id": order_result.deal,
            "volume": order_result.volume,
            "price": order_result.price,
            "bid": order_result.bid,
            "ask": order_result.ask,
            "comment": order_result.comment,
            "request_id": order_result.request_id
            }
        return order_info
    
    async def get_position_by_id(self, position_id: int) -> tuple[bool, list]:
        positions = self.client.terminal.positions_get(ticket=position_id)
        # 如果是None, 则有错误
        if positions is None:
            return False, self.client.terminal.last_error()
        
        logging.debug("get position by id original: " + str(positions))
        
        position_info = []
        for pos in positions:
            pos_info = parse_position(pos)
            logging.debug("get position by id parsed: " + str(pos_info))
            position_info.append(pos_info)  
        return True, position_info
    
    # 根据symbol获取持仓
    async def get_positions_by_symbol(self, symbol: str) -> list:
        
        positions = self.client.terminal.positions_get(symbol=symbol)
        # 如果是None, 则有错误
        if positions is None:
            return False, self.client.terminal.last_error()
        
        
        position_list = []
        for pos in positions:
            position_info = parse_position(pos)
            position_list.append(position_info)
        return True, position_list
    

    
    async def get_position_number(self, symbol: str, position_side: Optional[str] = None) -> tuple[bool, int]:
        positions = await self.get_positions_by_symbol(symbol=symbol)
        # 如果获取持仓失败, 则返回错误
        if not positions[0]:
            return False, positions[1]
        
        # 判断是否需要按side统计
        if position_side:
            position_number = 0
            for pos in positions[1]:
                print(pos)
                if pos["position_type"] == position_side:
                    position_number += 1

            return True, position_number
        
        else:
            return True, len(positions[1]) 