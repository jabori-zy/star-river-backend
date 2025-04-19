import importlib
import sys
import os
import asyncio
script_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))  # 获取script目录
if script_dir not in sys.path:
    sys.path.append(script_dir)

from client_manager import mt5_client_manager
import logging
from uvicorn.logging import DefaultFormatter

# 配置日志
handler = logging.StreamHandler()
handler.setFormatter(DefaultFormatter("%(levelprefix)s %(message)s"))
logging.basicConfig(
    level=logging.DEBUG,
    handlers=[handler]
)

# 客户端1
mt5_client_manager.create_terminal(1)
mt5_client_1 = mt5_client_manager.get_terminal(1)

# 客户端2
mt5_client_manager.create_terminal(2)
mt5_client_2 = mt5_client_manager.get_terminal(2) 

# 客户端3
mt5_client_manager.create_terminal(3)
mt5_client_3 = mt5_client_manager.get_terminal(3) 




async def main():
    print("客户端1的MT5模块ID:", id(mt5_client_1.terminal))
    print("客户端2的MT5模块ID:", id(mt5_client_2.terminal))
    # print("客户端3的MT5模块ID:", id(mt5_client_3.client))
    init_result_1 = await mt5_client_1.initialize_terminal(terminal_path="D:/Program Files/MetaTrader 5-1/terminal64.exe")
    print(init_result_1)
    if init_result_1[0]:
        login_result_1 = await mt5_client_1.login(account_id=23643, password="HhazJ520!!!!", server="EBCFinancialGroupKY-Demo")
        print(login_result_1)
    
    init_result_2 = await mt5_client_2.initialize_terminal(terminal_path="D:/Program Files/MetaTrader 5-2/terminal64.exe")
    print(init_result_2)
    if init_result_2[0]:
        login_result_2 = await mt5_client_2.login(account_id=76898751, password="HhazJ520....", server="Exness-MT5Trial5")
        print(login_result_2)
        
    init_result_3 = await mt5_client_3.initialize_terminal(terminal_path="D:/Program Files/MetaTrader 5-3/terminal64.exe")
    print(init_result_3)
    if init_result_3[0]:
        login_result_3 = await mt5_client_3.login(account_id=27077, password="Ebc@123456", server="EBCFinancialGroupKY-Demo")
        print(login_result_3)
        
        

    


if __name__ == "__main__":
    asyncio.run(main())


