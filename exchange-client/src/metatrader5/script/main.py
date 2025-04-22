import uvicorn
import argparse
import logging
from api import create_app

def start_server(port=8001):
    """启动FastAPI服务器"""
    # 配置日志
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    logger = logging.getLogger("MT5服务")
    logger.info(f"准备启动MT5终端服务，端口：{port}")
    
    app = create_app()
    logger.info(f"MT5终端服务初始化完成，正在启动...")
    
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=port,
        reload=False,
        log_level="info"
    )

if __name__ == "__main__":
    # 解析命令行参数
    parser = argparse.ArgumentParser(description='启动MT5终端服务')
    parser.add_argument('--port', type=int, default=8001, help='服务端口号')
    args = parser.parse_args()
    
    # 启动服务
    start_server(port=args.port) 