# 导出各个端点模块 



def standardize_response(success: bool, message: str = None, data: any = None, error_code: int = None):
    """
    标准化API响应格式
    
    Args:
        success: 操作是否成功
        message: 操作消息
        data: 返回数据
        error_code: 错误码(仅当success=False时有效)
    
    Returns:
        dict: 标准格式的响应
    """
    code = 0 if success else (error_code or 1)
    
    return {
        "code": code,
        "message": message,
        "data": data
    }