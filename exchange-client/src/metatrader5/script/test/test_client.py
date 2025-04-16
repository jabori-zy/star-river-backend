# import MetaTrader5 as mt5_1
# import MetaTrader5 as mt5_2
# import MetaTrader5 as mt5_3
import importlib
import sys
import os
script_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))  # 获取script目录
if script_dir not in sys.path:
    sys.path.append(script_dir)
from mt5_client.client import Mt5Client

def get_mt5_module(client_id: int):
    module_name = f"MetaTrader5_{client_id}"
    if module_name not in sys.modules:
        original_module = importlib.import_module("sdk.MetaTrader5")
        # 使用新名称添加到sys.modules
        sys.modules[module_name] = original_module
    print(sys.modules[module_name])
    return sys.modules[module_name]

mt5_1 = get_mt5_module(1)
mt5_2 = get_mt5_module(2)
mt5_3 = get_mt5_module(3)

print("客户端1的MT5模块ID:", id(mt5_1))
print("客户端2的MT5模块ID:", id(mt5_2))
print("客户端3的MT5模块ID:", id(mt5_3))

mt5_1.initialize(path="D:/Program Files/MetaTrader 5-1/terminal64.exe")
mt5_1.login(23643, 'HhazJ520!!!!', 'EBCFinancialGroupKY-Demo')
print(mt5_1.terminal_info())
mt5_2.initialize(path="D:/Program Files/MetaTrader 5-2/terminal64.exe")
mt5_2.login(76898751, 'HhazJ520....', 'Exness-MT5Trial5')
print(mt5_2.terminal_info())
mt5_3.initialize(path="D:/Program Files/MetaTrader 5-3/terminal64.exe")
mt5_3.login(27077, 'Ebc@123456', 'EBCFinancialGroupKY-Demo')
print(mt5_3.terminal_info())



