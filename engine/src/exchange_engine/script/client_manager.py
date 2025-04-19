from mt5_terminal.terminal import Mt5Terminal
import importlib
import sys

def get_mt5_module(client_id: int):
    module_name = f"MetaTrader5_{client_id}"
    if module_name not in sys.modules:
        original_module = importlib.import_module("MetaTrader5")
        # 使用新名称添加到sys.modules
        sys.modules[module_name] = original_module
        print(original_module)
    return sys.modules[module_name]


class Mt5ClientManager:
    def __init__(self):
        self.terminals = {}  # 存储终端ID和对应的Mt5Client实例
        self.mt5_modules = {}  # 存储终端ID和对应的MetaTrader5模块


    def ping_terminal(self, terminal_id: int) -> bool:
        terminal = self.get_terminal(terminal_id)
        if terminal is None:
            return False, "终端不存在"
        return terminal.ping()

    def create_terminal(self, terminal_id: int) -> Mt5Terminal:
        if terminal_id not in self.terminals:
            mt5_module = get_mt5_module(terminal_id)
            self.terminals[terminal_id] = Mt5Terminal(terminal_id, mt5_module)
        return self.terminals[terminal_id]
    

    
    def get_terminal(self, terminal_id: int) -> Mt5Terminal:
        if terminal_id not in self.terminals:
            return None
        return self.terminals[terminal_id]
    

    def delete_terminal(self, terminal_id: int):
        if terminal_id in self.terminals:
            del self.terminals[terminal_id]


mt5_client_manager = Mt5ClientManager()

