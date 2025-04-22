import subprocess
import sys
import os
import argparse
import time
import json
import logging

# 配置日志
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

# 默认配置
DEFAULT_CONFIG = [
    {
        "port": 8001,
        "terminal_path": "D:/Program Files/MetaTrader 5-1/terminal64.exe",
        "description": "MT5终端1"
    },
    {
        "port": 8002,
        "terminal_path": "D:/Program Files/MetaTrader 5-2/terminal64.exe",
        "description": "MT5终端2"
    }
]

def start_terminal(port, terminal_path, description):
    """启动一个终端服务"""
    cmd = [sys.executable, "main.py", "--port", str(port)]
    logger.info(f"启动终端服务 - 端口: {port}, 路径: {terminal_path}, 描述: {description}")
    
    # 使用环境变量传递终端配置信息
    env = os.environ.copy()
    env["MT5_TERMINAL_PATH"] = terminal_path
    env["MT5_TERMINAL_DESCRIPTION"] = description
    
    # 以子进程方式启动服务
    try:
        process = subprocess.Popen(
            cmd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        return process
    except Exception as e:
        logger.error(f"启动终端服务失败: {e}")
        return None

def main():
    """主函数"""
    parser = argparse.ArgumentParser(description='启动多个MT5终端服务')
    parser.add_argument('--config', type=str, help='终端配置文件路径')
    args = parser.parse_args()
    
    # 加载配置
    config = DEFAULT_CONFIG
    if args.config:
        try:
            with open(args.config, 'r') as f:
                config = json.load(f)
            logger.info(f"从配置文件加载了 {len(config)} 个终端配置")
        except Exception as e:
            logger.error(f"加载配置文件失败: {e}, 使用默认配置")
    
    # 启动所有终端服务
    processes = []
    for terminal_config in config:
        port = terminal_config.get("port")
        terminal_path = terminal_config.get("terminal_path")
        description = terminal_config.get("description", f"MT5终端{port}")
        
        process = start_terminal(port, terminal_path, description)
        if process:
            processes.append((process, terminal_config))
            # 延迟一点时间，避免同时启动多个服务
            time.sleep(1)
    
    logger.info(f"成功启动 {len(processes)} 个终端服务")
    
    try:
        # 保持脚本运行，直到按Ctrl+C
        logger.info("按Ctrl+C停止所有服务")
        while True:
            time.sleep(1)
            
            # 检查子进程状态
            for i, (process, config) in enumerate(processes[:]):
                if process.poll() is not None:
                    logger.warning(f"终端服务已停止 - 端口: {config['port']}, 退出码: {process.returncode}")
                    # 尝试重启
                    logger.info(f"尝试重启终端服务 - 端口: {config['port']}")
                    new_process = start_terminal(
                        config['port'], 
                        config.get('terminal_path'), 
                        config.get('description', f"MT5终端{config['port']}")
                    )
                    if new_process:
                        processes[i] = (new_process, config)
                    else:
                        processes.remove((process, config))
    
    except KeyboardInterrupt:
        logger.info("正在关闭所有终端服务...")
        for process, _ in processes:
            try:
                process.terminate()
            except:
                pass
        
        # 等待所有进程完全关闭
        for process, config in processes:
            try:
                process.wait(timeout=5)
                logger.info(f"终端服务已停止 - 端口: {config['port']}")
            except subprocess.TimeoutExpired:
                logger.warning(f"终端服务无响应，强制终止 - 端口: {config['port']}")
                process.kill()

if __name__ == "__main__":
    main() 