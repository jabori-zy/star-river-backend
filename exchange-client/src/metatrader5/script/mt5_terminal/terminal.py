import pytz
from datetime import datetime
from typing import Optional
import logging
from parse import parse_terminal_info, parse_account_info
from .market import MarketManager
from .order import OrderManager
from .position import PositionManager
from .deal import DealManager
import MetaTrader5 as mt5
import numpy as np

class Mt5Terminal:
    def __init__(self) -> None:
        self.login = None
        self.password = None
        self.server = None
        self.terminal_path = None
        self.timezone = pytz.timezone("Etc/UTC")
        self.terminal = mt5
        
        # 初始化各个管理器
        self.market = MarketManager(self)
        self.order = OrderManager(self)
        self.position = PositionManager(self)
        self.deal = DealManager(self)

    # 初始化终端
    async def initialize_terminal(self, terminal_path, login, password, server):
        init_result = self.terminal.initialize(path=terminal_path, login=login, password=password, server=server)
        # 如果初始化成功，则获取客户端信息
        if init_result:
            terminal_info = await self.get_terminal_info()
            return True, terminal_info[1]
        else:
            return False, self.terminal.last_error()
        
    def ping(self):
        version = self.terminal.version()
        if version is None:
            return False, self.terminal.last_error()
        return True, self.client_id
    
        
    async def login_mt5(self, login, password, server) -> tuple[bool, dict]:
        terminal_info = await self.get_terminal_info()
        if not terminal_info[0]:
            return False, self.terminal.last_error()
        
        login_result = self.terminal.login(login, password, server)
            
        if login_result:
            self.login = login
            self.password = password
            self.server = server
            account_info = await self.get_account_info()
            return True, account_info[1]
        else:
            return False, self.terminal.last_error()
    
    # 获取客户端信息
    async def get_terminal_info(self):
        terminal_info = self.terminal.terminal_info()
        if terminal_info is None:
            return False, self.terminal.last_error()
        terminal_info_dict = parse_terminal_info(terminal_info)
        return True, terminal_info_dict
    
    
    async def get_account_info(self):
        account_info = self.terminal.account_info()
        

        if account_info is None:
            return False, self.terminal.last_error()
        
        # 获取终端信息
        terminal_info = self.terminal.terminal_info()
        if terminal_info is None:
            return False, self.terminal.last_error()
        
        account_info_dict = parse_account_info(account_info, terminal_info)
        return True, account_info_dict
    
    def get_terminal_path(self):
        return self.terminal_path

    # 账户相关方法
    def set_login(self, login):
        self.login = login

    def set_password(self, password):
        self.password = password

    def set_server(self, server):
        self.server = server

    def set_terminal_path(self, terminal_path):
        self.terminal_path = terminal_path
    
    def get_login(self):
        return self.login

    def get_password(self):
        return self.password

    def get_server(self):
        return self.server
    
    def get_client(self):
        return self.terminal

