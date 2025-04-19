import logging
from typing import Optional
from parse import parse_account_info, parse_terminal_info

class AccountManager:
    def __init__(self, terminal):
        self.terminal = terminal

    async def get_account_info(self):
        terminal_info = await self.terminal.get_terminal_info()
        if not terminal_info[0]:
            return False, self.terminal.terminal.last_error()
        
        account_info = self.terminal.terminal.account_info()
        if account_info is None:
            return False, self.terminal.terminal.last_error()
        account_info_dict = parse_account_info(account_info)
        return True, account_info_dict
    



