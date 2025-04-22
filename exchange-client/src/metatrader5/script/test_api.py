import requests
import json
import time

BASE_URL = "http://localhost:8002"

def pretty_print(data):
    """漂亮打印JSON数据"""
    print(json.dumps(data, indent=2, ensure_ascii=False))

def test_create_terminal():
    """测试创建终端"""
    print("\n=== 测试创建终端 ===")
    
    # 检查当前终端列表
    response = requests.get(f"{BASE_URL}/")
    data = response.json()
    print("当前终端列表:")
    pretty_print(data)
    
    # 创建新终端
    terminal_data = {
        "terminal_id": 1,
        "terminal_path": "D:/Program Files/MetaTrader 5-1/terminal64.exe",
        "description": "测试终端1"
    }
    
    response = requests.post(f"{BASE_URL}/terminals", json=terminal_data)
    data = response.json()
    print("\n创建终端结果:")
    pretty_print(data)
    
    # 再次检查终端列表
    response = requests.get(f"{BASE_URL}/")
    data = response.json()
    print("\n创建后终端列表:")
    pretty_print(data)

def test_get_terminal_info():
    """测试获取终端信息"""
    print("\n=== 测试获取终端信息 ===")
    
    response = requests.get(f"{BASE_URL}/terminals/1")
    data = response.json()
    print("终端1信息:")
    pretty_print(data)

def test_delete_terminal():
    """测试删除终端"""
    print("\n=== 测试删除终端 ===")
    
    # 删除终端
    response = requests.delete(f"{BASE_URL}/terminals/1")
    print(f"删除终端状态码: {response.status_code}")
    
    # 检查终端列表
    response = requests.get(f"{BASE_URL}/")
    data = response.json()
    print("\n删除后终端列表:")
    pretty_print(data)

if __name__ == "__main__":
    try:
        # 运行测试
        test_create_terminal()
        time.sleep(1)
        
        test_get_terminal_info()
        time.sleep(1)
        
        test_delete_terminal()
        
    except Exception as e:
        print(f"测试出错: {e}") 