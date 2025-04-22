import asyncio
import MetaTrader5 as mt5
import logging
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .terminal import Mt5Terminal

class ConnectionManager:
    def __init__(self, terminal: "Mt5Terminal"):
        self.terminal = terminal
    
    async def initialize(self, terminal_path):
        # 获取客户端信息
        terminal_info = await self.terminal.account.get_terminal_info()
        print(f"获取客户端信息: {terminal_info}")
        logging.info(f"获取客户端信息: {terminal_info}")
        is_initialized = terminal_info[0]
        # 如果获取客户端信息失败，则初始化
        if not is_initialized:
            # 如果客户端未初始化，则初始化
            logging.info(f"准备初始化 MetaTrader 5 客户端-{self.terminal.client_id}, 客户端路径: {terminal_path}")
            print(f"准备初始化 MetaTrader 5 客户端-{self.terminal.client_id}, 客户端路径: {terminal_path}")
            init_result = self.terminal.terminal.initialize(terminal_path=terminal_path)
            print(f"初始化结果: {init_result}")
            # 如果初始化成功，则获取客户端信息
            if init_result:
                terminal_info = await self.terminal.account.get_terminal_info()
                print(f"获取客户端信息: {terminal_info}")
                logging.info(f"获取客户端信息: {terminal_info}")
                is_initialized = terminal_info[0]
                if not is_initialized:
                    return False

        # 如果初始化失败，则关闭客户端并返回False
        if not init_result:
            logging.error(f"初始化 MetaTrader 5 失败。错误码：{self.terminal.terminal.last_error()}")
            self.terminal.terminal.shutdown()
            return False
        self.terminal.set_terminal_path(terminal_path)
        return True
        
    async def login(self, account_id, password, server) -> bool: 
        login_result = mt5.login(account_id, password, server)
            
        if login_result:
            print("Connected to MetaTrader 5")
            self.terminal.set_login(account_id)
            self.terminal.set_password(password)
            self.terminal.set_server(server)
            return login_result
        else:
            print("连接 MetaTrader 5 失败。错误码：", mt5.last_error())
            return login_result