import asyncio
from mt5_terminal import Mt5Terminal
import time
import MetaTrader5 as mt5

async def test_mt5(mt5_client: Mt5Terminal):

    await mt5_client.initialize_terminal(r'C:/Program Files/MetaTrader 5/terminal64.exe')

    # mt5.set_account_id(23643)
    mt5_client.set_account_id(76898751)

    # mt5_client.set_password("HhazJ520!!!!")
    mt5_client.set_password("HhazJ520....")

    # mt5.set_server("EBCFinancialGroupKY-Demo")
    mt5_client.set_server("Exness-MT5Trial5")

    await mt5_client.login()

    klines = await mt5_client.get_latest_kline('BTCUSDm', "M1")
    print(klines)

    # order_info = await mt5_client.create_order("market", "long", 'BTCUSDm', 0.01, 82000.00, 5000, 5000)
    # print("open order_info", order_info)

    # time.sleep(2)

    position_info = await mt5_client.get_position_by_symbol('BTCUSDm')
    print("position_info", position_info)

    
    

    time.sleep(1000)


if __name__ == "__main__":
    mt5_client = Mt5Terminal()
    asyncio.run(test_mt5(mt5_client))



