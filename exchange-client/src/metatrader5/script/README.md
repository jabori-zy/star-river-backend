# Star River MT5交易接口

## 项目结构

```
script/
├── api/                  # API模块
│   ├── endpoints/        # API端点
│   │   ├── basic.py      # 基础API
│   │   ├── order.py      # 订单API
│   │   ├── position.py   # 持仓API
│   │   └── trade.py      # 交易API
│   ├── app.py            # FastAPI应用创建
│   ├── router.py         # 路由收集
│   └── websocket.py      # WebSocket实现
├── mt5_client/           # MT5客户端模块
│   ├── client.py         # 主客户端类
│   ├── connection.py     # 连接管理
│   ├── deal.py           # 成交明细
│   ├── kline.py          # K线数据
│   ├── order.py          # 订单管理
│   ├── position.py       # 持仓管理
│   └── symbol.py         # 交易品种
├── client.py             # 客户端实例化
├── main.py               # 主程序入口
├── parse.py              # 数据解析工具
└── util.py               # 工具函数
```

## 运行方法

1. 确保已安装所有依赖:
```
pip install fastapi uvicorn MetaTrader5 pytz
```

2. 启动服务:
```
cd exchange-client/src/metatrader5/script
python main.py
```

3. 服务将在 http://0.0.0.0:8000 上运行，可以通过以下URL访问API文档:
```
http://localhost:8000/docs
```

## API说明

- `/ping` - 检查服务状态
- `/client_status` - 检查MT5客户端状态
- `/initialize_client` - 初始化MT5客户端
- `/login` - 登录MT5账户
- `/get_symbols` - 获取交易品种列表
- `/get_symbol_info` - 获取交易品种信息
- `/get_kline_series` - 获取K线数据
- `/order/*` - 订单相关API
- `/position/*` - 持仓相关API
- `/trade/*` - 交易相关API

## WebSocket接口

通过WebSocket接口 `/ws` 可以订阅实时行情数据。 