from typing import Dict, Any, Tuple, Optional, List

class DataTypeHandler:
    @staticmethod
    def get_subscription_key(data_type: str, terminal_id: int, params: Dict[str, Any]) -> Optional[Tuple]:
        """根据数据类型和参数生成订阅键"""
        if data_type == "kline":
            symbol = params.get("symbol")
            interval = params.get("interval")
            if not symbol or not interval:
                return None
            return ("kline", terminal_id, symbol, interval)
            
        elif data_type == "order":
            account_id = params.get("account_id")
            if not account_id:
                return None
            return ("order", terminal_id, account_id)
            
        elif data_type == "position":
            account_id = params.get("account_id")
            symbol = params.get("symbol")  # 可选
            if not account_id:
                return None
            return ("position", terminal_id, account_id, symbol) if symbol else ("position", terminal_id, account_id)
            
        elif data_type == "account":
            account_id = params.get("account_id")
            if not account_id:
                return None
            return ("account", terminal_id, account_id)
            
        elif data_type == "tick":
            symbol = params.get("symbol")
            if not symbol:
                return None
            return ("tick", terminal_id, symbol)
            
        return None  # 未知数据类型
    
    @staticmethod
    def get_required_params(data_type: str) -> List[str]:
        """获取指定数据类型的必要参数"""
        if data_type == "kline":
            return ["symbol", "interval"]
        elif data_type == "order":
            return ["account_id"]
        elif data_type == "position":
            return ["account_id"]
        elif data_type == "account":
            return ["account_id"]
        elif data_type == "tick":
            return ["symbol"]
        return []
    
    @staticmethod
    def validate_params(data_type: str, params: Dict[str, Any]) -> Tuple[bool, str]:
        """验证参数是否满足数据类型要求"""
        required_params = DataTypeHandler.get_required_params(data_type)
        
        for param in required_params:
            if param not in params or not params[param]:
                return False, f"缺少必要参数 {param}"
                
        return True, "" 