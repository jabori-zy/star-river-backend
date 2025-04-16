import asyncio
import pytz
from datetime import datetime
from typing import Optional
import logging
from parse import parse_terminal_info, parse_account_info
from .connection import ConnectionManager
from .market import MarketManager
from .order import OrderManager
from .position import PositionManager
from .deal import DealManager
from .account import AccountManager

class Mt5Client:
    def __init__(self, client_id: int, client) -> None:
        self.client_id = client_id
        self.account_id = None
        self.password = None
        self.server = None
        self.terminal_path = None
        self.timezone = pytz.timezone("Etc/UTC")
        self.client = client
        
        # 初始化各个管理器
        self.market = MarketManager(self)
        self.order = OrderManager(self)
        self.position = PositionManager(self)
        self.deal = DealManager(self)
        self.account = AccountManager(self)

    # 初始化终端
    async def initialize_terminal(self, terminal_path):
        logging.info(f"准备初始化 MetaTrader 5 客户端-{self.client_id}, 客户端路径: {terminal_path}")
        init_result = self.client.initialize(path=terminal_path)
        # 如果初始化成功，则获取客户端信息
        if init_result:
            terminal_info = await self.get_terminal_info()
            return True, terminal_info[1]
        else:
            return False, self.client.last_error()
        
    def ping(self):
        version = self.client.version()
        logging.info(f"MT5客户端-{self.client_id}的版本: {version}")
        if version is None:
            return False, self.client.last_error()
        return True, self.client_id
    
        
    async def login(self, account_id, password, server) -> bool:
        terminal_info = await self.get_terminal_info()
        if not terminal_info[0]:
            return False, self.client.last_error()
        
        login_result = self.client.login(account_id, password, server)
            
        if login_result:
            self.account_id = account_id
            self.password = password
            self.server = server
            account_info = await self.account.get_account_info()
            return True, account_info[1]
        else:
            return False, self.client.last_error()
    
    # 获取客户端信息
    async def get_terminal_info(self):
        terminal_info = self.client.terminal_info()
        if terminal_info is None:
            return False, self.client.last_error()
        terminal_info_dict = parse_terminal_info(terminal_info)
        return True, terminal_info_dict
    
    
    async def get_account_info(self):
        account_info = self.client.account_info()
        if account_info is None:
            return False, self.client.last_error()
        account_info_dict = parse_account_info(account_info)
        return True, account_info_dict

    def get_client_info(self):
        return {
            "client_id": self.get_client_id(),
            "terminal_path": self.get_terminal_path(),
            "account_id": self.get_account_id(),
            "server": self.get_server()
        }

    def get_client_id(self):
        return self.client_id
    
    def get_terminal_path(self):
        return self.terminal_path

    # 账户相关方法
    def set_account_id(self, account_id):
        self.account_id = account_id

    def set_password(self, password):
        self.password = password

    def set_server(self, server):
        self.server = server

    def set_terminal_path(self, terminal_path):
        self.terminal_path = terminal_path
    
    def get_account_id(self):
        return self.account_id

    def get_password(self):
        return self.password

    def get_server(self):
        return self.server
    
    def get_client(self):
        return self.client

